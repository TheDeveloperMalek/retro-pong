

use super::*;

impl Hub {
    pub(in crate::network::server) fn handle(
        &mut self,
        connection_id: u64,
        message: ClientMessage,
    ) {
        let message = match message {
            ClientMessage::Hello {
                player_name,
                reconnect_token,
            } => {
                self.hello(connection_id, player_name, reconnect_token);
                return;
            }
            other => other,
        };

        let Some(player_id) = self
            .peers
            .get(&connection_id)
            .and_then(|peer| peer.player_id.clone())
        else {
            self.send(
                connection_id,
                error(
                    "not_authenticated",
                    "Send Hello before any other request.",
                ),
            );
            return;
        };

        self.handle_authenticated_request(connection_id, &player_id, message);
    }

    fn handle_authenticated_request(
        &mut self,
        connection_id: u64,
        player_id: &str,
        message: ClientMessage,
    ) {
        match message {
            ClientMessage::Hello { .. } => unreachable!("Hello is handled before authentication"),
            ClientMessage::Ping { nonce } => {
                self.send(connection_id, ServerMessage::Pong { nonce });
            }
            ClientMessage::ListRooms => self.list_rooms(connection_id),
            ClientMessage::ListPlayers => self.list_players(connection_id, player_id),
            ClientMessage::CancelQueue => self.cancel_queue(player_id),
            ClientMessage::QuickMatch { mode } => self.quick_match(player_id, mode),
            ClientMessage::CreateRoom {
                visibility,
                mode,
                winning_score,
            } => self.create_room(player_id, visibility, mode, winning_score),
            ClientMessage::JoinRoom {
                room_code,
                invite_code,
            } => self.join_room(player_id, &room_code, invite_code.as_deref()),
            ClientMessage::JoinInvite { invite_code } => {
                self.join_invite(player_id, &invite_code);
            }
            ClientMessage::InviteFriend { target_player_id } => {
                self.invite_friend(player_id, target_player_id);
            }
            ClientMessage::SetReady { ready } => self.set_ready(player_id, ready),
            ClientMessage::StartMatch => self.start_match(player_id),
            ClientMessage::StartTournament => self.start_tournament(player_id),
            ClientMessage::Input { vertical } => self.update_player_input(player_id, vertical),
            ClientMessage::ActivatePowerUp { slot } => {
                self.activate_powerup(player_id, slot as usize);
            }
            ClientMessage::ReleaseBall => self.release_ball(player_id),
            ClientMessage::LeaveRoom => self.leave_room(player_id),
        }
    }

    fn cancel_queue(&mut self, player_id: &str) {
        self.quick_queue
            .retain(|(queued_id, _)| queued_id != player_id);
        self.send_player(
            player_id,
            ServerMessage::QueueStatus {
                queued: false,
                position: 0,
            },
        );
    }

    fn join_invite(&mut self, player_id: &str, invite_code: &str) {
        let room_code = self
            .invites
            .get(invite_code)
            .filter(|invite| invite.expires_at > Instant::now())
            .map(|invite| invite.room_code.clone());

        match room_code {
            Some(room_code) => self.join_room(player_id, &room_code, Some(invite_code)),
            None => self.send_player(
                player_id,
                error(
                    "invalid_invite",
                    "This invitation has expired or is invalid.",
                ),
            ),
        }
    }

    fn set_ready(&mut self, player_id: &str, ready: bool) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.ready = ready;
        }
        self.notify_room(player_id);
    }

    fn update_player_input(&mut self, player_id: &str, vertical: i8) {
        let is_active_member = self
            .players
            .get(player_id)
            .and_then(|player| {
                player
                    .room_code
                    .as_ref()
                    .and_then(|room_code| self.rooms.get(room_code))
            })
            .map(|room| {
                room.in_progress
                    && (room.mode != RoomMode::Tournament
                        || room.active_pair.iter().any(|active_id| active_id == player_id))
            })
            .unwrap_or(false);

        if !is_active_member {
            return;
        }

        if let Some(player) = self.players.get_mut(player_id) {
            player.vertical_input = vertical.clamp(-1, 1);
        }
    }
}
