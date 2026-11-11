use super::*;

impl Hub {
    pub(super) fn start_match(&mut self, player_id: &str) {
        let Some(room_code) = self.players.get(player_id).and_then(|player| player.room_code.clone()) else { return; };
        let Some(room) = self.rooms.get(&room_code) else { return; };
        if room.host_id != player_id {
            self.send_player(player_id, error("host_only", "Only the room host can start the match."));
            return;
        }
        if room.in_progress {
            self.send_player(player_id, error("already_started", "This room already has an active match."));
            return;
        }
        if room.mode == RoomMode::Tournament {
            self.send_player(player_id, error("tournament_start", "Use Start Tournament to build and begin the bracket."));
            return;
        }
        let required_players = if room.mode == RoomMode::Teams { 4 } else { 2 };
        if room.player_ids.len() < required_players {
            self.send_player(player_id, error(
                "not_enough_players",
                if room.mode == RoomMode::Teams {
                    "A 2v2 team match needs four connected players."
                } else {
                    "At least two players are required."
                },
            ));
            return;
        }
        let match_settings = self.settings.clone();
        if let Some(room) = self.rooms.get_mut(&room_code) {
            room.in_progress = true;
            room.game = GameState::with_settings(room.winning_score, match_settings);
            room.active_pair = [room.player_ids[0].clone(), room.player_ids[1].clone()];
        }
        self.assign_room_teams(&room_code);
        if let Some(room) = self.rooms.get(&room_code) {
            self.broadcast_room(&room_code, ServerMessage::MatchStarting {
                snapshot: room.game.snapshot(self.active_team_names(room)),
            });
        }
    }

    pub(super) fn release_ball(&mut self, player_id: &str) {
        let Some((room_code, team)) = self.players.get(player_id)
            .and_then(|player| player.room_code.clone().map(|room_code| (room_code, player.team))) else { return; };
        if let Some(room) = self.rooms.get_mut(&room_code) {
            if room.in_progress { room.game.release_ball(team); }
        }
    }

    pub(super) fn activate_powerup(&mut self, player_id: &str, slot: usize) {
        let Some(player) = self.players.get(player_id) else { return; };
        let Some(room_code) = player.room_code.clone() else { return; };
        let team = player.team;
        if let Some(room) = self.rooms.get_mut(&room_code) { room.game.activate_powerup(team, slot); }
    }
}
