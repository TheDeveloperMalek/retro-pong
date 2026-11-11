

use super::*;

impl Hub {
    pub(in crate::network::server) fn on_connected(
        &mut self,
        connection_id: u64,
        outbound: Sender<ServerMessage>,
    ) {
        self.peers.insert(
            connection_id,
            Peer {
                player_id: None,
                outbound,
            },
        );
    }

    pub(in crate::network::server) fn on_disconnected(&mut self, connection_id: u64) {
        let disconnected_player = self.peers.remove(&connection_id).and_then(|peer| {
            let player_id = peer.player_id?;
            let player = self.players.get_mut(&player_id)?;

            player.connection_id = None;
            player.disconnected_at = Some(Instant::now());
            player
                .room_code
                .clone()
                .map(|room_code| (player_id, room_code))
        });

        if let Some((player_id, room_code)) = disconnected_player {
            self.broadcast_room(
                &room_code,
                ServerMessage::PlayerLeft {
                    player_id,
                    reconnect_grace_secs: RECONNECT_GRACE.as_secs() as u32,
                },
            );
        }
    }

    pub(super) fn hello(
        &mut self,
        connection_id: u64,
        player_name: String,
        reconnect_token: Option<String>,
    ) {
        let Some(peer) = self.peers.get(&connection_id) else {
            return;
        };

        if peer.player_id.is_some() {
            self.send(
                connection_id,
                error(
                    "already_authenticated",
                    "This connection already completed Hello.",
                ),
            );
            return;
        }

        if let Some(player_id) = self.find_reconnectable_player(reconnect_token.as_deref()) {
            self.restore_connection(connection_id, &player_id, &player_name);
            return;
        }

        self.register_new_player(connection_id, &player_name);
    }

    fn find_reconnectable_player(&self, reconnect_token: Option<&str>) -> Option<String> {
        let token = reconnect_token?;
        self.players
            .iter()
            .find(|(_, player)| {
                player.reconnect_token == token
                    && player.connection_id.is_none()
                    && player
                        .disconnected_at
                        .map(|disconnected_at| disconnected_at.elapsed() <= RECONNECT_GRACE)
                        .unwrap_or(false)
            })
            .map(|(player_id, _)| player_id.clone())
    }

    fn restore_connection(
        &mut self,
        connection_id: u64,
        player_id: &str,
        player_name: &str,
    ) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.connection_id = Some(connection_id);
            player.disconnected_at = None;
            player.vertical_input = 0;

            if !player_name.trim().is_empty() {
                player.name = clean_name(player_name);
            }
        }

        if let Some(peer) = self.peers.get_mut(&connection_id) {
            peer.player_id = Some(player_id.to_string());
        }

        let Some(token) = self
            .players
            .get(player_id)
            .map(|player| player.reconnect_token.clone())
        else {
            return;
        };

        self.send(
            connection_id,
            ServerMessage::Welcome {
                player_id: player_id.to_string(),
                reconnect_token: token,
            },
        );

        let Some(room_code) = self
            .players
            .get(player_id)
            .and_then(|player| player.room_code.clone())
        else {
            return;
        };

        self.send_room_joined(player_id, &room_code, None);

        if let Some(room) = self.rooms.get(&room_code) {
            if room.in_progress {
                self.send(
                    connection_id,
                    ServerMessage::MatchStarting {
                        snapshot: room.game.snapshot(self.active_team_names(room)),
                    },
                );
            }
        }

        self.broadcast_room(
            &room_code,
            ServerMessage::PlayerReconnected {
                player_id: player_id.to_string(),
            },
        );
    }

    fn register_new_player(&mut self, connection_id: u64, player_name: &str) {
        let player_id = format!("p{:06}", self.next_id);
        let reconnect_token = self.next_code("R");

        self.players.insert(
            player_id.clone(),
            Player {
                id: player_id.clone(),
                name: clean_name(player_name),
                reconnect_token: reconnect_token.clone(),
                connection_id: Some(connection_id),
                room_code: None,
                team: 0,
                ready: false,
                vertical_input: 0,
                disconnected_at: None,
            },
        );

        if let Some(peer) = self.peers.get_mut(&connection_id) {
            peer.player_id = Some(player_id.clone());
        }

        self.send(
            connection_id,
            ServerMessage::Welcome {
                player_id,
                reconnect_token,
            },
        );
    }
}

fn clean_name(name: &str) -> String {
    let safe_name: String = name
        .trim()
        .chars()
        .filter(|character| !character.is_control())
        .take(20)
        .collect();

    if safe_name.is_empty() {
        "PLAYER".to_string()
    } else {
        safe_name
    }
}
