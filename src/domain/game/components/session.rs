

use bevy::prelude::*;
use super::powerups::PowerUpKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchPhase {
    Rally,
    PointPause,
    Finished,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Player,
    Ai,
    PlayerTwo,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MatchMode {
    #[default]
    SinglePlayer,
    LocalVersus,
    Networked,
    OpenPlay,
}

#[derive(Clone, Copy, Debug)]
pub struct PowerUpSlot {
    pub kind: PowerUpKind,
    pub charges: u8,
    pub cooldown_remaining: f32,
}

impl PowerUpSlot {
    pub fn new(kind: PowerUpKind, charges: u8) -> Self {
        Self {
            kind,
            charges: charges.max(1),
            cooldown_remaining: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct PowerUpInventory {
    pub player_one: Vec<PowerUpSlot>,
    pub player_two: Vec<PowerUpSlot>,
    pub ai: Vec<PowerUpSlot>,
}



#[derive(Resource, Clone, Copy)]
pub struct OpenPlayDuration {
    pub minutes: u16,
    pub seconds: u8,
}

impl Default for OpenPlayDuration {
    fn default() -> Self { Self { minutes: 3, seconds: 0 } }
}

impl OpenPlayDuration {
    pub fn total_seconds(self) -> f32 { (self.minutes as f32 * 60.0) + self.seconds.min(59) as f32 }
}

#[derive(Resource, Default)]
pub struct InventoryDiscardState {
    pub selected: Option<(Side, usize)>,
    pub held_seconds: f32,
}

#[derive(Resource, Default)]
pub struct PendingPowerUp {
    pub kind: Option<PowerUpKind>,
    pub owner: Side,
}

impl PowerUpInventory {
    pub fn for_side(&self, side: Side) -> &[PowerUpSlot] {
        match side {
            Side::PlayerTwo => &self.player_two,
            Side::Ai => &self.ai,
            Side::Player => &self.player_one,
        }
    }

    pub fn for_side_mut(&mut self, side: Side) -> &mut Vec<PowerUpSlot> {
        match side {
            Side::PlayerTwo => &mut self.player_two,
            Side::Ai => &mut self.ai,
            Side::Player => &mut self.player_one,
        }
    }
}

#[derive(Resource)]
pub struct MatchSession {
    pub player_score: u32,
    pub ai_score: u32,
    pub winning_score: u32,
    pub phase: MatchPhase,
    pub phase_timer: f32,
    pub last_scorer: Option<Side>,
    pub rally_points: u32,
    pub rng_state: u64,
    pub result_saved: bool,
    pub active: bool,
    pub restart_requested: bool,
    pub mode: MatchMode,
    pub open_play_remaining: f32,
    pub open_play_sudden_death: bool,
}

impl Default for MatchSession {
    fn default() -> Self {
        Self {
            player_score: 0,
            ai_score: 0,
            winning_score: 7,
            phase: MatchPhase::Rally,
            phase_timer: 0.0,
            last_scorer: None,
            rally_points: 0,
            rng_state: 0x5EED_CAFE_F00D,
            result_saved: false,
            active: false,
            restart_requested: false,
            mode: MatchMode::SinglePlayer,
            open_play_remaining: 180.0,
            open_play_sudden_death: false,
        }
    }
}

impl MatchSession {
    pub fn random_unit(&mut self) -> f32 {
        crate::random::unit_f32(&mut self.rng_state)
    }
}
