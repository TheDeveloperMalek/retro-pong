

use bevy::prelude::Resource;
use crate::persistence::Difficulty;

#[derive(Resource, Default)]
pub struct AiRandom {
    pub state: u64,
}

impl AiRandom {
    pub fn unit(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(2_862_933_555_777_941_757)
            .wrapping_add(3_037_000_493);

        (self.state >> 33) as u32 as f32 / u32::MAX as f32
    }
}

pub fn difficulty_tuning(difficulty: Difficulty) -> (f32, f32, f32) {
    match difficulty {
        Difficulty::Easy => (0.68, 270.0, 125.0),
        Difficulty::Medium => (0.36, 360.0, 66.0),
        Difficulty::Hard => (0.21, 455.0, 30.0),
        Difficulty::Expert => (0.13, 525.0, 12.0),
    }
}
