use super::*;

impl Hub {
    pub(super) fn create_room(&mut self, player_id: &str, visibility: RoomVisibility, mode: RoomMode, winning_score: u32) {
        if self.players.get(player_id).and_then(|player| player.room_code.as_ref()).is_some() {
            self.send_player(player_id, error("already_in_room", "Leave the current room before creating another."));
            return;
        }
        let code = self.next_code("");
        let max_players = match mode { RoomMode::Classic => 2, RoomMode::Teams => 4, RoomMode::Tournament => 8 };
        let mut room = Room {
            code: code.clone(), visibility, mode, host_id: player_id.to_string(),
            player_ids: vec![player_id.to_string()], max_players, winning_score: winning_score.clamp(1, 99),
            in_progress: false, game: GameState::with_settings(winning_score, self.settings.clone()), tournament: None,
            active_pair: [player_id.to_string(), String::new()], last_snapshot: Instant::now(),
        };
        if let Some(player) = self.players.get_mut(player_id) { player.room_code = Some(code.clone()); player.team = 0; player.ready = true; }
        if mode == RoomMode::Teams { room.active_pair[1] = String::new(); }
        self.rooms.insert(code.clone(), room);
        let invite_code = if visibility == RoomVisibility::Private {
            let invite_code = self.next_code("I");
            self.invites.insert(invite_code.clone(), Invite {
                room_code: code.clone(),
                expires_at: Instant::now() + Duration::from_secs(10 * 60),
            });
            Some(invite_code)
        } else { None };
        self.send_room_joined(player_id, &code, invite_code.clone());
        if let Some(invite_code) = invite_code {
            self.send_player(player_id, ServerMessage::InviteCreated { invite_code, room_code: code });
        }
    }

    pub(super) fn join_room(&mut self, player_id: &str, room_code: &str, invite_code: Option<&str>) {
        if self.players.get(player_id).and_then(|player| player.room_code.as_ref()).is_some() {
            self.send_player(player_id, error("already_in_room", "Leave the current room before joining another."));
            return;
        }
        let Some(room_snapshot) = self.rooms.get(room_code) else {
            self.send_player(player_id, error("room_not_found", "The room code was not found."));
            return;
        };
        if room_snapshot.visibility == RoomVisibility::Private {
            let allowed = invite_code.and_then(|code| self.invites.get(code))
                .map(|invite| invite.room_code == room_code && invite.expires_at > Instant::now())
                .unwrap_or(false);
            if !allowed {
                self.send_player(player_id, error("private_room", "A valid invitation code is required for this private room."));
                return;
            }
        }
        let tournament_started = room_snapshot.mode == RoomMode::Tournament && room_snapshot.tournament.is_some();
        if (room_snapshot.in_progress && room_snapshot.mode != RoomMode::Teams && room_snapshot.mode != RoomMode::Tournament)
            || tournament_started {
            self.send_player(player_id, error("match_in_progress", "This match or tournament has already started."));
            return;
        }
        if room_snapshot.player_ids.len() >= room_snapshot.max_players {
            self.send_player(player_id, error("room_full", "This room is full."));
            return;
        }
        let team = room_snapshot.player_ids.len() as u8 % 2;
        if let Some(room) = self.rooms.get_mut(room_code) { room.player_ids.push(player_id.to_string()); }
        if let Some(player) = self.players.get_mut(player_id) {
            player.room_code = Some(room_code.to_string()); player.team = team; player.ready = true;
        }
        self.broadcast_room(room_code, ServerMessage::PlayerJoined {
            player: self.player_summary(player_id).unwrap(),
        });
        self.send_room_joined(player_id, room_code, None);
        let should_start = self.rooms.get(room_code).map(|room| {
            match room.mode {
                RoomMode::Classic => room.player_ids.len() >= 2 && !room.in_progress,
                RoomMode::Teams => room.player_ids.len() >= 4 && !room.in_progress,
                RoomMode::Tournament => false,
            }
        }).unwrap_or(false);
        if should_start {
            let match_settings = self.settings.clone();
            if let Some(room) = self.rooms.get_mut(room_code) {
                room.in_progress = true;
                room.game = GameState::with_settings(room.winning_score, match_settings);
                room.active_pair = [room.player_ids[0].clone(), room.player_ids[1].clone()];
            }
            self.assign_room_teams(room_code);
            if let Some(room) = self.rooms.get(room_code) {
                self.broadcast_room(room_code, ServerMessage::MatchStarting {
                    snapshot: room.game.snapshot(self.active_team_names(room)),
                });
            }
        }
    }

    pub(super) fn leave_room(&mut self, player_id: &str) {
        let Some(room_code) = self.players.get(player_id).and_then(|player| player.room_code.clone()) else { return; };

        
        
        
        let forfeit_team = self.rooms.get(&room_code).and_then(|room| {
            if room.mode != RoomMode::Tournament || !room.in_progress { return None; }
            room.active_pair.iter().position(|active_id| active_id == player_id)
                .map(|losing_team| (1_usize.saturating_sub(losing_team)) as u8)
        });
        if let Some(winning_team) = forfeit_team {
            let (champion, start_next) = self.advance_tournament(&room_code, winning_team);
            self.broadcast_room(&room_code, ServerMessage::MatchEnded {
                winning_team,
                tournament_champion: champion.clone(),
            });
            if start_next {
                self.start_current_tournament_match(&room_code);
            } else {
                self.broadcast_tournament_update(&room_code);
            }
        }

        let mut room_empty = false;
        if let Some(room) = self.rooms.get_mut(&room_code) {
            room.player_ids.retain(|id| id != player_id);
            if room.host_id == player_id {
                room.host_id = room.player_ids.first().cloned().unwrap_or_default();
            }
            room_empty = room.player_ids.is_empty();
        }
        if room_empty { self.rooms.remove(&room_code); }
        else { self.broadcast_room(&room_code, ServerMessage::PlayerLeft { player_id: player_id.to_string(), reconnect_grace_secs: 0 }); }
        if let Some(player) = self.players.get_mut(player_id) {
            player.room_code = None;
            player.ready = false;
            player.vertical_input = 0;
        }
    }

    pub(super) fn notify_room(&self, player_id: &str) {
        let Some(room_code) = self
            .players
            .get(player_id)
            .and_then(|player| player.room_code.as_deref())
        else {
            return;
        };

        if let Some(player) = self.player_summary(player_id) {
            self.broadcast_room(room_code, ServerMessage::PlayerJoined { player });
        }
    }

    pub(super) fn assign_room_teams(&mut self, room_code: &str) {
        let Some(room) = self.rooms.get(room_code) else { return; };
        let mode = room.mode;
        let players = if mode == RoomMode::Tournament { room.active_pair.to_vec() } else { room.player_ids.clone() };
        for (index, player_id) in players.iter().enumerate() {
            if let Some(player) = self.players.get_mut(player_id) { player.team = (index % 2) as u8; }
        }
    }

    pub(super) fn send_room_joined(&self, player_id: &str, room_code: &str, invite_code: Option<String>) {
        let Some(room) = self.room_summary(room_code) else { return; };
        let player = self.players.get(player_id);
        self.send_player(player_id, ServerMessage::RoomJoined {
            room, player_id: player_id.to_string(), team: player.map(|p| p.team).unwrap_or(0),
            host: self.rooms.get(room_code).map(|room| room.host_id == player_id).unwrap_or(false),
            invite_code,
        });
    }

    pub(super) fn room_summary(&self, room_code: &str) -> Option<RoomSummary> {
        let room = self.rooms.get(room_code)?;
        let players = room.player_ids.iter().filter_map(|id| self.player_summary(id)).collect();
        let host_name = self.players.get(&room.host_id).map(|player| player.name.clone()).unwrap_or_default();
        Some(RoomSummary {
            room_code: room.code.clone(), visibility: room.visibility, mode: room.mode,
            host_name, players, max_players: room.max_players, in_progress: room.in_progress,
            winning_score: room.winning_score,
        })
    }

    pub(super) fn player_summary(&self, player_id: &str) -> Option<PlayerSummary> {
        let player = self.players.get(player_id)?;
        Some(PlayerSummary {
            player_id: player.id.clone(), display_name: player.name.clone(), team: player.team,
            ready: player.ready, connected: player.connection_id.is_some(),
        })
    }

    pub(super) fn active_team_names(&self, room: &Room) -> [String; 2] {
        [0, 1].map(|team| {
            room.active_pair.get(team).and_then(|id| self.players.get(id))
                .map(|player| player.name.clone()).unwrap_or_else(|| format!("TEAM {}", team + 1))
        })
    }
}
