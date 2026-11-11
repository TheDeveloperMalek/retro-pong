

use super::{gamepad_button_from_name, key_code_from_name, ControlBindings, GamepadControlBindings, Progress, SaveData, Settings, TimedPowerUpSettings};

pub(super) fn normalize_loaded_data(data: &mut SaveData) {
    normalize_settings(&mut data.settings);
    normalize_progress(&mut data.progress);
    normalize_high_scores(&mut data.high_scores);
}

fn normalize_settings(settings: &mut Settings) {
    settings.winning_score = settings.winning_score.clamp(1, 99);
    settings.music_volume = settings.music_volume.clamp(0.0, 1.0);
    settings.sfx_volume = settings.sfx_volume.clamp(0.0, 1.0);
    settings.ui_scale = if settings.ui_scale.is_finite() { settings.ui_scale.clamp(0.80, 1.10) } else { 1.0 };
    
    settings.paddle_skin = settings.paddle_skin.min(12);
    settings.ball_skin = settings.ball_skin.min(12);
    sanitize_skin_asset_path(&mut settings.paddle_custom_image);
    sanitize_skin_asset_path(&mut settings.ball_custom_image);
    settings.paddle_shade = settings.paddle_shade.clamp(-4, 4);
    settings.ball_shade = settings.ball_shade.clamp(-4, 4);
    settings.max_active_balls = settings.max_active_balls.clamp(2, 4);
    settings.inventory_slots = settings.inventory_slots.clamp(1, 5);
    settings.powerup_spawn_interval_secs = settings.powerup_spawn_interval_secs.clamp(5.0, 30.0);
    settings.double_ball_spawn_probability = settings.double_ball_spawn_probability.clamp(0.0, 1.0);

    normalize_timed_power_up(&mut settings.speed_powerup, 1.05..=2.0);
    normalize_timed_power_up(&mut settings.slow_powerup, 0.2..=0.95);
    normalize_timed_power_up(&mut settings.enlarge_powerup, 1.1..=1.8);

    normalize_powerup_item(&mut settings.fireball_powerup, 0.5..=0.95);
    normalize_powerup_item(&mut settings.snowball_powerup, 0.0..=2.0);
    normalize_powerup_item(&mut settings.reversal_powerup, 0.0..=2.0);
    normalize_powerup_item(&mut settings.sticky_ball_powerup, 250.0..=700.0);
    normalize_powerup_item(&mut settings.ball_storm_powerup, 300.0..=750.0);
    normalize_powerup_item(&mut settings.boomerang_powerup, 300.0..=700.0);
    settings.ball_storm_powerup.min_projectiles = settings.ball_storm_powerup.min_projectiles.clamp(3, 5);
    settings.ball_storm_powerup.max_projectiles = settings.ball_storm_powerup.max_projectiles.clamp(settings.ball_storm_powerup.min_projectiles, 5);
    settings.ball_storm_powerup.opponent_blocks = settings.ball_storm_powerup.opponent_blocks.clamp(1, 5);
    settings.boomerang_powerup.throws = settings.boomerang_powerup.throws.clamp(1, 5);

    settings.double_ball_max_charges = settings.double_ball_max_charges.clamp(1, 5);
    settings.double_ball_cooldown_secs = settings.double_ball_cooldown_secs.clamp(0.0, 30.0);
    normalize_controls(&mut settings.controls);
    normalize_menu_navigation(&mut settings.menu_navigation);
    normalize_gamepad_controls(&mut settings.gamepad_controls);
}

fn normalize_timed_power_up(
    config: &mut TimedPowerUpSettings,
    magnitude_range: std::ops::RangeInclusive<f32>,
) {
    config.duration_secs = config.duration_secs.clamp(1.0, 30.0);
    config.magnitude = config
        .magnitude
        .clamp(*magnitude_range.start(), *magnitude_range.end());
    config.spawn_weight = config.spawn_weight.clamp(0.0, 1.0);
    config.max_charges = config.max_charges.clamp(1, 5);
    config.cooldown_secs = config.cooldown_secs.clamp(0.0, 30.0);
}

fn normalize_powerup_item(
    config: &mut super::PowerUpItemSettings,
    magnitude_range: std::ops::RangeInclusive<f32>,
) {
    config.duration_secs = config.duration_secs.clamp(0.0, 30.0);
    config.magnitude = config.magnitude.clamp(*magnitude_range.start(), *magnitude_range.end());
    config.spawn_weight = config.spawn_weight.clamp(0.0, 1.0);
    config.max_charges = config.max_charges.clamp(1, 5);
    config.cooldown_secs = config.cooldown_secs.clamp(0.0, 30.0);
    config.min_projectiles = config.min_projectiles.clamp(3, 5);
    config.max_projectiles = config.max_projectiles.clamp(config.min_projectiles, 5);
    config.opponent_blocks = config.opponent_blocks.clamp(1, 5);
    config.throws = config.throws.clamp(1, 5);
}



fn normalize_gamepad_controls(bindings: &mut GamepadControlBindings) {
    let defaults = GamepadControlBindings::default();
    for action in 0..17 {
        let invalid = bindings.binding(action).is_none_or(|name| gamepad_button_from_name(name).is_none());
        if invalid {
            if let Some(default) = defaults.binding(action) {
                bindings.set_binding(action, default.to_owned());
    }
        }
    }
    for action in 0..17 {
        let button = bindings.binding(action).unwrap_or("Unsupported").to_owned();
        if bindings.conflict(action, &button).is_some() {
            if let Some(default) = defaults.binding(action) {
                bindings.set_binding(action, default.to_owned());
            }
        }
    }
}

fn normalize_menu_navigation(bindings: &mut super::MenuNavigationBindings) {
    let defaults = super::MenuNavigationBindings::default();
    let invalid = (0..6).any(|action| {
        bindings.binding(action).is_none_or(|name| key_code_from_name(name).is_none())
    });
    let duplicate = (0..6).any(|left| {
        ((left + 1)..6).any(|right| {
            bindings.binding(left).is_some()
                && bindings.binding(left) == bindings.binding(right)
        })
    });
    if invalid || duplicate {
        *bindings = defaults;
    }
}

fn normalize_controls(controls: &mut ControlBindings) {
    let defaults = ControlBindings::default();
    for action in 0..17 {
        let invalid = controls.binding(action).is_none_or(|name| key_code_from_name(name).is_none());
        if invalid {
            if let Some(default) = defaults.binding(action) {
                controls.set_binding(action, default.to_owned());
            }
        }
    }
    
    
    for action in 0..17 {
        let key = controls.binding(action).unwrap_or("Unsupported").to_owned();
        if controls.conflict(action, &key).is_some() {
            if let Some(default) = defaults.binding(action) {
                controls.set_binding(action, default.to_owned());
            }
        }
    }
    
    
    for action in 0..17 {
        let key = controls.binding(action).unwrap_or("Unsupported").to_owned();
        if controls.conflict(action, &key).is_some() {
            if let Some(default) = defaults.binding(action) {
                controls.set_binding(action, default.to_owned());
            }
        }
    }
}


fn sanitize_skin_asset_path(path: &mut Option<String>) {
    let Some(value) = path.as_mut() else { return; };
    let normalized = value.replace('\\', "/");
    let allowed_extension = ["png", "jpg", "jpeg", "webp"]
        .iter()
        .any(|extension| normalized.to_ascii_lowercase().ends_with(&format!(".{extension}")));
    if !normalized.starts_with("custom_skins/")
        || normalized.contains("..")
        || normalized.starts_with('/')
        || !allowed_extension
    {
        *path = None;
    } else {
        *value = normalized;
    }
}

fn normalize_progress(progress: &mut Progress) {
    
    
    progress.unlocked_paddle_skins = (0..13).collect();
    progress.unlocked_ball_skins = (0..13).collect();
}

fn normalize_high_scores(scores: &mut Vec<super::HighScore>) {
    scores.sort_by(|left, right| {
        right
            .player_score
            .cmp(&left.player_score)
            .then_with(|| left.opponent_score.cmp(&right.opponent_score))
            .then_with(|| right.date.cmp(&left.date))
    });
    scores.truncate(1000);
}

