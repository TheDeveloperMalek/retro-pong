use super::*;

impl Hub {
    pub(super) fn quick_match(&mut self, player_id: &str, mode: RoomMode) {
        self.quick_queue.retain(|(id, _)| self.players.get(id)
            .map(|player| player.connection_id.is_some() && player.room_code.is_none())
            .unwrap_or(false));
        if self.players.get(player_id).and_then(|player| player.room_code.as_ref()).is_some() {
            self.send_player(player_id, error("already_in_room", "Leave the current room before matchmaking."));
            return;
        }
        if !self.quick_queue.iter().any(|(id, queued_mode)| id == player_id && *queued_mode == mode) {
            self.quick_queue.push_back((player_id.to_string(), mode));
        }
        let required = match mode {
            RoomMode::Classic => 2,
            RoomMode::Teams => 4,
            RoomMode::Tournament => 4,
        };
        let matching: Vec<String> = self.quick_queue.iter()
            .filter(|(_, queued_mode)| *queued_mode == mode)
            .take(required)
            .map(|(id, _)| id.clone())
            .collect();
        if matching.len() < required {
            let position = self.quick_queue.iter().filter(|(_, queued_mode)| *queued_mode == mode).position(|(id, _)| id == player_id).unwrap_or(0) + 1;
            self.send_player(player_id, ServerMessage::QueueStatus { queued: true, position });
            return;
        }
        let selected: std::collections::HashSet<String> = matching.iter().cloned().collect();
        self.quick_queue.retain(|(id, _)| !selected.contains(id));
        let host_id = matching[0].clone();
        if self.players.get(&host_id).and_then(|player| player.room_code.as_ref()).is_some() {
            self.send_player(player_id, error("matchmaking_stale", "A queued player has already joined a room. Please queue again."));
            return;
        }
        self.create_room(&host_id, RoomVisibility::Public, mode, self.winning_score);
        let room_code = self.players.get(&host_id).and_then(|player| player.room_code.clone()).unwrap_or_default();
        if room_code.is_empty() { return; }
        for other_id in matching.iter().skip(1) {
            if self.players.get(other_id).and_then(|player| player.room_code.as_ref()).is_none() {
                self.join_room(other_id, &room_code, None);
            }
        }
        if mode == RoomMode::Tournament {
            self.start_tournament(&host_id);
        }
    }

    pub(super) fn list_rooms(&self, connection_id: u64) {
        let rooms = self.rooms.values()
            .filter(|room| room.visibility == RoomVisibility::Public)
            .filter(|room| room.player_ids.len() < room.max_players)
            .filter(|room| !room.in_progress || room.mode == RoomMode::Teams)
            .filter_map(|room| self.room_summary(&room.code))
            .collect();
        self.send(connection_id, ServerMessage::RoomList { rooms });
    }

    pub(super) fn list_players(&self, connection_id: u64, requesting_player_id: &str) {
        let players = self.players.iter()
            .filter(|(id, player)| id.as_str() != requesting_player_id && player.connection_id.is_some())
            .filter_map(|(id, _)| self.player_summary(id))
            .collect();
        self.send(connection_id, ServerMessage::PlayerList { players });
    }

    pub(super) fn invite_friend(&mut self, player_id: &str, target_player_id: Option<String>) {
        let Some(room_code) = self.players.get(player_id).and_then(|player| player.room_code.clone()) else {
            self.send_player(player_id, error("not_in_room", "Create or join a room before inviting a friend."));
            return;
        };
        if let Some(target_id) = target_player_id.as_deref() {
            if target_id == player_id || !self.players.get(target_id).map(|player| player.connection_id.is_some()).unwrap_or(false) {
                self.send_player(player_id, error("player_unavailable", "The selected player is not connected."));
                return;
            }
        }
        let invite_code = self.next_code("I");
        self.invites.insert(invite_code.clone(), Invite {
            room_code: room_code.clone(),
            expires_at: Instant::now() + Duration::from_secs(10 * 60),
        });
        if let Some(target_id) = target_player_id {
            let from_name = self.players.get(player_id).map(|player| player.name.clone()).unwrap_or_default();
            self.send_player(&target_id, ServerMessage::InviteReceived {
                from_name, invite_code: invite_code.clone(), room_code: room_code.clone(),
            });
        }
        self.send_player(player_id, ServerMessage::InviteCreated { invite_code, room_code });
    }
}
