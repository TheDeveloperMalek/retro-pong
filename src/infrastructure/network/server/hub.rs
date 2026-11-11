

use super::simulation::GameState;
use crate::network::protocol::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::Sender,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const RECONNECT_GRACE: Duration = Duration::from_secs(30);
const AI_TAKEOVER_DELAY: Duration = Duration::from_secs(3);

struct Peer {
    player_id: Option<String>,
    outbound: Sender<ServerMessage>,
}

struct Player {
    id: String,
    name: String,
    reconnect_token: String,
    connection_id: Option<u64>,
    room_code: Option<String>,
    team: u8,
    ready: bool,
    vertical_input: i8,
    disconnected_at: Option<Instant>,
}

struct Tournament {
    rounds: Vec<Vec<Vec<String>>>,
    round_index: usize,
    match_index: usize,
    winners: Vec<String>,
    champion: Option<String>,
}

struct Room {
    code: String,
    visibility: RoomVisibility,
    mode: RoomMode,
    host_id: String,
    player_ids: Vec<String>,
    max_players: usize,
    winning_score: u32,
    in_progress: bool,
    game: GameState,
    tournament: Option<Tournament>,
    active_pair: [String; 2],
    last_snapshot: Instant,
}

struct Invite {
    room_code: String,
    expires_at: Instant,
}

pub(super) struct Hub {
    peers: HashMap<u64, Peer>,
    players: HashMap<String, Player>,
    rooms: HashMap<String, Room>,
    invites: HashMap<String, Invite>,
    quick_queue: VecDeque<(String, RoomMode)>,
    next_id: u64,
    winning_score: u32,
    settings: NetworkGameSettings,
}

impl Hub {

    pub(super) fn with_settings(winning_score: u32, settings: NetworkGameSettings) -> Self {
        Self {
            peers: HashMap::new(),
            players: HashMap::new(),
            rooms: HashMap::new(),
            invites: HashMap::new(),
            quick_queue: VecDeque::new(),
            next_id: 1,
            winning_score: winning_score.clamp(1, 99),
            settings,
        }
    }

    pub(super) fn next_code(&mut self, prefix: &str) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        const CODE_LENGTH: usize = 6;

        let id = self.next_id;
        self.next_id += 1;

        let mut value = id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ now_millis();
        let mut code = String::with_capacity(CODE_LENGTH);
        for _ in 0..CODE_LENGTH {
            let index = value as usize % ALPHABET.len();
            code.push(ALPHABET[index] as char);
            value = value.rotate_left(7).wrapping_add(0xA076_1D64_78BD_642F);
        }

        format!("{prefix}{code}")
    }

}

fn error(code: &str, message: &str) -> ServerMessage {
    ServerMessage::Error { code: code.to_string(), message: message.to_string() }
}

fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[path = "hub/match_control.rs"]
mod match_control;
#[path = "hub/messaging.rs"]
mod messaging;
#[path = "hub/matchmaking.rs"]
mod matchmaking;
#[path = "hub/rooms.rs"]
mod rooms;
#[path = "hub/requests.rs"]
mod requests;
#[path = "hub/runtime.rs"]
mod runtime;
#[path = "hub/sessions.rs"]
mod sessions;
#[path = "hub/tournaments.rs"]
mod tournaments;

