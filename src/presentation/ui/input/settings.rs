

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::{
    game::{components::{Ball, MatchMode, Paddle, PowerUpEditorSelection}, systems, Screen},
    persistence::{self, Difficulty, PersistentData, Settings},
    ui::{powerup_editor_field, PowerUpEditorField},
};

pub(super) fn apply_live_display_settings(
    screen: Screen,
    index: usize,
    settings: &Settings,
    windows: &mut Query<&mut Window, With<PrimaryWindow>>,
    paddles: &mut Query<(&mut Sprite, &Paddle), (With<Paddle>, Without<Ball>)>,
    balls: &mut Query<&mut Sprite, (With<Ball>, Without<Paddle>)>,
    asset_server: &AssetServer,
) {
    if screen == Screen::Settings && index == 7 {
        for mut window in windows.iter_mut() {
            window.mode = if settings.fullscreen {
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
                WindowMode::Windowed
            };
        }
    }
    if screen == Screen::Skins && index <= 2 {
        systems::refresh_paddle_skin(paddles, settings, asset_server);
        for mut sprite in balls.iter_mut() {
            *sprite = systems::ball_skin_sprite(settings, asset_server);
        }
    }
}


pub(super) fn menu_count(screen: Screen, inventory_slots: u8, match_mode: MatchMode) -> usize {
    match screen {
        Screen::MainMenu => 16,
        Screen::NetworkLobby => 1,
        Screen::NetworkGuide => 9,
        Screen::Difficulty => 5,
        Screen::ScoreLimit => 6,
        Screen::OpenPlaySetup => 4,
        Screen::ArenaSelect => 2,
        Screen::Controls => 16 + inventory_slots.clamp(1, 5) as usize * 2,
        Screen::GamepadControls => 19,
        Screen::HighScores => 6,
        Screen::HighScoresRefresh | Screen::Progress => 1,
        Screen::Credits => 4,
        Screen::DeveloperDocument => 1,
        Screen::Settings => 14,
        Screen::Skins => 7,
        Screen::PowerUpSettings => 9,
        Screen::PowerUpEditor => 27,
        Screen::Paused => if match_mode == MatchMode::Networked { 4 } else { 6 },
        Screen::QuickSettings => 6,
        Screen::RestartConfirm => 2,
        Screen::Inventory => 12,
        Screen::Results => 2,
        Screen::Playing => 1,
    }
}


pub(super) fn adjust_selected(
    screen: Screen,
    index: usize,
    direction: i32,
    settings: &mut Settings,
    _persistent: &PersistentData,
    powerup_editor: &mut PowerUpEditorSelection,
    open_play_duration: &mut crate::game::components::OpenPlayDuration,
    arena: &mut crate::game::components::ArenaState,
) {
    match screen {
        Screen::MainMenu if index == 14 => settings.language = settings.language.step(direction),
        Screen::OpenPlaySetup if index == 0 => {
            open_play_duration.minutes = (open_play_duration.minutes as i32 + direction).clamp(0, 99) as u16;
        }
        Screen::OpenPlaySetup if index == 1 => {
            open_play_duration.seconds = (open_play_duration.seconds as i32 + direction).rem_euclid(60) as u8;
        }
        Screen::ArenaSelect if index == 0 => {
            arena.selected = (arena.selected as i32 + direction).rem_euclid(crate::game::components::ArenaState::COUNT as i32) as usize;
        }
        Screen::Difficulty if index < 4 => {
            settings.difficulty = Difficulty::from_index((settings.difficulty.index() as i32 + direction).rem_euclid(4) as usize);
        }
        Screen::ScoreLimit if index == 4 => {
            settings.winning_score = (settings.winning_score as i32 + direction).clamp(1, 99) as u32;
        }
        Screen::Settings => match index {
            0 => settings.music_enabled = !settings.music_enabled,
            1 => settings.sfx_enabled = !settings.sfx_enabled,
            2 => settings.music_volume = (settings.music_volume + direction as f32 * 0.05).clamp(0.0, 1.0),
            3 => settings.sfx_volume = (settings.sfx_volume + direction as f32 * 0.05).clamp(0.0, 1.0),
            4 => settings.powerups_enabled = !settings.powerups_enabled,
            5 => settings.special_ball_enabled = !settings.special_ball_enabled,
            6 => settings.obstacles_enabled = !settings.obstacles_enabled,
            7 => settings.fullscreen = !settings.fullscreen,
            8 => settings.reduce_motion = !settings.reduce_motion,
            9 => settings.ui_scale = (settings.ui_scale + direction as f32 * 0.01).clamp(0.80, 1.10),
            _ => {}
        },
        Screen::PowerUpSettings => match index {
            0 => settings.manual_powerup_activation = !settings.manual_powerup_activation,
            1 => settings.double_ball_enabled = !settings.double_ball_enabled,
            2 => settings.max_active_balls = (settings.max_active_balls as i32 + direction).clamp(2, 4) as u8,
            3 => settings.inventory_slots = (settings.inventory_slots as i32 + direction).clamp(1, 5) as u8,
            4 => settings.powerup_spawn_interval_secs = (settings.powerup_spawn_interval_secs + direction as f32).clamp(5.0, 30.0),
            5 => settings.double_ball_spawn_probability = (settings.double_ball_spawn_probability + direction as f32 * 0.05).clamp(0.0, 1.0),
            6 => settings.powerup_event_drops = !settings.powerup_event_drops,
            _ => {}
        },
        Screen::PowerUpEditor => {
            let selected = powerup_editor.0.min(14);
            if index == 0 {
                powerup_editor.0 = (selected as i32 + direction).rem_euclid(15) as usize;
            } else if index == 1 {
                toggle_powerup_enabled(settings, selected);
            } else {
                adjust_powerup_editor_field(settings, selected, index, direction);
            }
        },
        _ => {}
    }
}



fn toggle_powerup_enabled(settings: &mut Settings, selected: usize) {
    match selected.min(14) {
        0 => settings.speed_powerup.enabled = !settings.speed_powerup.enabled,
        1 => settings.slow_powerup.enabled = !settings.slow_powerup.enabled,
        2 => settings.enlarge_powerup.enabled = !settings.enlarge_powerup.enabled,
        3 => settings.double_ball_enabled = !settings.double_ball_enabled,
        4 => settings.fireball_powerup.enabled = !settings.fireball_powerup.enabled,
        5 => settings.snowball_powerup.enabled = !settings.snowball_powerup.enabled,
        6 => settings.reversal_powerup.enabled = !settings.reversal_powerup.enabled,
        7 => settings.sticky_ball_powerup.enabled = !settings.sticky_ball_powerup.enabled,
        8 => settings.ball_storm_powerup.enabled = !settings.ball_storm_powerup.enabled,
        9 => settings.boomerang_powerup.enabled = !settings.boomerang_powerup.enabled,
        10 => settings.magnetic_field_powerup.enabled = !settings.magnetic_field_powerup.enabled,
        11 => settings.teleport_ball_powerup.enabled = !settings.teleport_ball_powerup.enabled,
        12 => settings.energy_shield_powerup.enabled = !settings.energy_shield_powerup.enabled,
        13 => settings.overdrive_powerup.enabled = !settings.overdrive_powerup.enabled,
        _ => settings.phantom_ball_powerup.enabled = !settings.phantom_ball_powerup.enabled,
    }
}

fn adjust_powerup_editor_field(settings: &mut Settings, selected: usize, row: usize, direction: i32) {
    use PowerUpEditorField::*;

    let step = direction as f32;
    match powerup_editor_field(selected, row) {
        Some(Duration) => match selected {
            0 => settings.speed_powerup.duration_secs = (settings.speed_powerup.duration_secs + step * 0.5).clamp(1.0, 30.0),
            1 => settings.slow_powerup.duration_secs = (settings.slow_powerup.duration_secs + step * 0.5).clamp(1.0, 30.0),
            2 => settings.enlarge_powerup.duration_secs = (settings.enlarge_powerup.duration_secs + step * 0.5).clamp(1.0, 30.0),
            4 => settings.fireball_powerup.duration_secs = (settings.fireball_powerup.duration_secs + step * 0.5).clamp(1.0, 15.0),
            5 => settings.snowball_powerup.duration_secs = (settings.snowball_powerup.duration_secs + step * 0.5).clamp(1.0, 15.0),
            10 => settings.magnetic_field_powerup.duration_secs = (settings.magnetic_field_powerup.duration_secs + step * 0.5).clamp(1.0, 10.0),
            12 => settings.energy_shield_powerup.duration_secs = (settings.energy_shield_powerup.duration_secs + step * 0.5).clamp(1.0, 12.0),
            13 => settings.overdrive_powerup.duration_secs = (settings.overdrive_powerup.duration_secs + step * 0.5).clamp(1.0, 10.0),
            14 => settings.phantom_ball_powerup.duration_secs = (settings.phantom_ball_powerup.duration_secs + step * 0.5).clamp(1.0, 10.0),
            _ => {},
        },
        Some(Magnitude) => match selected {
            0 => settings.speed_powerup.magnitude = (settings.speed_powerup.magnitude + step * 0.05).clamp(1.05, 2.0),
            1 => settings.slow_powerup.magnitude = (settings.slow_powerup.magnitude + step * 0.05).clamp(0.2, 0.95),
            2 => settings.enlarge_powerup.magnitude = (settings.enlarge_powerup.magnitude + step * 0.05).clamp(1.1, 1.8),
            4 => settings.fireball_powerup.magnitude = (settings.fireball_powerup.magnitude + step * 0.02).clamp(0.5, 0.95),
            7 => settings.sticky_ball_powerup.magnitude = (settings.sticky_ball_powerup.magnitude + step * 10.0).clamp(250.0, 700.0),
            8 => settings.ball_storm_powerup.magnitude = (settings.ball_storm_powerup.magnitude + step * 10.0).clamp(300.0, 750.0),
            9 => settings.boomerang_powerup.magnitude = (settings.boomerang_powerup.magnitude + step * 10.0).clamp(300.0, 700.0),
            10 => settings.magnetic_field_powerup.magnitude = (settings.magnetic_field_powerup.magnitude + step * 0.05).clamp(0.1, 1.0),
            13 => settings.overdrive_powerup.magnitude = (settings.overdrive_powerup.magnitude + step * 0.05).clamp(1.1, 2.0),
            14 => settings.phantom_ball_powerup.magnitude = (settings.phantom_ball_powerup.magnitude + step * 0.05).clamp(0.2, 0.8),
            _ => {},
        },
        Some(SpawnWeight) => {
            let delta = step * 0.05;
            match selected {
                0 => settings.speed_powerup.spawn_weight = (settings.speed_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                1 => settings.slow_powerup.spawn_weight = (settings.slow_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                2 => settings.enlarge_powerup.spawn_weight = (settings.enlarge_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                3 => settings.double_ball_spawn_probability = (settings.double_ball_spawn_probability + delta).clamp(0.0, 1.0),
                4 => settings.fireball_powerup.spawn_weight = (settings.fireball_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                5 => settings.snowball_powerup.spawn_weight = (settings.snowball_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                6 => settings.reversal_powerup.spawn_weight = (settings.reversal_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                7 => settings.sticky_ball_powerup.spawn_weight = (settings.sticky_ball_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                8 => settings.ball_storm_powerup.spawn_weight = (settings.ball_storm_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                9 => settings.boomerang_powerup.spawn_weight = (settings.boomerang_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                10 => settings.magnetic_field_powerup.spawn_weight = (settings.magnetic_field_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                11 => settings.teleport_ball_powerup.spawn_weight = (settings.teleport_ball_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                12 => settings.energy_shield_powerup.spawn_weight = (settings.energy_shield_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                13 => settings.overdrive_powerup.spawn_weight = (settings.overdrive_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                14 => settings.phantom_ball_powerup.spawn_weight = (settings.phantom_ball_powerup.spawn_weight + delta).clamp(0.0, 1.0),
                _ => {},
            }
        }
        Some(MaxCharges) => match selected {
            0 => settings.speed_powerup.max_charges = (settings.speed_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            1 => settings.slow_powerup.max_charges = (settings.slow_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            2 => settings.enlarge_powerup.max_charges = (settings.enlarge_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            3 => settings.double_ball_max_charges = (settings.double_ball_max_charges as i32 + direction).clamp(1, 5) as u8,
            4 => settings.fireball_powerup.max_charges = (settings.fireball_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            5 => settings.snowball_powerup.max_charges = (settings.snowball_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            6 => settings.reversal_powerup.max_charges = (settings.reversal_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            7 => settings.sticky_ball_powerup.max_charges = (settings.sticky_ball_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            8 => settings.ball_storm_powerup.max_charges = (settings.ball_storm_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            9 => settings.boomerang_powerup.max_charges = (settings.boomerang_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            10 => settings.magnetic_field_powerup.max_charges = (settings.magnetic_field_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            11 => settings.teleport_ball_powerup.max_charges = (settings.teleport_ball_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            12 => settings.energy_shield_powerup.max_charges = (settings.energy_shield_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            13 => settings.overdrive_powerup.max_charges = (settings.overdrive_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            14 => settings.phantom_ball_powerup.max_charges = (settings.phantom_ball_powerup.max_charges as i32 + direction).clamp(1, 5) as u8,
            _ => {},
        },
        Some(Cooldown) => {
            let delta = step * 0.5;
            match selected {
                0 => settings.speed_powerup.cooldown_secs = (settings.speed_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                1 => settings.slow_powerup.cooldown_secs = (settings.slow_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                2 => settings.enlarge_powerup.cooldown_secs = (settings.enlarge_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                3 => settings.double_ball_cooldown_secs = (settings.double_ball_cooldown_secs + delta).clamp(0.0, 30.0),
                4 => settings.fireball_powerup.cooldown_secs = (settings.fireball_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                5 => settings.snowball_powerup.cooldown_secs = (settings.snowball_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                6 => settings.reversal_powerup.cooldown_secs = (settings.reversal_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                7 => settings.sticky_ball_powerup.cooldown_secs = (settings.sticky_ball_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                8 => settings.ball_storm_powerup.cooldown_secs = (settings.ball_storm_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                9 => settings.boomerang_powerup.cooldown_secs = (settings.boomerang_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                10 => settings.magnetic_field_powerup.cooldown_secs = (settings.magnetic_field_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                11 => settings.teleport_ball_powerup.cooldown_secs = (settings.teleport_ball_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                12 => settings.energy_shield_powerup.cooldown_secs = (settings.energy_shield_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                13 => settings.overdrive_powerup.cooldown_secs = (settings.overdrive_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                14 => settings.phantom_ball_powerup.cooldown_secs = (settings.phantom_ball_powerup.cooldown_secs + delta).clamp(0.0, 30.0),
                _ => {},
            }
        }
        Some(MaxActiveBalls) => settings.max_active_balls = (settings.max_active_balls as i32 + direction).clamp(2, 4) as u8,
        Some(MinProjectiles) => settings.ball_storm_powerup.min_projectiles =
            (settings.ball_storm_powerup.min_projectiles as i32 + direction)
                .clamp(3, settings.ball_storm_powerup.max_projectiles as i32)
                .clamp(3, 5) as u8,
        Some(MaxProjectiles) => settings.ball_storm_powerup.max_projectiles =
            (settings.ball_storm_powerup.max_projectiles as i32 + direction)
                .clamp(settings.ball_storm_powerup.min_projectiles as i32, 5) as u8,
        Some(OpponentBlocks) => settings.ball_storm_powerup.opponent_blocks =
            (settings.ball_storm_powerup.opponent_blocks as i32 + direction).clamp(1, 5) as u8,
        Some(Throws) => settings.boomerang_powerup.throws =
            (settings.boomerang_powerup.throws as i32 + direction).clamp(1, 5) as u8,
        None => {},
    }
}

pub(super) fn reset_powerup_item_settings(settings: &mut Settings, selected: usize) {
    let defaults = Settings::default();
    match selected.min(14) {
        0 => settings.speed_powerup = defaults.speed_powerup,
        1 => settings.slow_powerup = defaults.slow_powerup,
        2 => settings.enlarge_powerup = defaults.enlarge_powerup,
        3 => {
            settings.double_ball_enabled = defaults.double_ball_enabled;
            settings.max_active_balls = defaults.max_active_balls;
            settings.double_ball_spawn_probability = defaults.double_ball_spawn_probability;
            settings.double_ball_max_charges = defaults.double_ball_max_charges;
            settings.double_ball_cooldown_secs = defaults.double_ball_cooldown_secs;
        }
        4 => settings.fireball_powerup = defaults.fireball_powerup,
        5 => settings.snowball_powerup = defaults.snowball_powerup,
        6 => settings.reversal_powerup = defaults.reversal_powerup,
        7 => settings.sticky_ball_powerup = defaults.sticky_ball_powerup,
        8 => settings.ball_storm_powerup = defaults.ball_storm_powerup,
        9 => settings.boomerang_powerup = defaults.boomerang_powerup,
        10 => settings.magnetic_field_powerup = defaults.magnetic_field_powerup,
        11 => settings.teleport_ball_powerup = defaults.teleport_ball_powerup,
        12 => settings.energy_shield_powerup = defaults.energy_shield_powerup,
        13 => settings.overdrive_powerup = defaults.overdrive_powerup,
        _ => settings.phantom_ball_powerup = defaults.phantom_ball_powerup,
    }
}

pub(super) fn persist_settings(settings: &Settings, persistent: &mut PersistentData) {
    persistent.0.settings = settings.clone();
    persistence::save(&persistent.0);
}





