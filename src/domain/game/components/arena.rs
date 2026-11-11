use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug)]
pub struct ArenaState {
    pub selected: usize,
    pub active: usize,
    pub ball_speed_multiplier: f32,
    pub paddle_speed_multiplier: f32,
}

impl Default for ArenaState {
    fn default() -> Self { Self { selected: 0, active: 0, ball_speed_multiplier: 1.0, paddle_speed_multiplier: 1.0 } }
}

impl ArenaState {
    pub const RANDOM_INDEX: usize = 20;
    pub const COUNT: usize = 21;

    pub fn name(index: usize) -> &'static str {
        const NAMES: [&str; 21] = [
            "NEON GRID", "RETRO ARCADE", "CYBER CITY", "SYNTHWAVE SUNSET", "DEEP SPACE",
            "LUNAR BASE", "MARS COLONY", "UNDERWATER DOME", "ICE CAVERN", "VOLCANO CORE",
            "JUNGLE RUINS", "ANCIENT TEMPLE", "DIGITAL CIRCUIT", "HOLOGRAPHIC ARENA",
            "INDUSTRIAL FACTORY", "DESERT DUNES", "SKY STADIUM", "CRYSTAL CAVE",
            "STORM NEXUS", "COSMIC RIFT", "RANDOM ARENA",
        ];
        NAMES[index.min(Self::COUNT - 1)]
    }

    pub fn palette(index: usize) -> (Color, Color, Color) {
        let (background, accent, secondary) = match index.min(19) {
            0 => ([0.025,0.04,0.10],[0.10,0.95,0.88],[0.15,0.34,0.68]),
            1 => ([0.10,0.045,0.10],[1.0,0.69,0.25],[0.60,0.25,0.42]),
            2 => ([0.035,0.035,0.13],[0.40,0.56,1.0],[0.88,0.24,0.62]),
            3 => ([0.13,0.035,0.13],[1.0,0.42,0.68],[0.45,0.30,0.90]),
            4 => ([0.008,0.015,0.07],[0.43,0.78,1.0],[0.28,0.35,0.82]),
            5 => ([0.035,0.07,0.13],[0.40,0.85,1.0],[0.38,0.58,0.82]),
            6 => ([0.12,0.035,0.02],[1.0,0.45,0.22],[0.72,0.25,0.12]),
            7 => ([0.01,0.10,0.16],[0.25,0.90,0.87],[0.15,0.48,0.68]),
            8 => ([0.055,0.11,0.17],[0.74,0.94,1.0],[0.35,0.63,0.86]),
            9 => ([0.14,0.025,0.015],[1.0,0.32,0.12],[0.88,0.58,0.18]),
            10 => ([0.02,0.09,0.055],[0.35,0.92,0.48],[0.24,0.58,0.29]),
            11 => ([0.12,0.075,0.035],[1.0,0.79,0.35],[0.62,0.39,0.16]),
            12 => ([0.025,0.07,0.10],[0.3,0.92,1.0],[0.19,0.57,0.73]),
            13 => ([0.06,0.035,0.13],[0.76,0.55,1.0],[0.35,0.28,0.77]),
            14 => ([0.07,0.07,0.09],[0.92,0.69,0.36],[0.42,0.44,0.50]),
            15 => ([0.11,0.075,0.03],[0.98,0.77,0.38],[0.66,0.43,0.18]),
            16 => ([0.035,0.11,0.17],[0.52,0.88,1.0],[0.35,0.62,0.89]),
            17 => ([0.07,0.04,0.14],[0.72,0.58,1.0],[0.44,0.35,0.76]),
            18 => ([0.045,0.07,0.12],[0.35,0.91,1.0],[0.65,0.43,1.0]),
            _ => ([0.035,0.025,0.11],[0.84,0.55,1.0],[0.36,0.24,0.76]),
        };
        (Color::srgb(background[0],background[1],background[2]), Color::srgb(accent[0],accent[1],accent[2]), Color::srgb(secondary[0],secondary[1],secondary[2]))
    }

    pub fn activate(&mut self, random_value: f32) {
        self.active = if self.selected == Self::RANDOM_INDEX { ((random_value.clamp(0.0, 0.999_999) * 20.0).floor() as usize).min(19) } else { self.selected.min(19) };
        let (ball, paddle) = match self.active {
            0|12|18 => (1.035,0.985), 1|14 => (0.985,1.025), 2|13|19 => (1.02,1.0),
            3|17 => (1.0,1.025), 4|5|8 => (0.975,1.0), 6|9 => (1.025,0.975),
            7|10|11|15|16 => (1.0,0.985), _ => (1.0,1.0),
        };
        self.ball_speed_multiplier = ball;
        self.paddle_speed_multiplier = paddle;
    }
}
