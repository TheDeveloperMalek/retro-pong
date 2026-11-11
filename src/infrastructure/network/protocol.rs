use serde::{Deserialize, Serialize};





#[derive(Clone, Debug)]
pub struct NetworkTimedPowerUpSettings {
    pub enabled: bool,
    pub duration_secs: f32,
    pub magnitude: f32,
    pub spawn_weight: f32,
    pub max_charges: u8,
    pub cooldown_secs: f32,
}

impl Default for NetworkTimedPowerUpSettings {
    fn default() -> Self {
        Self { enabled: true, duration_secs: 8.0, magnitude: 1.0, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkPowerUpItemSettings {
    pub enabled: bool,
    pub duration_secs: f32,
    pub magnitude: f32,
    pub spawn_weight: f32,
    pub max_charges: u8,
    pub cooldown_secs: f32,
    pub min_projectiles: u8,
    pub max_projectiles: u8,
    pub opponent_blocks: u8,
    pub throws: u8,
}

impl Default for NetworkPowerUpItemSettings {
    fn default() -> Self {
        Self { enabled: true, duration_secs: 5.0, magnitude: 1.0, spawn_weight: 0.62, max_charges: 1,
            cooldown_secs: 1.0, min_projectiles: 3, max_projectiles: 5, opponent_blocks: 2, throws: 2 }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkGameSettings {
    pub powerups_enabled: bool,
    pub manual_powerup_activation: bool,
    pub max_active_balls: u8,
    pub inventory_slots: u8,
    pub powerup_spawn_interval_secs: f32,
    pub double_ball_enabled: bool,
    pub double_ball_spawn_probability: f32,
    pub double_ball_max_charges: u8,
    pub double_ball_cooldown_secs: f32,
    pub speed_powerup: NetworkTimedPowerUpSettings,
    pub slow_powerup: NetworkTimedPowerUpSettings,
    pub enlarge_powerup: NetworkTimedPowerUpSettings,
    pub fireball_powerup: NetworkPowerUpItemSettings,
    pub snowball_powerup: NetworkPowerUpItemSettings,
    pub reversal_powerup: NetworkPowerUpItemSettings,
    pub sticky_ball_powerup: NetworkPowerUpItemSettings,
    pub ball_storm_powerup: NetworkPowerUpItemSettings,
    pub boomerang_powerup: NetworkPowerUpItemSettings,
    pub magnetic_field_powerup: NetworkPowerUpItemSettings,
    pub teleport_ball_powerup: NetworkPowerUpItemSettings,
    pub energy_shield_powerup: NetworkPowerUpItemSettings,
    pub overdrive_powerup: NetworkPowerUpItemSettings,
    pub phantom_ball_powerup: NetworkPowerUpItemSettings,
}

impl Default for NetworkGameSettings {
    fn default() -> Self {
        Self {
            powerups_enabled: true,
            manual_powerup_activation: false,
            max_active_balls: 2,
            inventory_slots: 3,
            powerup_spawn_interval_secs: 10.0,
            double_ball_enabled: true,
            double_ball_spawn_probability: 0.20,
            double_ball_max_charges: 1,
            double_ball_cooldown_secs: 3.0,
            speed_powerup: NetworkTimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 1.28, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            slow_powerup: NetworkTimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 0.72, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            enlarge_powerup: NetworkTimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 1.55, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            fireball_powerup: NetworkPowerUpItemSettings { duration_secs: 5.0, magnitude: 0.78, ..NetworkPowerUpItemSettings::default() },
            snowball_powerup: NetworkPowerUpItemSettings { duration_secs: 3.0, magnitude: 1.0, ..NetworkPowerUpItemSettings::default() },
            reversal_powerup: NetworkPowerUpItemSettings { duration_secs: 0.0, magnitude: 1.0, ..NetworkPowerUpItemSettings::default() },
            sticky_ball_powerup: NetworkPowerUpItemSettings { duration_secs: 0.0, magnitude: 425.0, ..NetworkPowerUpItemSettings::default() },
            ball_storm_powerup: NetworkPowerUpItemSettings { duration_secs: 0.0, magnitude: 510.0, min_projectiles: 3, max_projectiles: 5, opponent_blocks: 2, ..NetworkPowerUpItemSettings::default() },
            boomerang_powerup: NetworkPowerUpItemSettings { duration_secs: 0.0, magnitude: 465.0, throws: 2, ..NetworkPowerUpItemSettings::default() },
            magnetic_field_powerup: NetworkPowerUpItemSettings { duration_secs: 5.0, magnitude: 0.62, ..NetworkPowerUpItemSettings::default() },
            teleport_ball_powerup: NetworkPowerUpItemSettings { duration_secs: 0.0, magnitude: 1.0, ..NetworkPowerUpItemSettings::default() },
            energy_shield_powerup: NetworkPowerUpItemSettings { duration_secs: 7.0, magnitude: 1.0, ..NetworkPowerUpItemSettings::default() },
            overdrive_powerup: NetworkPowerUpItemSettings { duration_secs: 5.0, magnitude: 1.5, ..NetworkPowerUpItemSettings::default() },
            phantom_ball_powerup: NetworkPowerUpItemSettings { duration_secs: 5.0, magnitude: 0.5, ..NetworkPowerUpItemSettings::default() },
        }
    }
}




pub mod wire_code {
    pub mod power_up {
        pub const SPEED: u8 = 0;
        pub const SLOW: u8 = 1;
        pub const ENLARGE: u8 = 2;
        pub const DOUBLE_BALL: u8 = 3;
        pub const FIREBALL: u8 = 4;
        pub const SNOWBALL: u8 = 5;
        pub const REVERSAL_BALL: u8 = 6;
        pub const STICKY_BALL: u8 = 7;
        pub const BALL_STORM: u8 = 8;
        pub const BOOMERANG_BALL: u8 = 9;
        pub const MAGNETIC_FIELD: u8 = 10;
        pub const TELEPORT_BALL: u8 = 11;
        pub const ENERGY_SHIELD: u8 = 12;
        pub const OVERDRIVE: u8 = 13;
        pub const PHANTOM_BALL: u8 = 14;
    }

    pub mod effect {
        pub const SPEED: u8 = 0;
        pub const SLOW: u8 = 1;
        pub const ENLARGE: u8 = 2;
        pub const FIREBALL_SHRINK: u8 = 3;
        pub const SNOWBALL_FREEZE: u8 = 4;
        pub const MAGNETIC_FIELD: u8 = 5;
        pub const ENERGY_SHIELD: u8 = 6;
        pub const OVERDRIVE: u8 = 7;
        pub const PHANTOM_BALL: u8 = 8;
    }

    pub mod special_ball {
        pub const NORMAL: u8 = 0;
        pub const FIREBALL: u8 = 1;
        pub const STICKY: u8 = 2;
        pub const BOOMERANG: u8 = 3;
        pub const STORM_PROJECTILE: u8 = 4;
    }

    pub mod power_up_event {
        pub const RESPAWN: u8 = 0;
        pub const COLLECT: u8 = 1;
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomVisibility {
    #[default]
    Public,
    Private,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomMode {
    #[default]
    Classic,
    Teams,
    Tournament,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { player_name: String, reconnect_token: Option<String> },
    ListRooms,
    ListPlayers,
    CancelQueue,
    QuickMatch { mode: RoomMode },
    CreateRoom { visibility: RoomVisibility, mode: RoomMode, winning_score: u32 },
    JoinRoom { room_code: String, invite_code: Option<String> },
    JoinInvite { invite_code: String },
    InviteFriend { target_player_id: Option<String> },
    SetReady { ready: bool },
    StartMatch,
    StartTournament,
    Input { vertical: i8 },
    ActivatePowerUp { slot: u8 },
    ReleaseBall,
    LeaveRoom,
    Ping { nonce: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome { player_id: String, reconnect_token: String },
    QueueStatus { queued: bool, position: usize },
    RoomList { rooms: Vec<RoomSummary> },
    PlayerList { players: Vec<PlayerSummary> },
    RoomJoined { room: RoomSummary, player_id: String, team: u8, host: bool, invite_code: Option<String> },
    InviteCreated { invite_code: String, room_code: String },
    InviteReceived { from_name: String, invite_code: String, room_code: String },
    PlayerJoined { player: PlayerSummary },
    PlayerLeft { player_id: String, reconnect_grace_secs: u32 },
    PlayerReconnected { player_id: String },
    MatchStarting { snapshot: MatchSnapshot },
    Snapshot { snapshot: MatchSnapshot },
    MatchEnded { winning_team: u8, tournament_champion: Option<String> },
    TournamentUpdate { round: usize, match_index: usize, participants: Vec<String>, champion: Option<String> },
    Pong { nonce: u64 },
    Error { code: String, message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSummary {
    pub player_id: String,
    pub display_name: String,
    pub team: u8,
    pub ready: bool,
    pub connected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomSummary {
    pub room_code: String,
    pub visibility: RoomVisibility,
    pub mode: RoomMode,
    pub host_name: String,
    pub players: Vec<PlayerSummary>,
    pub max_players: usize,
    pub in_progress: bool,
    pub winning_score: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BallSnapshot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub last_hit_team: u8,
    pub primary: bool,
    pub special_kind: u8,
    pub special_owner: u8,
    pub special_hits: u8,
    pub attached: bool,
    pub returning: bool,
    pub throws_left: u8,
    pub offset_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PickupSnapshot {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub kind: u8,
}



#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PowerUpEventSnapshot {
    pub id: u64,
    pub event: u8,
    pub kind: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerUpSlotSnapshot {
    pub kind: u8,
    pub charges: u8,
    pub cooldown_remaining: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectSnapshot {
    pub kind: u8,
    pub team: u8,
    pub remaining: f32,
    pub magnitude: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchSnapshot {
    pub server_tick: u64,
    pub scores: [u32; 2],
    pub winning_score: u32,
    pub paddle_y: [f32; 2],
    pub paddle_height: [f32; 2],
    pub balls: Vec<BallSnapshot>,
    pub pickups: Vec<PickupSnapshot>,
    #[serde(default)]
    pub powerup_events: Vec<PowerUpEventSnapshot>,
    pub inventories: [Vec<PowerUpSlotSnapshot>; 2],
    pub effects: Vec<EffectSnapshot>,
    pub phase: String,
    pub active_team: [String; 2],
}



