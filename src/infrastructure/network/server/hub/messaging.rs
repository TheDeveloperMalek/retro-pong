

use super::*;

impl Hub {
    pub(super) fn send(&self, connection_id: u64, message: ServerMessage) {
        let Some(peer) = self.peers.get(&connection_id) else {
            return;
        };

        
        let _ = peer.outbound.send(message);
    }

    pub(super) fn send_player(&self, player_id: &str, message: ServerMessage) {
        let connection_id = self
            .players
            .get(player_id)
            .and_then(|player| player.connection_id);
        if let Some(connection_id) = connection_id {
            self.send(connection_id, message);
        }
    }

    pub(super) fn broadcast_room(&self, room_code: &str, message: ServerMessage) {
        let Some(player_ids) = self
            .rooms
            .get(room_code)
            .map(|room| room.player_ids.clone())
        else {
            return;
        };

        for player_id in player_ids {
            self.send_player(&player_id, message.clone());
        }
    }
}
