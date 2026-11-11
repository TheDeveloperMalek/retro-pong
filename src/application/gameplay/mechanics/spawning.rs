

use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;
use crate::{
    game::components::*,
    persistence::Settings,
};
use super::effects::recalculate_effects;


fn select_powerup_kind(settings: &Settings, session: &mut MatchSession) -> Option<PowerUpKind> {
    let choices = [
        (PowerUpKind::Speed, settings.speed_powerup.enabled, settings.speed_powerup.spawn_weight),
        (PowerUpKind::Slow, settings.slow_powerup.enabled, settings.slow_powerup.spawn_weight),
        (PowerUpKind::Enlarge, settings.enlarge_powerup.enabled, settings.enlarge_powerup.spawn_weight),
        (PowerUpKind::DoubleBall, settings.double_ball_enabled, settings.double_ball_spawn_probability),
        (PowerUpKind::Fireball, settings.fireball_powerup.enabled, settings.fireball_powerup.spawn_weight),
        (PowerUpKind::Snowball, settings.snowball_powerup.enabled, settings.snowball_powerup.spawn_weight),
        (PowerUpKind::ReversalBall, settings.reversal_powerup.enabled, settings.reversal_powerup.spawn_weight),
        (PowerUpKind::StickyBall, settings.sticky_ball_powerup.enabled, settings.sticky_ball_powerup.spawn_weight),
        (PowerUpKind::BallStorm, settings.ball_storm_powerup.enabled, settings.ball_storm_powerup.spawn_weight),
        (PowerUpKind::BoomerangBall, settings.boomerang_powerup.enabled, settings.boomerang_powerup.spawn_weight),
        (PowerUpKind::MagneticField, settings.magnetic_field_powerup.enabled, settings.magnetic_field_powerup.spawn_weight),
        (PowerUpKind::TeleportBall, settings.teleport_ball_powerup.enabled, settings.teleport_ball_powerup.spawn_weight),
        (PowerUpKind::EnergyShield, settings.energy_shield_powerup.enabled, settings.energy_shield_powerup.spawn_weight),
        (PowerUpKind::Overdrive, settings.overdrive_powerup.enabled, settings.overdrive_powerup.spawn_weight),
        (PowerUpKind::PhantomBall, settings.phantom_ball_powerup.enabled, settings.phantom_ball_powerup.spawn_weight),
    ];

    let choices: Vec<(PowerUpKind, f32)> = choices
        .into_iter()
        .filter(|(_, enabled, weight)| *enabled && weight.is_finite() && *weight > 0.0)
        .map(|(kind, _, weight)| (kind, weight.clamp(0.0, 1.0)))
        .collect();
    let total_weight: f32 = choices.iter().map(|(_, weight)| *weight).sum();
    if total_weight <= f32::EPSILON || !total_weight.is_finite() {
        return None;
    }

    let roll = session.random_unit() * total_weight;
    let mut cumulative_weight = 0.0;
    let fallback = choices.last().map(|(kind, _)| *kind)?;
    for (kind, weight) in choices {
        cumulative_weight += weight;
        if roll < cumulative_weight {
            return Some(kind);
        }
    }

    Some(fallback)
}

pub(super) fn spawn_random_powerup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    settings: &Settings,
    session: &mut MatchSession,
) -> bool {
    let Some(selected_kind) = select_powerup_kind(settings, session) else { return false; };

    let y = session.random_unit() * 290.0 - 145.0;
    let x = session.random_unit() * 150.0 - 75.0;
    let first_frame = format!("powerups/{}/frame_0.png", selected_kind.pixel_art_key());
    let mut sprite = Sprite::from_image(asset_server.load(first_frame));
    sprite.custom_size = Some(Vec2::splat(30.0));
    commands.spawn((
        GameEntity,
        PowerUp { kind: selected_kind, age: 0.0, frame_index: 0 },
        sprite,
        Transform::from_xyz(x, y, 4.0),
    ));
    true
}

pub(super) fn apply_powerup<F: QueryFilter>(
    kind: PowerUpKind,
    owner: Side,
    collected_ball: Option<Entity>,
    commands: &mut Commands,
    settings: &Settings,
    clock: &mut MechanicsClock,
    session: &mut MatchSession,
    paddles: &mut Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
    balls: &mut Query<(Entity, &mut Transform, &mut Ball), F>,
) {
    match kind {
        PowerUpKind::DoubleBall => {
            split_ball(commands, settings, balls);
        }
        PowerUpKind::Speed | PowerUpKind::Slow | PowerUpKind::Enlarge => {
            let (duration, magnitude) = match kind {
                PowerUpKind::Speed => (settings.speed_powerup.duration_secs, settings.speed_powerup.magnitude),
                PowerUpKind::Slow => (settings.slow_powerup.duration_secs, settings.slow_powerup.magnitude),
                PowerUpKind::Enlarge => (settings.enlarge_powerup.duration_secs, settings.enlarge_powerup.magnitude),
                _ => unreachable!(),
            };
            push_or_refresh_effect(clock, kind, owner, duration, magnitude);
            recalculate_effects(clock, paddles, balls);
        }
        PowerUpKind::Fireball => {
            if let Some(entity) = find_target_ball(collected_ball, balls) {
                commands.entity(entity).insert(BallBehavior {
                    kind: BallBehaviorKind::Fireball, owner, attached: false,
                    returning: false, remaining_throws: 0, offset_y: 0.0,
                });
            }
        }
        PowerUpKind::Snowball => {
            push_or_refresh_effect(clock, PowerUpKind::Snowball, owner, settings.snowball_powerup.duration_secs, 0.0);
        }
        PowerUpKind::ReversalBall => {
            if let Some(entity) = find_target_ball(collected_ball, balls) {
                if let Ok((_, _, mut ball)) = balls.get_mut(entity) {
                    ball.velocity = -ball.velocity;
                    ball.curve = -ball.curve;
                }
            }
        }
        PowerUpKind::StickyBall | PowerUpKind::BoomerangBall => {
            let Some(entity) = find_target_ball(collected_ball, balls) else { return; };
            let mut owner_position = None;
            for (transform, paddle, _) in paddles.iter_mut() {
                if paddle.side == owner { owner_position = Some(transform.translation); break; }
            }
            let Some(paddle_position) = owner_position else { return; };
            if let Ok((_, mut transform, mut ball)) = balls.get_mut(entity) {
                let direction = if owner == Side::Player { 1.0 } else { -1.0 };
                transform.translation = Vec3::new(
                    paddle_position.x + direction * (PADDLE_WIDTH * 0.5 + ball.radius + 1.0),
                    paddle_position.y,
                    5.0,
                );
                ball.velocity = Vec2::ZERO;
                ball.curve = 0.0;
                ball.last_hit_by = owner;
                commands.entity(entity).insert(BallBehavior {
                    kind: if kind == PowerUpKind::StickyBall { BallBehaviorKind::Sticky } else { BallBehaviorKind::Boomerang },
                    owner,
                    attached: true,
                    returning: false,
                    remaining_throws: if kind == PowerUpKind::BoomerangBall { settings.boomerang_powerup.throws.clamp(1, 5) } else { 1 },
                    offset_y: 0.0,
                });
            }
        }
        PowerUpKind::BallStorm => spawn_ball_storm(commands, settings, session, owner, paddles),
        PowerUpKind::MagneticField => push_or_refresh_effect(clock, kind, owner, settings.magnetic_field_powerup.duration_secs.clamp(1.0, 10.0), settings.magnetic_field_powerup.magnitude.clamp(0.1, 1.0)),
        PowerUpKind::TeleportBall => {
            if let Some(entity) = find_target_ball(collected_ball, balls) {
                let target_x = if owner == Side::Player { FIELD_HALF_W * 0.48 } else { -FIELD_HALF_W * 0.48 };
                let (mut y, radius) = if let Ok((_, _, ball)) = balls.get_mut(entity) { (session.random_unit() * (FIELD_HALF_H - ball.radius) * 1.7 - (FIELD_HALF_H - ball.radius) * 0.85, ball.radius) } else { (0.0, BALL_SIZE * 0.5) };
                y = y.clamp(-FIELD_HALF_H + radius + 2.0, FIELD_HALF_H - radius - 2.0);
                if let Ok((_, mut transform, _)) = balls.get_mut(entity) {
                    transform.translation = Vec3::new(target_x, y, transform.translation.z);
                }
            }
        }
        PowerUpKind::EnergyShield => push_or_refresh_effect(clock, kind, owner, settings.energy_shield_powerup.duration_secs.clamp(1.0, 12.0), 1.0),
        PowerUpKind::Overdrive => push_or_refresh_effect(clock, kind, owner, settings.overdrive_powerup.duration_secs.clamp(1.0, 10.0), settings.overdrive_powerup.magnitude.clamp(1.1, 2.0)),
        PowerUpKind::PhantomBall => push_or_refresh_effect(clock, kind, owner, settings.phantom_ball_powerup.duration_secs.clamp(1.0, 10.0), settings.phantom_ball_powerup.magnitude.clamp(0.2, 0.8)),
    }
}

fn find_target_ball<F: QueryFilter>(
    preferred: Option<Entity>,
    balls: &mut Query<(Entity, &mut Transform, &mut Ball), F>,
) -> Option<Entity> {
    if let Some(entity) = preferred {
        if balls.get_mut(entity).is_ok() { return Some(entity); }
    }
    balls.iter_mut().find(|(_, _, ball)| ball.is_primary).map(|(entity, _, _)| entity)
}

fn push_or_refresh_effect(clock: &mut MechanicsClock, kind: PowerUpKind, owner: Side, duration: f32, magnitude: f32) {
    if let Some(existing) = clock.effects.iter_mut().find(|effect| effect.kind == kind && effect.owner == owner) {
        existing.remaining = duration;
        existing.magnitude = magnitude;
    } else {
        clock.effects.push(ActiveTimedEffect { kind, owner, remaining: duration, magnitude });
    }
}

fn spawn_ball_storm(
    commands: &mut Commands,
    settings: &Settings,
    session: &mut MatchSession,
    owner: Side,
    paddles: &mut Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
) {
    let mut origin_y = 0.0;
    for (transform, paddle, _) in paddles.iter_mut() {
        if paddle.side == owner { origin_y = transform.translation.y; break; }
    }
    let direction = if owner == Side::Player { 1.0 } else { -1.0 };
    let origin_x = if direction > 0.0 { PLAYER_X + 14.0 } else { AI_X - 14.0 };
    let min_balls = settings.ball_storm_powerup.min_projectiles.clamp(3, 5) as usize;
    let max_balls = settings.ball_storm_powerup.max_projectiles.clamp(min_balls as u8, 5) as usize;
    let count = min_balls + (session.random_unit() * (max_balls - min_balls + 1) as f32).floor() as usize;
    let projectile_speed = settings.ball_storm_powerup.magnitude.clamp(300.0, 750.0);
    let opponent_blocks = settings.ball_storm_powerup.opponent_blocks.clamp(1, 5);
    for index in 0..count {
        let spread = (index as f32 - (count as f32 - 1.0) * 0.5) * 0.19;
        let jitter = (session.random_unit() - 0.5) * 0.12;
        let angle = spread + jitter;
        let velocity = Vec2::new(direction * angle.cos(), angle.sin()) * projectile_speed;
        commands.spawn((
            GameEntity,
            Ball {
                velocity, curve: 0.0, radius: BALL_SIZE * 0.36, rally_speed: projectile_speed,
                last_hit_by: owner, is_primary: false,
            },
            StormProjectile { owner, remaining_blocks: opponent_blocks },
            Sprite::from_color(Color::srgb(1.0, 0.52, 0.13), Vec2::splat(BALL_SIZE * 0.72)),
            Transform::from_xyz(origin_x, origin_y + spread * 34.0, 5.2),
        ));
    }
}

fn split_ball<F: QueryFilter>(
    commands: &mut Commands,
    settings: &Settings,
    balls: &mut Query<(Entity, &mut Transform, &mut Ball), F>,
) {
    let active_count = balls.iter_mut().count();
    if active_count >= settings.max_active_balls.clamp(2, 4) as usize { return; }
    let mut spawn_data = None;
    for (_, transform, mut original) in balls.iter_mut() {
        if !original.is_primary { continue; }
        let position = transform.translation + Vec3::new(0.0, 0.0, 0.1);
        let original_data = *original;
        let speed = original_data.velocity.length().max(original_data.rally_speed);
        let base_direction = original_data.velocity.normalize_or_zero();
        if base_direction == Vec2::ZERO { return; }
        original.velocity = rotate_direction(base_direction, -0.24) * speed;
        original.curve = -original_data.curve.abs();
        let mut secondary = original_data;
        secondary.velocity = rotate_direction(base_direction, 0.24) * speed;
        secondary.curve = original_data.curve.abs();
        secondary.is_primary = false;
        spawn_data = Some((position, secondary));
        break;
    }
    let Some((position, secondary)) = spawn_data else { return; };
    commands.spawn((
        GameEntity,
        secondary,
        Sprite::from_color(Color::srgb(1.0, 0.73, 0.28), Vec2::splat(BALL_SIZE)),
        Transform::from_translation(position + Vec3::new(0.0, BALL_SIZE * 1.8, 0.0)),
    ));
}

fn rotate_direction(direction: Vec2, angle: f32) -> Vec2 { direction.rotate_angle(angle) }


