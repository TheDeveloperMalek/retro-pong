

use bevy::prelude::*;
use crate::{
    audio::{self, Sfx},
    game::components::*,
    persistence::Settings,
};
use super::spawning::apply_powerup;



pub fn powerup_max_charges(settings: &Settings, kind: PowerUpKind) -> u8 {
    let configured_charges = match kind {
        PowerUpKind::Speed => settings.speed_powerup.max_charges,
        PowerUpKind::Slow => settings.slow_powerup.max_charges,
        PowerUpKind::Enlarge => settings.enlarge_powerup.max_charges,
        PowerUpKind::DoubleBall => settings.double_ball_max_charges,
        PowerUpKind::Fireball => settings.fireball_powerup.max_charges,
        PowerUpKind::Snowball => settings.snowball_powerup.max_charges,
        PowerUpKind::ReversalBall => settings.reversal_powerup.max_charges,
        PowerUpKind::StickyBall => settings.sticky_ball_powerup.max_charges,
        PowerUpKind::BallStorm => settings.ball_storm_powerup.max_charges,
        PowerUpKind::BoomerangBall => settings.boomerang_powerup.max_charges,
        PowerUpKind::MagneticField => settings.magnetic_field_powerup.max_charges,
        PowerUpKind::TeleportBall => settings.teleport_ball_powerup.max_charges,
        PowerUpKind::EnergyShield => settings.energy_shield_powerup.max_charges,
        PowerUpKind::Overdrive => settings.overdrive_powerup.max_charges,
        PowerUpKind::PhantomBall => settings.phantom_ball_powerup.max_charges,
    };

    configured_charges.clamp(1, 5)
}

fn powerup_cooldown_secs(settings: &Settings, kind: PowerUpKind) -> f32 {
    let configured_cooldown = match kind {
        PowerUpKind::Speed => settings.speed_powerup.cooldown_secs,
        PowerUpKind::Slow => settings.slow_powerup.cooldown_secs,
        PowerUpKind::Enlarge => settings.enlarge_powerup.cooldown_secs,
        PowerUpKind::DoubleBall => settings.double_ball_cooldown_secs,
        PowerUpKind::Fireball => settings.fireball_powerup.cooldown_secs,
        PowerUpKind::Snowball => settings.snowball_powerup.cooldown_secs,
        PowerUpKind::ReversalBall => settings.reversal_powerup.cooldown_secs,
        PowerUpKind::StickyBall => settings.sticky_ball_powerup.cooldown_secs,
        PowerUpKind::BallStorm => settings.ball_storm_powerup.cooldown_secs,
        PowerUpKind::BoomerangBall => settings.boomerang_powerup.cooldown_secs,
        PowerUpKind::MagneticField => settings.magnetic_field_powerup.cooldown_secs,
        PowerUpKind::TeleportBall => settings.teleport_ball_powerup.cooldown_secs,
        PowerUpKind::EnergyShield => settings.energy_shield_powerup.cooldown_secs,
        PowerUpKind::Overdrive => settings.overdrive_powerup.cooldown_secs,
        PowerUpKind::PhantomBall => settings.phantom_ball_powerup.cooldown_secs,
    };

    configured_cooldown.clamp(0.0, 30.0)
}

fn requested_keyboard_activation(input: &ButtonInput<KeyCode>, settings: &Settings) -> Option<(Side, usize)> {
    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    for (side, first_action) in [(Side::Player, 2usize), (Side::PlayerTwo, 9usize)] {
        for slot in 0..capacity {
            if settings.controls.key_code(first_action + slot)
                .is_some_and(|key| input.just_pressed(key))
            {
                return Some((side, slot));
            }
        }
    }
    None
}

fn requested_activation(
    input: &ButtonInput<KeyCode>,
    gamepads: &Query<(Entity, &Gamepad)>,
    settings: &Settings,
) -> Option<(Side, usize)> {
    if let Some(keyboard_activation) = requested_keyboard_activation(input, settings) {
        return Some(keyboard_activation);
    }

    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());
    for (side, controller_index, first_action) in [
        (Side::Player, 0usize, 2usize),
        (Side::PlayerTwo, 1usize, 9usize),
    ] {
        for slot in 0..capacity {
            if controllers.get(controller_index)
                .is_some_and(|(_, gamepad)| settings.gamepad_controls.button_code(first_action + slot).is_some_and(|button| gamepad.just_pressed(button)))
            {
                return Some((side, slot));
            }
        }
    }
    None
}

pub fn activate_inventory_items(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    settings: Res<Settings>,
    mut session: ResMut<MatchSession>,
    asset_server: Res<AssetServer>,
    mut clock: ResMut<MechanicsClock>,
    mut inventory: ResMut<PowerUpInventory>,
    mut paddles: Query<
        (&mut Transform, &mut Paddle, &mut Sprite),
        (With<Paddle>, Without<Ball>),
    >,
    mut balls: Query<
        (Entity, &mut Transform, &mut Ball),
        (With<Ball>, Without<Paddle>, Without<Obstacle>),
    >,
) {
    if session.mode == MatchMode::Networked
        || !settings.powerups_enabled
        || session.phase != MatchPhase::Rally
    {
        return;
    }

    let has_manual_item = inventory
        .player_one
        .iter()
        .chain(inventory.player_two.iter())
        .any(|slot| slot.kind.requires_manual_activation());

    if session.mode != MatchMode::LocalVersus
        && !settings.manual_powerup_activation
        && !has_manual_item
    {
        return;
    }

    let Some((owner, slot_index)) = requested_activation(&input, &gamepads, &settings) else {
        return;
    };

    if owner == Side::PlayerTwo && session.mode != MatchMode::LocalVersus {
        return;
    }

    let slots = inventory.for_side_mut(owner);
    let Some(kind) = slots.get(slot_index).map(|slot| slot.kind) else {
        return;
    };
    let is_enabled = match kind {
        PowerUpKind::Speed => settings.speed_powerup.enabled,
        PowerUpKind::Slow => settings.slow_powerup.enabled,
        PowerUpKind::Enlarge => settings.enlarge_powerup.enabled,
        PowerUpKind::DoubleBall => settings.double_ball_enabled,
        PowerUpKind::Fireball => settings.fireball_powerup.enabled,
        PowerUpKind::Snowball => settings.snowball_powerup.enabled,
        PowerUpKind::ReversalBall => settings.reversal_powerup.enabled,
        PowerUpKind::StickyBall => settings.sticky_ball_powerup.enabled,
        PowerUpKind::BallStorm => settings.ball_storm_powerup.enabled,
        PowerUpKind::BoomerangBall => settings.boomerang_powerup.enabled,
        PowerUpKind::MagneticField => settings.magnetic_field_powerup.enabled,
        PowerUpKind::TeleportBall => settings.teleport_ball_powerup.enabled,
        PowerUpKind::EnergyShield => settings.energy_shield_powerup.enabled,
        PowerUpKind::Overdrive => settings.overdrive_powerup.enabled,
        PowerUpKind::PhantomBall => settings.phantom_ball_powerup.enabled,
    };
    if !is_enabled { return; }

    let should_remove_slot = {
        let Some(slot) = slots.get_mut(slot_index) else {
            return;
        };

        if slot.cooldown_remaining > 0.0 {
            return;
        }

        let active_ball_limit = settings.max_active_balls.clamp(2, 4) as usize;
        if kind == PowerUpKind::DoubleBall && balls.iter_mut().count() >= active_ball_limit {
            return;
        }

        let cooldown = powerup_cooldown_secs(&settings, kind);
        if slot.charges > 1 {
            slot.charges -= 1;
            slot.cooldown_remaining = cooldown;
            false
        } else {
            true
        }
    };

    if should_remove_slot {
        slots.remove(slot_index);
    }

    apply_powerup(
        kind,
        owner,
        None,
        &mut commands,
        &settings,
        &mut clock,
        &mut session,
        &mut paddles,
        &mut balls,
    );
    audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUp);
}


pub fn activate_ai_inventory_items(
    mut commands: Commands,
    settings: Res<Settings>,
    mut session: ResMut<MatchSession>,
    asset_server: Res<AssetServer>,
    mut clock: ResMut<MechanicsClock>,
    mut inventory: ResMut<PowerUpInventory>,
    mut paddles: Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
    mut balls: Query<(Entity, &mut Transform, &mut Ball), (With<Ball>, Without<Paddle>, Without<Obstacle>)>,
) {
    if !settings.powerups_enabled || session.phase != MatchPhase::Rally
        || !matches!(session.mode, MatchMode::SinglePlayer | MatchMode::OpenPlay) { return; }

    let mut incoming: Option<(f32, f32, f32)> = None;
    for (_, transform, ball) in balls.iter_mut() {
        if ball.velocity.x <= 0.0 { continue; }
        let travel = ((AI_X - transform.translation.x) / ball.velocity.x.max(1.0)).max(0.0);
        if incoming.is_none_or(|current| travel < current.0) {
            incoming = Some((travel, transform.translation.y + ball.velocity.y * travel, ball.velocity.length()));
        }
    }
    let threat = incoming.map(|(travel, projected_y, speed)| {
        let ai_y = paddles.iter().find(|(_, paddle, _)| paddle.side == Side::Ai).map(|(transform, _, _)| transform.translation.y).unwrap_or(0.0);
        let urgency = (1.0 - travel / 1.8).clamp(0.0, 1.0);
        (urgency, (projected_y - ai_y).abs(), speed)
    }).unwrap_or((0.0, FIELD_HALF_H, BALL_START_SPEED));
    let trailing = session.ai_score < session.player_score;
    let difficulty_gate = match settings.difficulty {
        crate::persistence::Difficulty::Easy => 0.82,
        crate::persistence::Difficulty::Medium => 0.58,
        crate::persistence::Difficulty::Hard => 0.34,
        crate::persistence::Difficulty::Expert => 0.16,
    };

    let mut choice: Option<(usize, f32)> = None;
    for (index, slot) in inventory.ai.iter().enumerate() {
        if slot.cooldown_remaining > 0.0 { continue; }
        let score = match slot.kind {
            PowerUpKind::MagneticField => 0.58 + threat.0 * 0.34,
            PowerUpKind::TeleportBall => 0.45 + threat.0 * 0.42,
            PowerUpKind::EnergyShield => 0.30 + (if threat.0 > 0.7 { 0.62 } else { 0.0 }) + (if trailing { 0.08 } else { 0.0 }),
            PowerUpKind::Overdrive => 0.42 + (if threat.1 > 38.0 { 0.36 } else { 0.12 }) + threat.0 * 0.15,
            PowerUpKind::PhantomBall => 0.52 + threat.0 * 0.18,
            PowerUpKind::Speed => 0.42 + if trailing { 0.28 } else { 0.10 },
            PowerUpKind::Slow | PowerUpKind::Snowball => 0.55 + if trailing { 0.18 } else { 0.0 },
            PowerUpKind::Enlarge => 0.48 + threat.0 * 0.30,
            PowerUpKind::DoubleBall | PowerUpKind::Fireball | PowerUpKind::BallStorm | PowerUpKind::BoomerangBall => 0.66 + if trailing { 0.20 } else { 0.0 },
            PowerUpKind::ReversalBall => if threat.0 > 0.45 { 0.82 } else { 0.30 },
            PowerUpKind::StickyBall => 0.35,
        };
        if choice.is_none_or(|(_, best)| score > best) { choice = Some((index, score)); }
    }
    let Some((index, quality)) = choice else { return; };
    if quality < difficulty_gate { return; }

    let kind = inventory.ai[index].kind;
    if kind == PowerUpKind::DoubleBall && balls.iter_mut().count() >= settings.max_active_balls.clamp(2, 4) as usize { return; }
    let use_chance = match settings.difficulty {
        crate::persistence::Difficulty::Easy => 0.70,
        crate::persistence::Difficulty::Medium => 0.86,
        crate::persistence::Difficulty::Hard => 0.96,
        crate::persistence::Difficulty::Expert => 1.0,
    };
    if session.random_unit() > use_chance { return; }
    let cooldown = match kind {
        PowerUpKind::Speed => settings.speed_powerup.cooldown_secs,
        PowerUpKind::Slow => settings.slow_powerup.cooldown_secs,
        PowerUpKind::Enlarge => settings.enlarge_powerup.cooldown_secs,
        PowerUpKind::DoubleBall => settings.double_ball_cooldown_secs,
        PowerUpKind::Fireball => settings.fireball_powerup.cooldown_secs,
        PowerUpKind::Snowball => settings.snowball_powerup.cooldown_secs,
        PowerUpKind::ReversalBall => settings.reversal_powerup.cooldown_secs,
        PowerUpKind::StickyBall => settings.sticky_ball_powerup.cooldown_secs,
        PowerUpKind::BallStorm => settings.ball_storm_powerup.cooldown_secs,
        PowerUpKind::BoomerangBall => settings.boomerang_powerup.cooldown_secs,
        PowerUpKind::MagneticField => settings.magnetic_field_powerup.cooldown_secs,
        PowerUpKind::TeleportBall => settings.teleport_ball_powerup.cooldown_secs,
        PowerUpKind::EnergyShield => settings.energy_shield_powerup.cooldown_secs,
        PowerUpKind::Overdrive => settings.overdrive_powerup.cooldown_secs,
        PowerUpKind::PhantomBall => settings.phantom_ball_powerup.cooldown_secs,
    }.clamp(0.0, 30.0);
    let remove = if inventory.ai[index].charges > 1 {
        inventory.ai[index].charges -= 1;
        inventory.ai[index].cooldown_remaining = cooldown.max(0.35);
        false
    } else { true };
    if remove { inventory.ai.remove(index); }

    apply_powerup(kind, Side::Ai, None, &mut commands, &settings, &mut clock, &mut session, &mut paddles, &mut balls);
    audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUp);
}

