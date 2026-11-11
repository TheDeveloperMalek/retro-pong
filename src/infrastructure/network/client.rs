

use super::{protocol::*, server::{self, ServerConfig}};
use crate::{
    localization::tr,
    persistence::{Language, NetworkPreferences, Settings},
};
use bevy::prelude::*;
use std::{
    sync::{mpsc::{self, Receiver, Sender}, Mutex},
    thread,
    time::Duration,
};

#[path = "client/controls.rs"]
mod controls;
#[path = "client/events.rs"]
mod events;
#[path = "client/lobby.rs"]
mod lobby;
#[path = "client/snapshot.rs"]
mod snapshot;
#[path = "client/transport.rs"]
mod transport;

pub use controls::network_controls;
pub use events::poll_events;
pub use lobby::network_lobby_controls;
pub use snapshot::apply_snapshot;
pub use transport::discover_lan_servers;

use transport::{connection_worker, local_lan_ipv4, normalize_endpoint};




pub(super) fn localized_runtime_status(status: &str, language: Language) -> String {
    
    
    
    const STATIC_STATUS_KEYS: &[&str] = &[
        "OFFLINE",
        "CONNECTED",
        "DISCONNECTED",
        "MATCHMAKING CANCELLED",
        "NO GAME FOUND ON THIS LAN // HOST A GAME OR ENTER AN ADDRESS",
        "SERVER START FAILED",
        "NO PENDING INVITATION",
        "CONNECT TO A SERVER FIRST",
        "CONNECT AND JOIN A ROOM FIRST",
        "CREATE A ROOM OR INVITE FIRST",
        "CREATE OR JOIN A ROOM FIRST",
        "HOST A GAME FIRST TO GET A SHARE ADDRESS",
        "HOST ADDRESS COPIED TO CLIPBOARD",
        "ROOM CODE COPIED TO CLIPBOARD",
        "FRIEND INVITE CODE COPIED",
        "YOUR PLAYER ID COPIED",
        "COULD NOT ACCESS THE SYSTEM CLIPBOARD",
        "SYSTEM CLIPBOARD PASTE IS NOT AVAILABLE",
        "CONNECTION FAILED CHECK ADDRESS",
    ];

    for source_language in Language::ALL {
        for key in STATIC_STATUS_KEYS {
            if status == tr(source_language, key) {
                return tr(language, key);
            }
        }

        let disconnected_prefix = format!("{}: ", tr(source_language, "DISCONNECTED"));
        if let Some(reason) = status.strip_prefix(&disconnected_prefix) {
            let reason_key = "CONNECTION FAILED CHECK ADDRESS";
            if reason == tr(source_language, reason_key) {
                return format!("{}: {}", tr(language, "DISCONNECTED"), tr(language, reason_key));
            }
        }

        let connecting_prefix = format!("{} ", tr(source_language, "CONNECTING TO"));
        if let Some(endpoint) = status.strip_prefix(&connecting_prefix) {
            return format!("{} {endpoint}", tr(language, "CONNECTING TO"));
        }

        let searching_prefix = tr(source_language, "SEARCHING FOR RETRO PONG GAMES ON YOUR LAN...");
        if status == searching_prefix || status.starts_with(&searching_prefix) {
            return tr(language, "SEARCHING FOR GAMES ON LAN");
        }

        let found_prefix = format!("{} ", tr(source_language, "FOUND SERVER"));
        if let Some(rest) = status.strip_prefix(&found_prefix) {
            let joining_suffix = format!(" // {}...", tr(source_language, "JOINING SERVER"));
            if let Some(endpoint) = rest.strip_suffix(&joining_suffix) {
                return format!("{} {endpoint} // {}...", tr(language, "FOUND SERVER"), tr(language, "JOINING SERVER"));
            }
        }

        for (key, room_key) in [("INVITE", "ROOM"), ("INVITATION RECEIVED", "ROOM")] {
            let prefix = format!("{} ", tr(source_language, key));
            if let Some(rest) = status.strip_prefix(&prefix) {
                let room_marker = format!(" // {} ", tr(source_language, room_key));
                if let Some((sender_or_code, room_code)) = rest.split_once(&room_marker) {
                    return format!("{} {} // {} {}", tr(language, key), sender_or_code, tr(language, room_key), room_code);
                }
            }
        }

        let tournament_prefix = format!("{} // ", tr(source_language, "TOURNAMENT"));
        if let Some(details) = status.strip_prefix(&tournament_prefix) {
            return format!("{} // {details}", tr(language, "TOURNAMENT"));
        }

        for key in ["TOURNAMENT CHAMPION", "CHAMPION"] {
            let prefix = format!("{}: ", tr(source_language, key));
            if let Some(name) = status.strip_prefix(&prefix) {
                return format!("{}: {name}", tr(language, key));
            }
        }

        let team_prefix = format!("{} ", tr(source_language, "TEAM"));
        if let Some(team_number) = status.strip_prefix(&team_prefix) {
            if !team_number.is_empty() && team_number.chars().all(|character| character.is_ascii_digit()) {
                return format!("{} {team_number}", tr(language, "TEAM"));
            }
        }

        let queue_prefix = format!("{} // {} ", tr(source_language, "MATCHMAKING QUEUE"), tr(source_language, "POSITION"));
        if let Some(position) = status.strip_prefix(&queue_prefix) {
            return format!("{} // {} {position}", tr(language, "MATCHMAKING QUEUE"), tr(language, "POSITION"));
        }

        for (key, suffix) in [
            ("CONNECTED", "PLAYER "),
            ("MATCH STARTED", "SERVER AUTHORITATIVE"),
            ("PLAYER JOINED", ""),
        ] {
            let prefix = if suffix.is_empty() {
                format!("{}: ", tr(source_language, key))
            } else {
                format!("{} // {suffix}", tr(source_language, key))
            };
            if let Some(value) = status.strip_prefix(&prefix) {
                let target_prefix = if suffix.is_empty() {
                    format!("{}: ", tr(language, key))
                } else {
                    format!("{} // {suffix}", tr(language, key))
                };
                return format!("{target_prefix}{value}");
            }
        }

        for key in ["PUBLIC ROOMS FOUND", "OTHER PLAYERS ONLINE"] {
            let suffix = format!(" {}", tr(source_language, key));
            if let Some(count) = status.strip_suffix(&suffix) {
                return format!("{} {}", count, tr(language, key));
            }
        }

        if let Some((player_id, grace)) = status.split_once(&format!(" // {} // ", tr(source_language, "PLAYER DISCONNECTED // AI TAKEOVER"))) {
            if !player_id.is_empty() {
                return format!("{} // {} // {}", player_id, tr(language, "PLAYER DISCONNECTED // AI TAKEOVER"), grace);
            }
        }

        let reconnect_suffix = format!(" {}", tr(source_language, "RECONNECTED"));
        if let Some(player_id) = status.strip_suffix(&reconnect_suffix) {
            return format!("{} {}", player_id, tr(language, "RECONNECTED"));
        }

        for (key, mode_keys) in [("ROOM", ["CLASSIC", "TEAM", "TOURNAMENT"])] {
            let prefix = format!("{} ", tr(source_language, key));
            if let Some(rest) = status.strip_prefix(&prefix) {
                for mode_key in mode_keys {
                    let mode_prefix = format!(" // {} ", tr(source_language, mode_key));
                    if let Some(team) = rest.split_once(&mode_prefix).map(|(_, team)| team) {
                        let code = rest.split_once(&mode_prefix).map(|(code, _)| code).unwrap_or_default();
                        return format!("{} {} // {} {}", tr(language, key), code, tr(language, mode_key), team);
                    }
                }
            }
        }

        let network_error_prefix = format!("{}: ", tr(source_language, "NETWORK ERROR"));
        if let Some(error_text) = status.strip_prefix(&network_error_prefix) {
            const ERROR_KEYS: &[&str] = &[
                "ERROR ALREADY IN ROOM", "ERROR ROOM NOT FOUND", "ERROR PRIVATE ROOM",
                "ERROR MATCH IN PROGRESS", "ERROR ROOM FULL", "ERROR HOST ONLY",
                "ERROR ALREADY STARTED", "ERROR TOURNAMENT START", "ERROR MATCHMAKING STALE",
                "ERROR NOT IN ROOM", "ERROR PLAYER UNAVAILABLE", "ERROR INVALID BRACKET SIZE",
                "ERROR NOT ENOUGH PLAYERS", "CONNECTION FAILED CHECK ADDRESS",
            ];
            for error_key in ERROR_KEYS {
                if error_text == tr(source_language, error_key) {
                    return format!("{}: {}", tr(language, "NETWORK ERROR"), tr(language, error_key));
                }
            }
        }
    }

    status.to_string()
}

enum ClientEvent {
    Connected,
    Disconnected(String),
    Message(ServerMessage),
    DiscoveryCompleted { endpoint: Option<String>, player_name: String },
    ServerStartFailed(String),
}


#[derive(Resource)]
pub struct NetworkMenuState {
    pub player_name: String,
    pub server_address: String,
    pub room_code: String,
    pub target_player_id: String,
}

impl Default for NetworkMenuState {
    fn default() -> Self {
        Self::from_preferences(&NetworkPreferences::default())
    }
}

impl NetworkMenuState {
    pub fn from_preferences(preferences: &NetworkPreferences) -> Self {
        let player_name = sanitize_preference(&preferences.player_name, 16, "PLAYER");
        let server_address = sanitize_preference(&preferences.server_address, 96, "127.0.0.1:41123");
        Self {
            player_name,
            server_address,
            room_code: String::new(),
            target_player_id: String::new(),
        }
    }
}

fn sanitize_preference(value: &str, max_chars: usize, fallback: &str) -> String {
    let value: String = value.trim().chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect();
    if value.trim().is_empty() { fallback.to_string() } else { value }
}

#[derive(Resource)]
pub struct NetworkRuntime {
    outbound: Option<Sender<ClientMessage>>,
    events: Mutex<Receiver<ClientEvent>>,
    event_sender: Sender<ClientEvent>,
    pub connected: bool,
    pub status: String,
    pub player_id: Option<String>,
    pub reconnect_token: Option<String>,
    pub room_code: Option<String>,
    pub invite_code: Option<String>,
    pub pending_invite: Option<String>,
    pub target_player_id: Option<String>,
    pub team: u8,
    pub host: bool,
    pub mode: RoomMode,
    pub rooms: Vec<RoomSummary>,
    pub players: Vec<PlayerSummary>,
    pub embedded_server_started: bool,
    pub host_lan_address: Option<String>,
    active_endpoint: Option<String>,
    active_player_name: Option<String>,
    pub latest_snapshot: Option<MatchSnapshot>,
    
    pub last_powerup_event_id: u64,
    pending_action: Option<ClientMessage>,
    input_accumulator: f32,
    pub match_started: bool,
    
    
    pub local_pause_active: bool,
}

impl Default for NetworkRuntime {
    fn default() -> Self {
        let (event_sender, receiver) = mpsc::channel();
        Self {
            outbound: None,
            events: Mutex::new(receiver),
            event_sender,
            connected: false,
            status: "OFFLINE".into(),
            player_id: None,
            reconnect_token: None,
            room_code: None,
            invite_code: None,
            pending_invite: None,
            target_player_id: None,
            team: 0,
            host: false,
            mode: RoomMode::Classic,
            rooms: Vec::new(),
            players: Vec::new(),
            embedded_server_started: false,
            host_lan_address: None,
            active_endpoint: None,
            active_player_name: None,
            latest_snapshot: None,
            last_powerup_event_id: 0,
            pending_action: None,
            input_accumulator: 0.0,
            match_started: false,
            local_pause_active: false,
        }
    }
}

impl NetworkRuntime {
    pub fn send(&self, message: ClientMessage) {
        if let Some(sender) = &self.outbound { let _ = sender.send(message); }
    }

    fn connect(&mut self, endpoint: String, player_name: String, action: ClientMessage) {
        let endpoint = normalize_endpoint(&endpoint);
        let (outbound_tx, outbound_rx) = mpsc::channel();
        let event_tx = self.event_sender.clone();
        self.outbound = Some(outbound_tx);
        self.active_endpoint = Some(endpoint.clone());
        self.active_player_name = Some(player_name.clone());
        self.connected = false;
        self.status = format!("CONNECTING TO {endpoint}");
        self.pending_action = Some(action);
        let token = self.reconnect_token.clone();
        thread::spawn(move || connection_worker(endpoint, player_name, token, outbound_rx, event_tx));
    }

    
    
    pub fn connect_or_send(&mut self, endpoint: String, player_name: String, action: ClientMessage) {
        let endpoint = normalize_endpoint(&endpoint);
        let same_connection = self.active_endpoint.as_deref() == Some(endpoint.as_str())
            && self.active_player_name.as_deref() == Some(player_name.as_str());
        if same_connection && self.outbound.is_some() {
            if self.connected { self.send(action); }
            else { self.pending_action = Some(action); }
        } else {
            self.connect(endpoint, player_name, action);
        }
    }

    pub fn discover_and_join(&mut self, player_name: String) {
        self.status = "SEARCHING FOR RETRO PONG GAMES ON YOUR LAN...".into();
        let events = self.event_sender.clone();
        thread::spawn(move || {
            let endpoint = discover_lan_servers(Duration::from_secs(2)).into_iter().next();
            let _ = events.send(ClientEvent::DiscoveryCompleted { endpoint, player_name });
        });
    }

    
    
    pub fn host_on_this_pc(&mut self, player_name: String, settings: Settings) {
        let share_address = format!("{}:41123", local_lan_ipv4());
        self.host_lan_address = Some(share_address.clone());
        if !self.embedded_server_started {
            self.embedded_server_started = true;
            let events = self.event_sender.clone();
            thread::spawn(move || {
                if let Err(error) = server::run_server(ServerConfig {
                    bind_addr: "0.0.0.0:41123".into(),
                    winning_score: settings.winning_score.clamp(1, 99),
                    settings: network_game_settings(&settings),
                }) {
                    let _ = events.send(ClientEvent::ServerStartFailed(error.to_string()));
                }
            });
        }
        self.connect_or_send(
            "127.0.0.1:41123".into(),
            player_name,
            ClientMessage::ListRooms,
        );
    }
}


fn network_game_settings(settings: &Settings) -> NetworkGameSettings {
    let timed = |value: &crate::persistence::TimedPowerUpSettings| NetworkTimedPowerUpSettings {
        enabled: value.enabled,
        duration_secs: value.duration_secs,
        magnitude: value.magnitude,
        spawn_weight: value.spawn_weight,
        max_charges: value.max_charges,
        cooldown_secs: value.cooldown_secs,
    };
    let item = |value: &crate::persistence::PowerUpItemSettings| NetworkPowerUpItemSettings {
        enabled: value.enabled,
        duration_secs: value.duration_secs,
        magnitude: value.magnitude,
        spawn_weight: value.spawn_weight,
        max_charges: value.max_charges,
        cooldown_secs: value.cooldown_secs,
        min_projectiles: value.min_projectiles,
        max_projectiles: value.max_projectiles,
        opponent_blocks: value.opponent_blocks,
        throws: value.throws,
    };

    NetworkGameSettings {
        powerups_enabled: settings.powerups_enabled,
        manual_powerup_activation: settings.manual_powerup_activation,
        max_active_balls: settings.max_active_balls,
        inventory_slots: settings.inventory_slots,
        powerup_spawn_interval_secs: settings.powerup_spawn_interval_secs,
        double_ball_enabled: settings.double_ball_enabled,
        double_ball_spawn_probability: settings.double_ball_spawn_probability,
        double_ball_max_charges: settings.double_ball_max_charges,
        double_ball_cooldown_secs: settings.double_ball_cooldown_secs,
        speed_powerup: timed(&settings.speed_powerup),
        slow_powerup: timed(&settings.slow_powerup),
        enlarge_powerup: timed(&settings.enlarge_powerup),
        fireball_powerup: item(&settings.fireball_powerup),
        snowball_powerup: item(&settings.snowball_powerup),
        reversal_powerup: item(&settings.reversal_powerup),
        sticky_ball_powerup: item(&settings.sticky_ball_powerup),
        ball_storm_powerup: item(&settings.ball_storm_powerup),
        boomerang_powerup: item(&settings.boomerang_powerup),
        magnetic_field_powerup: item(&settings.magnetic_field_powerup),
        teleport_ball_powerup: item(&settings.teleport_ball_powerup),
        energy_shield_powerup: item(&settings.energy_shield_powerup),
        overdrive_powerup: item(&settings.overdrive_powerup),
        phantom_ball_powerup: item(&settings.phantom_ball_powerup),
    }
}


