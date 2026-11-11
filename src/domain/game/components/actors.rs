

use bevy::prelude::*;
use super::{PowerUpKind, Side};

#[derive(Component)]
pub struct Paddle {
    pub side: Side,
    pub height: f32,
    pub base_height: f32,
}

#[derive(Component)]
pub struct AiBrain {
    pub reaction_timer: f32,
    pub target_y: f32,
    pub error_offset: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Ball {
    pub velocity: Vec2,
    pub curve: f32,
    pub radius: f32,
    pub rally_speed: f32,
    pub last_hit_by: Side,
    pub is_primary: bool,
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub age: f32,
    pub frame_index: u8,
}

#[derive(Component)]
pub struct Obstacle {
    pub phase: f32,
    pub width: f32,
    pub height: f32,
}
