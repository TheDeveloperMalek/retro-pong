use super::*;

impl Hub {
    pub(in crate::network::server) fn tick(&mut self, _dt: f32) {
        let now = Instant::now();
        self.invites.retain(|_, invite| invite.expires_at > now);
        self.quick_queue.retain(|(id, _)| self.players.get(id)
            .map(|player| player.connection_id.is_some() && player.room_code.is_none())
            .unwrap_or(false));

        let expired: Vec<String> = self.players.iter().filter_map(|(id, player)| {
            player.disconnected_at.filter(|at| at.elapsed() > RECONNECT_GRACE).map(|_| id.clone())
        }).collect();
        for player_id in expired { self.leave_room(&player_id); self.players.remove(&player_id); }

        let room_codes: Vec<String> = self.rooms.keys().cloned().collect();
        let mut finished_events: Vec<(String, u8)> = Vec::new();
        let mut snapshots: Vec<(String, MatchSnapshot)> = Vec::new();
        for room_code in room_codes {
            let mut snapshot = None;
            let mut finished = None;
            if let Some(room) = self.rooms.get_mut(&room_code) {
                if !room.in_progress || now.duration_since(room.last_snapshot) < Duration::from_millis(33) { continue; }
                room.last_snapshot = now;
                let active_ids = if room.mode == RoomMode::Tournament { room.active_pair.to_vec() } else { room.player_ids.clone() };
                let mut team_inputs: [Vec<i8>; 2] = [Vec::new(), Vec::new()];
                let mut ai_takeover = [true, true];
                for id in active_ids {
                    if let Some(player) = self.players.get(&id) {
                        if player.room_code.as_deref() != Some(room_code.as_str()) { continue; }
                        let team = player.team as usize;
                        if player.connection_id.is_some() {
                            ai_takeover[team] = false;
                            if player.vertical_input != 0 { team_inputs[team].push(player.vertical_input); }
                        } else if player.disconnected_at.map(|at| at.elapsed() < AI_TAKEOVER_DELAY).unwrap_or(false) {
                            ai_takeover[team] = false;
                        }
                    }
                }
                finished = room.game.tick(1.0 / 30.0, team_inputs, ai_takeover);
                let active_names = [0, 1].map(|team| {
                    room.active_pair.get(team).and_then(|id| self.players.get(id))
                        .map(|player| player.name.clone()).unwrap_or_else(|| format!("TEAM {}", team + 1))
                });
                snapshot = Some(room.game.snapshot(active_names.clone()));
            }
            if let Some(team) = finished { finished_events.push((room_code.clone(), team)); }
            if let Some(snapshot) = snapshot { snapshots.push((room_code, snapshot)); }
        }
        for (room_code, snapshot) in snapshots {
            let recipients = self.rooms.get(&room_code).map(|room| room.player_ids.clone()).unwrap_or_default();
            for player_id in recipients {
                if let Some(connection_id) = self.players.get(&player_id).and_then(|player| player.connection_id) {
                    self.send(connection_id, ServerMessage::Snapshot { snapshot: snapshot.clone() });
                }
            }
        }
        for (room_code, winner_team) in finished_events {
            let is_tournament = self.rooms.get(&room_code).and_then(|room| room.tournament.as_ref()).is_some();
            let (champion, start_next) = if is_tournament { self.advance_tournament(&room_code, winner_team) } else { (None, false) };
            self.broadcast_room(&room_code, ServerMessage::MatchEnded { winning_team: winner_team, tournament_champion: champion.clone() });
            if is_tournament && start_next {
                self.start_current_tournament_match(&room_code);
            } else if is_tournament {
                self.broadcast_tournament_update(&room_code);
            }
        }
    }

}

