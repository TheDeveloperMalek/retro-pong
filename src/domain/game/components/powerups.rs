

use bevy::prelude::*;
use super::Side;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PowerUpKind {
    Speed,
    Slow,
    Enlarge,
    DoubleBall,
    Fireball,
    Snowball,
    ReversalBall,
    StickyBall,
    BallStorm,
    BoomerangBall,
    MagneticField,
    TeleportBall,
    EnergyShield,
    Overdrive,
    PhantomBall,
}

impl PowerUpKind {
    pub fn requires_manual_activation(self) -> bool {
        matches!(
            self,
            Self::StickyBall | Self::BoomerangBall | Self::MagneticField | Self::TeleportBall
                | Self::EnergyShield | Self::Overdrive | Self::PhantomBall
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Speed => "SPEED BOOST",
            Self::Slow => "SLOW FIELD",
            Self::Enlarge => "BIG PADDLE",
            Self::DoubleBall => "DOUBLE BALL",
            Self::Fireball => "FIREBALL",
            Self::Snowball => "SNOWBALL",
            Self::ReversalBall => "REVERSAL BALL",
            Self::StickyBall => "STICKY BALL",
            Self::BallStorm => "BALL STORM",
            Self::BoomerangBall => "BOOMERANG BALL",
            Self::MagneticField => "MAGNETIC FIELD",
            Self::TeleportBall => "TELEPORT BALL",
            Self::EnergyShield => "ENERGY SHIELD",
            Self::Overdrive => "OVERDRIVE",
            Self::PhantomBall => "PHANTOM BALL",
        }
    }

    
    pub fn pixel_art_key(self) -> &'static str {
        match self {
            Self::Speed => "speed",
            Self::Slow => "slow",
            Self::Enlarge => "enlarge",
            Self::DoubleBall => "double_ball",
            Self::Fireball => "fireball",
            Self::Snowball => "snowball",
            Self::ReversalBall => "reversal",
            Self::StickyBall => "sticky",
            Self::BallStorm => "ball_storm",
            Self::BoomerangBall => "boomerang",
            Self::MagneticField => "magnetic_field",
            Self::TeleportBall => "teleport_ball",
            Self::EnergyShield => "energy_shield",
            Self::Overdrive => "overdrive",
            Self::PhantomBall => "phantom_ball",
        }
    }

    
    pub fn pixel_art_frame_count(self) -> u8 {
        match self {
            Self::Speed | Self::Slow | Self::Enlarge | Self::StickyBall => 2,
            Self::DoubleBall | Self::Snowball | Self::ReversalBall | Self::BoomerangBall => 3,
            Self::Fireball | Self::BallStorm | Self::MagneticField | Self::EnergyShield => 4,
            Self::TeleportBall | Self::Overdrive | Self::PhantomBall => 3,
        }
    }

    
    pub fn pixel_art_frame_duration(self) -> f32 {
        match self {
            Self::Speed => 0.11,
            Self::Slow => 0.26,
            Self::Enlarge => 0.22,
            Self::DoubleBall => 0.17,
            Self::Fireball => 0.095,
            Self::Snowball => 0.21,
            Self::ReversalBall => 0.16,
            Self::StickyBall => 0.24,
            Self::BallStorm => 0.10,
            Self::BoomerangBall => 0.16,
            Self::MagneticField => 0.14,
            Self::TeleportBall => 0.16,
            Self::EnergyShield => 0.20,
            Self::Overdrive => 0.12,
            Self::PhantomBall => 0.18,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Speed => Color::srgb(1.0, 0.31, 0.30),
            Self::Slow => Color::srgb(0.28, 0.76, 1.0),
            Self::Enlarge => Color::srgb(0.45, 1.0, 0.47),
            Self::DoubleBall => Color::srgb(1.0, 0.74, 0.25),
            Self::Fireball => Color::srgb(1.0, 0.24, 0.06),
            Self::Snowball => Color::srgb(0.72, 0.94, 1.0),
            Self::ReversalBall => Color::srgb(0.76, 0.42, 1.0),
            Self::StickyBall => Color::srgb(0.96, 0.40, 0.76),
            Self::BallStorm => Color::srgb(1.0, 0.58, 0.18),
            Self::BoomerangBall => Color::srgb(0.34, 1.0, 0.72),
            Self::MagneticField => Color::srgb(0.45, 0.72, 1.0),
            Self::TeleportBall => Color::srgb(0.72, 0.46, 1.0),
            Self::EnergyShield => Color::srgb(0.30, 0.92, 1.0),
            Self::Overdrive => Color::srgb(1.0, 0.52, 0.22),
            Self::PhantomBall => Color::srgb(0.70, 0.82, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BallBehaviorKind {
    Fireball,
    Sticky,
    Boomerang,
}


#[derive(Component, Clone, Copy, Debug)]
pub struct BallBehavior {
    pub kind: BallBehaviorKind,
    pub owner: Side,
    pub attached: bool,
    pub returning: bool,
    pub remaining_throws: u8,
    pub offset_y: f32,
}


#[derive(Component, Clone, Copy, Debug)]
pub struct StormProjectile {
    pub owner: Side,
    pub remaining_blocks: u8,
}

impl StormProjectile {
    
    pub fn register_paddle_contact(&mut self, defending_side: Side) -> bool {
        if defending_side == self.owner {
            return false;
        }

        self.remaining_blocks = self.remaining_blocks.saturating_sub(1);
        self.remaining_blocks == 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveTimedEffect {
    pub kind: PowerUpKind,
    pub owner: Side,
    pub remaining: f32,
    pub magnitude: f32,
}

#[derive(Resource)]
pub struct MechanicsClock {
    pub powerup_timer: f32,
    pub obstacle_clock: f32,
    
    pub effects: Vec<ActiveTimedEffect>,
    pub speed_multiplier: f32,
    pub last_score_event: u32,
}

impl Default for MechanicsClock {
    fn default() -> Self {
        Self {
            powerup_timer: 0.0,
            obstacle_clock: 0.0,
            effects: Vec::new(),
            speed_multiplier: 1.0,
            last_score_event: 0,
        }
    }
}

impl MechanicsClock {
    pub fn reset(&mut self) {
        self.powerup_timer = 0.0;
        self.obstacle_clock = 0.0;
        self.effects.clear();
        self.speed_multiplier = 1.0;
        self.last_score_event = 0;
    }
}

