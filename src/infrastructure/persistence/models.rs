

use bevy::prelude::{GamepadButton, KeyCode, Resource};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    English,
    German,
    French,
    Swedish,
    Spanish,
    Russian,
}

impl Language {
    pub const ALL: [Self; 6] = [Self::English, Self::German, Self::French, Self::Swedish, Self::Spanish, Self::Russian];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|language| *language == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Self { Self::ALL[index % Self::ALL.len()] }

    pub fn step(self, direction: i32) -> Self {
        Self::from_index((self.index() as i32 + direction).rem_euclid(Self::ALL.len() as i32) as usize)
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "Deutsch",
            Self::French => "Français",
            Self::Swedish => "Svenska",
            Self::Spanish => "Español",
            Self::Russian => "Русский",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    pub const ALL: [Self; 4] = [Self::Easy, Self::Medium, Self::Hard, Self::Expert];

    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Medium => "MEDIUM",
            Self::Hard => "HARD",
            Self::Expert => "EXPERT",
        }
    }

    pub fn from_index(index: usize) -> Self { Self::ALL[index % Self::ALL.len()] }
    pub fn index(self) -> usize { Self::ALL.iter().position(|difficulty| *difficulty == self).unwrap_or(1) }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TimedPowerUpSettings {
    pub enabled: bool,
    pub duration_secs: f32,
    pub magnitude: f32,
    pub spawn_weight: f32,
    
    pub max_charges: u8,
    
    pub cooldown_secs: f32,
}

impl Default for TimedPowerUpSettings {
    fn default() -> Self {
        Self { enabled: true, duration_secs: 8.0, magnitude: 1.0, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 }
    }
}





#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PowerUpItemSettings {
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

impl Default for PowerUpItemSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_secs: 5.0,
            magnitude: 1.0,
            spawn_weight: 0.62,
            max_charges: 1,
            cooldown_secs: 1.0,
            min_projectiles: 3,
            max_projectiles: 5,
            opponent_blocks: 2,
            throws: 2,
        }
    }
}





#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ControlBindings {
    pub player_one_up: String,
    pub player_one_down: String,
    pub player_one_inventory: [String; 5],
    pub player_two_up: String,
    pub player_two_down: String,
    pub player_two_inventory: [String; 5],
    pub player_one_release: String,
    pub player_two_release: String,
    pub pause: String,
}

impl Default for ControlBindings {
    fn default() -> Self {
        Self {
            player_one_up: "ArrowUp".into(),
            player_one_down: "ArrowDown".into(),
            player_one_inventory: ["Digit1", "Digit2", "Digit3", "Digit4", "Digit5"].map(str::to_owned),
            player_two_up: "KeyW".into(),
            player_two_down: "KeyS".into(),
            player_two_inventory: ["KeyJ", "KeyK", "KeyL", "KeyU", "KeyI"].map(str::to_owned),
            player_one_release: "Space".into(),
            player_two_release: "Enter".into(),
            pause: "Escape".into(),
        }
    }
}

impl ControlBindings {
    
    
    pub fn binding(&self, action: usize) -> Option<&str> {
        match action {
            0 => Some(&self.player_one_up),
            1 => Some(&self.player_one_down),
            2..=6 => self.player_one_inventory.get(action - 2).map(String::as_str),
            7 => Some(&self.player_two_up),
            8 => Some(&self.player_two_down),
            9..=13 => self.player_two_inventory.get(action - 9).map(String::as_str),
            14 => Some(&self.player_one_release),
            15 => Some(&self.player_two_release),
            16 => Some(&self.pause),
            _ => None,
        }
    }

    pub fn set_binding(&mut self, action: usize, key: String) -> bool {
        let target = match action {
            0 => &mut self.player_one_up,
            1 => &mut self.player_one_down,
            2..=6 => &mut self.player_one_inventory[action - 2],
            7 => &mut self.player_two_up,
            8 => &mut self.player_two_down,
            9..=13 => &mut self.player_two_inventory[action - 9],
            14 => &mut self.player_one_release,
            15 => &mut self.player_two_release,
            16 => &mut self.pause,
            _ => return false,
        };
        *target = key;
        true
    }

    pub fn conflict(&self, action: usize, key: &str) -> Option<usize> {
        (0..17).find(|candidate| *candidate != action && self.binding(*candidate) == Some(key))
    }

    pub fn action_label(action: usize) -> &'static str {
        match action {
            0 => "P1 MOVE UP", 1 => "P1 MOVE DOWN",
            2..=6 => "P1 POWER-UP SLOT", 7 => "P2 MOVE UP", 8 => "P2 MOVE DOWN",
            9..=13 => "P2 POWER-UP SLOT", 14 => "P1 RELEASE", 15 => "P2 RELEASE",
            16 => "PAUSE", _ => "UNKNOWN ACTION",
        }
    }

    pub fn key_code(&self, action: usize) -> Option<KeyCode> {
        key_code_from_name(self.binding(action)?)
    }
}



pub fn key_code_from_name(name: &str) -> Option<KeyCode> {
    Some(match name {
        "ArrowUp" => KeyCode::ArrowUp, "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft, "ArrowRight" => KeyCode::ArrowRight,
        "Space" => KeyCode::Space, "Enter" => KeyCode::Enter, "Escape" => KeyCode::Escape,
        "Tab" => KeyCode::Tab, "Backspace" => KeyCode::Backspace,
        "Digit0" => KeyCode::Digit0, "Digit1" => KeyCode::Digit1, "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3, "Digit4" => KeyCode::Digit4, "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6, "Digit7" => KeyCode::Digit7, "Digit8" => KeyCode::Digit8, "Digit9" => KeyCode::Digit9,
        "KeyA" => KeyCode::KeyA, "KeyB" => KeyCode::KeyB, "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD, "KeyE" => KeyCode::KeyE, "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG, "KeyH" => KeyCode::KeyH, "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ, "KeyK" => KeyCode::KeyK, "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM, "KeyN" => KeyCode::KeyN, "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP, "KeyQ" => KeyCode::KeyQ, "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS, "KeyT" => KeyCode::KeyT, "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV, "KeyW" => KeyCode::KeyW, "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY, "KeyZ" => KeyCode::KeyZ,
        "F1" => KeyCode::F1, "F2" => KeyCode::F2, "F3" => KeyCode::F3, "F4" => KeyCode::F4,
        "F5" => KeyCode::F5, "F6" => KeyCode::F6, "F7" => KeyCode::F7, "F8" => KeyCode::F8,
        "F9" => KeyCode::F9, "F10" => KeyCode::F10, "F11" => KeyCode::F11, "F12" => KeyCode::F12,
        _ => return None,
    })
}

pub fn key_name(code: KeyCode) -> &'static str {
    match code {
        KeyCode::ArrowUp => "ArrowUp", KeyCode::ArrowDown => "ArrowDown",
        KeyCode::ArrowLeft => "ArrowLeft", KeyCode::ArrowRight => "ArrowRight",
        KeyCode::Space => "Space", KeyCode::Enter => "Enter", KeyCode::Escape => "Escape",
        KeyCode::Tab => "Tab", KeyCode::Backspace => "Backspace",
        KeyCode::Digit0 => "Digit0", KeyCode::Digit1 => "Digit1", KeyCode::Digit2 => "Digit2",
        KeyCode::Digit3 => "Digit3", KeyCode::Digit4 => "Digit4", KeyCode::Digit5 => "Digit5",
        KeyCode::Digit6 => "Digit6", KeyCode::Digit7 => "Digit7", KeyCode::Digit8 => "Digit8", KeyCode::Digit9 => "Digit9",
        KeyCode::KeyA => "KeyA", KeyCode::KeyB => "KeyB", KeyCode::KeyC => "KeyC", KeyCode::KeyD => "KeyD",
        KeyCode::KeyE => "KeyE", KeyCode::KeyF => "KeyF", KeyCode::KeyG => "KeyG", KeyCode::KeyH => "KeyH",
        KeyCode::KeyI => "KeyI", KeyCode::KeyJ => "KeyJ", KeyCode::KeyK => "KeyK", KeyCode::KeyL => "KeyL",
        KeyCode::KeyM => "KeyM", KeyCode::KeyN => "KeyN", KeyCode::KeyO => "KeyO", KeyCode::KeyP => "KeyP",
        KeyCode::KeyQ => "KeyQ", KeyCode::KeyR => "KeyR", KeyCode::KeyS => "KeyS", KeyCode::KeyT => "KeyT",
        KeyCode::KeyU => "KeyU", KeyCode::KeyV => "KeyV", KeyCode::KeyW => "KeyW", KeyCode::KeyX => "KeyX",
        KeyCode::KeyY => "KeyY", KeyCode::KeyZ => "KeyZ",
        KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3", KeyCode::F4 => "F4",
        KeyCode::F5 => "F5", KeyCode::F6 => "F6", KeyCode::F7 => "F7", KeyCode::F8 => "F8",
        KeyCode::F9 => "F9", KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
        _ => "Unsupported",
    }
}






#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GamepadControlBindings {
    pub buttons: [String; 17],
}

impl Default for GamepadControlBindings {
    fn default() -> Self {
        Self {
            buttons: [
                "DPadUp", "DPadDown",
                "West", "North", "East", "LeftTrigger2", "RightTrigger2",
                "DPadUp", "DPadDown",
                "West", "North", "East", "LeftTrigger2", "RightTrigger2",
                "South", "South", "Start",
            ].map(str::to_owned),
        }
    }
}

impl GamepadControlBindings {
    pub fn binding(&self, action: usize) -> Option<&str> {
        self.buttons.get(action).map(String::as_str)
    }

    pub fn set_binding(&mut self, action: usize, button: String) -> bool {
        let Some(target) = self.buttons.get_mut(action) else { return false; };
        *target = button;
        true
    }

    
    
    
    pub fn conflict(&self, action: usize, button: &str) -> Option<usize> {
        let peers: &[usize] = match action {
            0..=6 | 14 => &[0, 1, 2, 3, 4, 5, 6, 14, 16],
            7..=13 | 15 => &[7, 8, 9, 10, 11, 12, 13, 15, 16],
            16 => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            _ => return None,
        };
        peers.iter().copied().find(|candidate| *candidate != action && self.binding(*candidate) == Some(button))
    }

    pub fn button_code(&self, action: usize) -> Option<GamepadButton> {
        gamepad_button_from_name(self.binding(action)?)
    }

    pub fn action_label(action: usize) -> &'static str {
        match action {
            0 => "P1 MOVE UP", 1 => "P1 MOVE DOWN",
            2..=6 => "P1 POWER-UP SLOT", 7 => "P2 MOVE UP", 8 => "P2 MOVE DOWN",
            9..=13 => "P2 POWER-UP SLOT", 14 => "P1 RELEASE", 15 => "P2 RELEASE",
            16 => "PAUSE", _ => "UNKNOWN ACTION",
        }
    }
}

pub fn gamepad_button_from_name(name: &str) -> Option<GamepadButton> {
    Some(match name {
        "South" => GamepadButton::South,
        "East" => GamepadButton::East,
        "North" => GamepadButton::North,
        "West" => GamepadButton::West,
        "C" => GamepadButton::C,
        "Z" => GamepadButton::Z,
        "LeftTrigger" => GamepadButton::LeftTrigger,
        "LeftTrigger2" => GamepadButton::LeftTrigger2,
        "RightTrigger" => GamepadButton::RightTrigger,
        "RightTrigger2" => GamepadButton::RightTrigger2,
        "Select" => GamepadButton::Select,
        "Start" => GamepadButton::Start,
        "Mode" => GamepadButton::Mode,
        "LeftThumb" => GamepadButton::LeftThumb,
        "RightThumb" => GamepadButton::RightThumb,
        "DPadUp" => GamepadButton::DPadUp,
        "DPadDown" => GamepadButton::DPadDown,
        "DPadLeft" => GamepadButton::DPadLeft,
        "DPadRight" => GamepadButton::DPadRight,
        _ => return None,
    })
}

pub fn gamepad_button_name(button: GamepadButton) -> &'static str {
    match button {
        GamepadButton::South => "South",
        GamepadButton::East => "East",
        GamepadButton::North => "North",
        GamepadButton::West => "West",
        GamepadButton::C => "C",
        GamepadButton::Z => "Z",
        GamepadButton::LeftTrigger => "LeftTrigger",
        GamepadButton::LeftTrigger2 => "LeftTrigger2",
        GamepadButton::RightTrigger => "RightTrigger",
        GamepadButton::RightTrigger2 => "RightTrigger2",
        GamepadButton::Select => "Select",
        GamepadButton::Start => "Start",
        GamepadButton::Mode => "Mode",
        GamepadButton::LeftThumb => "LeftThumb",
        GamepadButton::RightThumb => "RightThumb",
        GamepadButton::DPadUp => "DPadUp",
        GamepadButton::DPadDown => "DPadDown",
        GamepadButton::DPadLeft => "DPadLeft",
        GamepadButton::DPadRight => "DPadRight",
        _ => "Unsupported",
    }
}






#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MenuNavigationBindings {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub confirm: String,
    pub cancel: String,
}

impl Default for MenuNavigationBindings {
    fn default() -> Self {
        Self {
            up: "ArrowUp".into(),
            down: "ArrowDown".into(),
            left: "ArrowLeft".into(),
            right: "ArrowRight".into(),
            confirm: "Enter".into(),
            cancel: "Escape".into(),
        }
    }
}

impl MenuNavigationBindings {
    pub fn binding(&self, action: usize) -> Option<&str> {
        match action {
            0 => Some(&self.up),
            1 => Some(&self.down),
            2 => Some(&self.left),
            3 => Some(&self.right),
            4 => Some(&self.confirm),
            5 => Some(&self.cancel),
            _ => None,
        }
    }

    pub fn set_binding(&mut self, action: usize, key: String) -> bool {
        let target = match action {
            0 => &mut self.up,
            1 => &mut self.down,
            2 => &mut self.left,
            3 => &mut self.right,
            4 => &mut self.confirm,
            5 => &mut self.cancel,
            _ => return false,
        };
        *target = key;
        true
    }

    pub fn action_label(action: usize) -> &'static str {
        match action {
            0 => "MENU MOVE UP",
            1 => "MENU MOVE DOWN",
            2 => "MENU MOVE LEFT",
            3 => "MENU MOVE RIGHT",
            4 => "MENU CONFIRM",
            5 => "MENU CANCEL",
            _ => "UNKNOWN ACTION",
        }
    }
}

#[derive(Resource, Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub language: Language,
    pub difficulty: Difficulty,
    pub controls: ControlBindings,
    pub menu_navigation: MenuNavigationBindings,
    pub gamepad_controls: GamepadControlBindings,
    pub winning_score: u32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub powerups_enabled: bool,
    pub special_ball_enabled: bool,
    pub obstacles_enabled: bool,
    pub fullscreen: bool,
    pub reduce_motion: bool,
    
    pub ui_scale: f32,
    pub paddle_skin: u8,
    pub ball_skin: u8,
    pub paddle_shade: i8,
    pub ball_shade: i8,
    
    pub paddle_custom_image: Option<String>,
    pub ball_custom_image: Option<String>,
    pub manual_powerup_activation: bool,
    pub double_ball_enabled: bool,
    pub max_active_balls: u8,
    pub inventory_slots: u8,
    pub powerup_spawn_interval_secs: f32,
    pub double_ball_spawn_probability: f32,
    pub double_ball_max_charges: u8,
    pub double_ball_cooldown_secs: f32,
    pub powerup_event_drops: bool,
    pub speed_powerup: TimedPowerUpSettings,
    pub slow_powerup: TimedPowerUpSettings,
    pub enlarge_powerup: TimedPowerUpSettings,
    pub fireball_powerup: PowerUpItemSettings,
    pub snowball_powerup: PowerUpItemSettings,
    pub reversal_powerup: PowerUpItemSettings,
    pub sticky_ball_powerup: PowerUpItemSettings,
    pub ball_storm_powerup: PowerUpItemSettings,
    pub boomerang_powerup: PowerUpItemSettings,
    pub magnetic_field_powerup: PowerUpItemSettings,
    pub teleport_ball_powerup: PowerUpItemSettings,
    pub energy_shield_powerup: PowerUpItemSettings,
    pub overdrive_powerup: PowerUpItemSettings,
    pub phantom_ball_powerup: PowerUpItemSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::English,
            difficulty: Difficulty::Medium,
            controls: ControlBindings::default(),
            menu_navigation: MenuNavigationBindings::default(),
            gamepad_controls: GamepadControlBindings::default(),
            winning_score: 7,
            music_enabled: true,
            sfx_enabled: true,
            music_volume: 0.32,
            sfx_volume: 0.65,
            powerups_enabled: true,
            special_ball_enabled: true,
            obstacles_enabled: false,
            fullscreen: false,
            reduce_motion: false,
            ui_scale: 1.0,
            paddle_skin: 0,
            ball_skin: 0,
            paddle_shade: 0,
            ball_shade: 0,
            paddle_custom_image: None,
            ball_custom_image: None,
            manual_powerup_activation: false,
            double_ball_enabled: true,
            max_active_balls: 2,
            inventory_slots: 3,
            powerup_spawn_interval_secs: 10.0,
            double_ball_spawn_probability: 0.20,
            double_ball_max_charges: 1,
            double_ball_cooldown_secs: 3.0,
            powerup_event_drops: true,
            speed_powerup: TimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 1.28, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            slow_powerup: TimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 0.72, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            enlarge_powerup: TimedPowerUpSettings { enabled: true, duration_secs: 8.0, magnitude: 1.55, spawn_weight: 1.0, max_charges: 2, cooldown_secs: 2.0 },
            fireball_powerup: PowerUpItemSettings { duration_secs: 5.0, magnitude: 0.78, ..PowerUpItemSettings::default() },
            snowball_powerup: PowerUpItemSettings { duration_secs: 3.0, magnitude: 1.0, ..PowerUpItemSettings::default() },
            reversal_powerup: PowerUpItemSettings { duration_secs: 0.0, magnitude: 1.0, ..PowerUpItemSettings::default() },
            sticky_ball_powerup: PowerUpItemSettings { duration_secs: 0.0, magnitude: 425.0, ..PowerUpItemSettings::default() },
            ball_storm_powerup: PowerUpItemSettings { duration_secs: 0.0, magnitude: 510.0, min_projectiles: 3, max_projectiles: 5, opponent_blocks: 2, ..PowerUpItemSettings::default() },
            boomerang_powerup: PowerUpItemSettings { duration_secs: 0.0, magnitude: 465.0, throws: 2, ..PowerUpItemSettings::default() },
            magnetic_field_powerup: PowerUpItemSettings { duration_secs: 5.0, magnitude: 0.62, ..PowerUpItemSettings::default() },
            teleport_ball_powerup: PowerUpItemSettings { duration_secs: 0.0, magnitude: 1.0, ..PowerUpItemSettings::default() },
            energy_shield_powerup: PowerUpItemSettings { duration_secs: 7.0, magnitude: 1.0, ..PowerUpItemSettings::default() },
            overdrive_powerup: PowerUpItemSettings { duration_secs: 5.0, magnitude: 1.5, ..PowerUpItemSettings::default() },
            phantom_ball_powerup: PowerUpItemSettings { duration_secs: 5.0, magnitude: 0.5, ..PowerUpItemSettings::default() },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SavedMatchType {
    #[default]
    SoloAi,
    LocalVersus,
    Network,
    OpenPlay,
}

impl SavedMatchType {
    pub fn label(self) -> &'static str {
        match self { Self::SoloAi => "SOLO AI", Self::LocalVersus => "LOCAL 2 PLAYER", Self::Network => "NETWORK", Self::OpenPlay => "OPEN PLAY" }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HighScore {
    pub player_score: u32,
    pub opponent_score: u32,
    pub difficulty: Difficulty,
    pub date: String,
    pub won: bool,
    
    pub personal_record: bool,
    pub match_type: SavedMatchType,
}

impl Default for HighScore {
    fn default() -> Self {
        Self { player_score: 0, opponent_score: 0, difficulty: Difficulty::Medium, date: String::new(), won: false, personal_record: false, match_type: SavedMatchType::SoloAi }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Progress {
    pub total_matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub points_scored: u32,
    pub current_win_streak: u32,
    pub best_win_streak: u32,
    pub achievements: Vec<String>,
    pub unlocked_paddle_skins: Vec<u8>,
    pub unlocked_ball_skins: Vec<u8>,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            total_matches: 0,
            wins: 0,
            losses: 0,
            points_scored: 0,
            current_win_streak: 0,
            best_win_streak: 0,
            achievements: Vec::new(),
            unlocked_paddle_skins: (0..13).collect(),
            unlocked_ball_skins: (0..13).collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct NetworkPreferences {
    pub player_name: String,
    pub server_address: String,
}

impl Default for NetworkPreferences {
    fn default() -> Self {
        Self {
            player_name: "PLAYER".into(),
            server_address: "127.0.0.1:41123".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SaveData {
    
    
    #[serde(skip)]
    pub save_allowed: bool,
    pub schema_version: u32,
    pub settings: Settings,
    pub network_preferences: NetworkPreferences,
    pub high_scores: Vec<HighScore>,
    pub progress: Progress,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            save_allowed: true,
            schema_version: 1,
            settings: Settings::default(),
            network_preferences: NetworkPreferences::default(),
            high_scores: Vec::new(),
            progress: Progress::default(),
        }
    }
}


