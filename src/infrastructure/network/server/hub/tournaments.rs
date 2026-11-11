use super::*;

impl Hub {
    pub(super) fn start_tournament(&mut self, player_id: &str) {
        let Some(room_code) = self.players.get(player_id).and_then(|player| player.room_code.clone()) else { return; };
        let Some(room) = self.rooms.get(&room_code) else { return; };
        if room.host_id != player_id || room.mode != RoomMode::Tournament {
            self.send_player(player_id, error("host_only", "Only the tournament host can start this bracket."));
            return;
        }
        if room.tournament.is_some() {
            self.send_player(player_id, error("already_started", "This tournament bracket has already started."));
            return;
        }
        let count = room.player_ids.len();
        if !matches!(count, 2 | 4 | 8) {
            self.send_player(player_id, error("invalid_bracket_size", "A single-elimination tournament needs exactly 2, 4, or 8 players."));
            return;
        }
        let participants = room.player_ids.clone();
        let mut round = Vec::new();
        for pair in participants.chunks_exact(2) { round.push(vec![pair[0].clone(), pair[1].clone()]); }
        if let Some(room) = self.rooms.get_mut(&room_code) {
            room.tournament = Some(Tournament { rounds: vec![round], round_index: 0, match_index: 0, winners: Vec::new(), champion: None });
            room.in_progress = true;
        }
        self.start_current_tournament_match(&room_code);
    }

    pub(super) fn start_current_tournament_match(&mut self, room_code: &str) {
        let pair = self.rooms.get(room_code).and_then(|room| {
            let tournament = room.tournament.as_ref()?;
            tournament.rounds.get(tournament.round_index)?.get(tournament.match_index).cloned()
        });
        let Some(pair) = pair else { return; };
        if pair.len() != 2 { return; }
        let match_settings = self.settings.clone();
        if let Some(room) = self.rooms.get_mut(room_code) {
            room.active_pair = [pair[0].clone(), pair[1].clone()];
            room.game = GameState::with_settings(room.winning_score, match_settings);
            room.in_progress = true;
        }
        self.assign_room_teams(room_code);
        if let Some(room) = self.rooms.get(room_code) {
            self.broadcast_room(room_code, ServerMessage::MatchStarting {
                snapshot: room.game.snapshot(self.active_team_names(room)),
            });
        }
        self.broadcast_tournament_update(room_code);
    }

    pub(super) fn advance_tournament(&mut self, room_code: &str, winner_team: u8) -> (Option<String>, bool) {
        let mut should_start_next = false;
        let mut champion = None;
        if let Some(room) = self.rooms.get_mut(room_code) {
            let Some(tournament) = room.tournament.as_mut() else { return (None, false); };
            let Some(pair) = tournament.rounds.get(tournament.round_index).and_then(|round| round.get(tournament.match_index)).cloned() else {
                return (None, false);
            };
            let Some(winner) = pair.get(winner_team as usize).cloned() else { return (None, false); };
            tournament.winners.push(winner);
            tournament.match_index += 1;
            if tournament.match_index < tournament.rounds[tournament.round_index].len() {
                should_start_next = true;
            } else if tournament.winners.len() == 1 {
                tournament.champion = tournament.winners.first().cloned();
                champion = tournament.champion.clone();
                room.in_progress = false;
            } else {
                let winners = std::mem::take(&mut tournament.winners);
                let mut next_round = Vec::new();
                for pair in winners.chunks_exact(2) { next_round.push(vec![pair[0].clone(), pair[1].clone()]); }
                tournament.rounds.push(next_round);
                tournament.round_index += 1;
                tournament.match_index = 0;
                should_start_next = true;
            }
        }
        (champion, should_start_next)
    }

    pub(super) fn broadcast_tournament_update(&self, room_code: &str) {
        let Some(room) = self.rooms.get(room_code) else { return; };
        let Some(tournament) = room.tournament.as_ref() else { return; };
        let participants = room.player_ids.iter().filter_map(|id| self.players.get(id).map(|p| p.name.clone())).collect();
        self.broadcast_room(room_code, ServerMessage::TournamentUpdate {
            round: tournament.round_index + 1,
            match_index: tournament.match_index + 1,
            participants,
            champion: tournament.champion.clone(),
        });
    }
}
