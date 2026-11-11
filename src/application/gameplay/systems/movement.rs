

use bevy::prelude::*;
use crate::{
    audio::{self, Sfx},
    game::components::*,
    persistence::Settings,
};
use super::{world::{ball_color_with_shade, spawn_storm_burst}};

pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<Settings>,
    session: Res<MatchSession>,
    clock: Res<MechanicsClock>,
    arena: Res<ArenaState>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut paddles: Query<(&mut Transform, &Paddle)>,
) {
    if session.mode == MatchMode::Networked || session.phase != MatchPhase::Rally { return; }
    let dt = time.delta_secs().min(0.05);
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());
    for (mut transform, paddle) in &mut paddles {
        if crate::game::mechanics::is_side_frozen(&clock, paddle.side) { continue; }
        let (up_action, down_action, controller_index) = match paddle.side {
            Side::Player => (0, 1, Some(0usize)),
            Side::PlayerTwo if session.mode == MatchMode::LocalVersus => (7, 8, Some(1usize)),
            _ => continue,
        };
        let keyboard_direction: f32 =
            (if settings.controls.key_code(up_action).is_some_and(|key| keyboard.pressed(key)) { 1.0 } else { 0.0 })
                - (if settings.controls.key_code(down_action).is_some_and(|key| keyboard.pressed(key)) { 1.0 } else { 0.0 });
        let gamepad_direction: f32 = controller_index
            .and_then(|index| controllers.get(index))
            .map(|(_, gamepad)| {
                let stick_y = gamepad.left_stick().y;
                let up = stick_y > 0.18 || settings.gamepad_controls.button_code(up_action).is_some_and(|button| gamepad.pressed(button));
                let down = stick_y < -0.18 || settings.gamepad_controls.button_code(down_action).is_some_and(|button| gamepad.pressed(button));
                match (up, down) {
                    (true, false) => 1.0,
                    (false, true) => -1.0,
                    _ => 0.0,
                }
            })
            .unwrap_or(0.0);
        let direction = (keyboard_direction + gamepad_direction).clamp(-1.0, 1.0);
        let overdrive = clock.effects.iter().filter(|effect| effect.kind == PowerUpKind::Overdrive && effect.owner == paddle.side).map(|effect| effect.magnitude).fold(1.0_f32, f32::max);
        transform.translation.y = (transform.translation.y + direction * 440.0 * arena.paddle_speed_multiplier * overdrive * dt)
            .clamp(-FIELD_HALF_H + paddle.height * 0.5, FIELD_HALF_H - paddle.height * 0.5);
    }
}

pub fn move_ball(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    settings: Res<Settings>,
    mut mechanics: ResMut<MechanicsClock>,
    asset_server: Res<AssetServer>,
    mut session: ResMut<MatchSession>,
    arena: Res<ArenaState>,
    mut ball_query: Query<(
        Entity,
        &mut Transform,
        &mut Ball,
        &mut Sprite,
        Option<&mut BallBehavior>,
        Option<&mut StormProjectile>,
    ), (With<Ball>, Without<Paddle>, Without<Obstacle>, Without<PowerUp>)>,
    paddles: Query<(&Transform, &Paddle), (With<Paddle>, Without<Ball>)>,
    obstacles: Query<(&Transform, &Obstacle), (With<Obstacle>, Without<Ball>)>,
) {
    if session.mode == MatchMode::Networked || session.phase != MatchPhase::Rally { return; }
    let dt = time.delta_secs().min(0.05);
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());

    for (entity, mut transform, mut ball, mut ball_sprite, behavior_opt, storm_opt) in &mut ball_query {
        let mut behavior = behavior_opt;
        let mut storm = storm_opt;
        if let Some(state) = behavior.as_ref() {
            ball_sprite.color = match state.kind {
                BallBehaviorKind::Fireball => Color::srgb(1.0, 0.25, 0.05),
                BallBehaviorKind::Sticky => Color::srgb(0.96, 0.40, 0.76),
                BallBehaviorKind::Boomerang => Color::srgb(0.34, 1.0, 0.72),
            };
        } else if storm.is_some() {
            ball_sprite.color = Color::srgb(1.0, 0.52, 0.13);
        } else if ball.is_primary {
            
            
            
            ball_sprite.color = if settings.ball_custom_image.is_some() || settings.ball_skin >= 10 {
                Color::WHITE
            } else {
                ball_color_with_shade(settings.ball_skin, settings.ball_shade)
            };
        } else {
            ball_sprite.color = Color::srgb(1.0, 0.73, 0.28);
        }

        
        
        
        if behavior.as_ref().is_some_and(|state| state.attached) {
            let Some((paddle_transform, _)) = paddles.iter().find(|(_, paddle)| {
                paddle.side == behavior.as_ref().expect("checked above").owner
            }) else { continue; };
            let owner = behavior.as_ref().expect("checked above").owner;
            let direction = if owner == Side::Player { 1.0 } else { -1.0 };
            let offset_y = behavior.as_ref().expect("checked above").offset_y;
            transform.translation = Vec3::new(
                paddle_transform.translation.x + direction * (PADDLE_WIDTH * 0.5 + ball.radius + 1.0),
                paddle_transform.translation.y + offset_y,
                5.0,
            );
            let release_action = if owner == Side::Player { 14 } else { 15 };
            let controller_index = if owner == Side::Player { 0 } else { 1 };
            let gamepad_release = controllers.get(controller_index)
                .is_some_and(|(_, gamepad)| settings.gamepad_controls.button_code(release_action).is_some_and(|button| gamepad.just_pressed(button)));
            let release_pressed = if owner == Side::Ai {
                let target_y = paddles.iter().find(|(_, paddle)| paddle.side == Side::Player)
                    .map(|(player_transform, _)| player_transform.translation.y).unwrap_or(0.0);
                let difficulty_factor = match settings.difficulty { crate::persistence::Difficulty::Easy => 0.60, crate::persistence::Difficulty::Medium => 0.78, crate::persistence::Difficulty::Hard => 0.92, crate::persistence::Difficulty::Expert => 1.0 };
                (session.random_unit() < difficulty_factor * dt * 5.0) || (target_y - paddle_transform.translation.y).abs() < 34.0
            } else {
                settings.controls.key_code(release_action).is_some_and(|key| keyboard.just_pressed(key)) || gamepad_release
            };
            if !release_pressed { continue; }
            let state = behavior.as_mut().expect("attached behavior exists");
            state.attached = false;
            state.returning = false;
            ball.last_hit_by = owner;
            let speed = if state.kind == BallBehaviorKind::Boomerang {
                settings.boomerang_powerup.magnitude.clamp(300.0, 700.0)
            } else {
                settings.sticky_ball_powerup.magnitude.clamp(250.0, 700.0)
            };
            ball.rally_speed = speed;
            ball.velocity = Vec2::new(direction * speed, (offset_y * 0.35).clamp(-95.0, 95.0));
            if state.kind == BallBehaviorKind::Sticky {
                commands.entity(entity).remove::<BallBehavior>();
            }
        }

        if settings.special_ball_enabled && ball.velocity != Vec2::ZERO {
            ball.velocity.y += ball.curve * 22.0 * dt;
            let direction = ball.velocity.normalize_or_zero();
            let natural_speed_cap = (MAX_BALL_SPEED / mechanics.speed_multiplier.max(0.1)).min(MAX_BALL_SPEED);
            ball.rally_speed = (ball.rally_speed + 5.0 * dt).min(natural_speed_cap);
            ball.velocity = direction * ball.rally_speed * mechanics.speed_multiplier * arena.ball_speed_multiplier;
        } else if ball.velocity != Vec2::ZERO {
            ball.curve = 0.0;
            ball.velocity = ball.velocity.normalize_or_zero() * ball.rally_speed * mechanics.speed_multiplier * arena.ball_speed_multiplier;
        }
        if ball.velocity == Vec2::ZERO { continue; }

        for field in mechanics.effects.iter().filter(|effect| effect.kind == PowerUpKind::MagneticField && effect.remaining > 0.0) {
            let target = paddles
                .iter()
                .find(|(_, paddle)| paddle.side == field.owner)
                .map(|(paddle_transform, _)| paddle_transform.translation.truncate())
                .unwrap_or_else(|| Vec2::new(if field.owner == Side::Player { PLAYER_X } else { AI_X }, 0.0));
            let offset = target - transform.translation.truncate();
            let distance = offset.length();
            if distance > 1.0 && distance < 205.0 {
                let speed = ball.velocity.length();
                let influence = (1.0 - distance / 205.0).clamp(0.0, 1.0) * field.magnitude.clamp(0.1, 1.0);
                let adjusted = (ball.velocity.normalize_or_zero() + offset.normalize_or_zero() * influence * dt * 2.4).normalize_or_zero();
                ball.velocity = adjusted * speed;
            }
        }

        if mechanics.effects.iter().any(|effect| effect.kind == PowerUpKind::PhantomBall && effect.remaining > 0.0) {
            ball_sprite.color.set_alpha(0.5);
        }

        transform.translation.x += ball.velocity.x * dt;
        transform.translation.y += ball.velocity.y * dt;
        for shield in mechanics.effects.iter().filter(|effect| effect.kind == PowerUpKind::EnergyShield && effect.remaining > 0.0) {
            let goal_x = if shield.owner == Side::Player { -FIELD_HALF_W } else { FIELD_HALF_W };
            let crossed = if shield.owner == Side::Player { transform.translation.x - ball.radius < goal_x } else { transform.translation.x + ball.radius > goal_x };
            if crossed {
                transform.translation.x = goal_x + if shield.owner == Side::Player { ball.radius + 1.0 } else { -ball.radius - 1.0 };
                ball.velocity.x = ball.velocity.x.abs() * if shield.owner == Side::Player { 1.0 } else { -1.0 };
            }
        }

        let y_limit = FIELD_HALF_H - ball.radius;
        if transform.translation.y > y_limit {
            transform.translation.y = y_limit;
            ball.velocity.y = -ball.velocity.y.abs();
            audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Hit);
        } else if transform.translation.y < -y_limit {
            transform.translation.y = -y_limit;
            ball.velocity.y = ball.velocity.y.abs();
            audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Hit);
        }

        let mut remove_ball = false;
        for (paddle_transform, paddle) in &paddles {
            let dx = (transform.translation.x - paddle_transform.translation.x).abs();
            let dy = (transform.translation.y - paddle_transform.translation.y).abs();
            if dx > PADDLE_WIDTH * 0.5 + ball.radius || dy > paddle.height * 0.5 + ball.radius { continue; }
            let moving_toward = match paddle.side {
                Side::Player => ball.velocity.x < 0.0,
                Side::Ai | Side::PlayerTwo => ball.velocity.x > 0.0,
            };
            if !moving_toward { continue; }

            if let Some(projectile) = storm.as_mut() {
                
                
                if projectile.register_paddle_contact(paddle.side) {
                    spawn_storm_burst(&mut commands, transform.translation);
                    remove_ball = true;
                    audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Hit);
                    break;
                }
                let away_from_paddle = if paddle.side == Side::Player { 1.0 } else { -1.0 };
                transform.translation.x = paddle_transform.translation.x
                    + away_from_paddle * (PADDLE_WIDTH * 0.5 + ball.radius + 1.0);
                ball.velocity.x = away_from_paddle * ball.velocity.x.abs();
                ball.last_hit_by = projectile.owner;
                break;
            }

            let mut boomerang_returned_to_owner = false;
            if let Some(state) = behavior.as_mut() {
                if state.kind == BallBehaviorKind::Boomerang && state.returning && paddle.side == state.owner {
                    if state.remaining_throws > 1 {
                        state.remaining_throws -= 1;
                        state.attached = true;
                        state.returning = false;
                        transform.translation = Vec3::new(
                            paddle_transform.translation.x
                                + if state.owner == Side::Player { 1.0 } else { -1.0 } * (PADDLE_WIDTH * 0.5 + ball.radius + 1.0),
                            paddle_transform.translation.y,
                            5.0,
                        );
                        ball.velocity = Vec2::ZERO;
                        ball.last_hit_by = state.owner;
                        boomerang_returned_to_owner = true;
                    } else {
                        commands.entity(entity).remove::<BallBehavior>();
                    }
                }
                if state.kind == BallBehaviorKind::Fireball && paddle.side != state.owner {
                    push_fireball_damage(&mut mechanics, &settings, state.owner);
                    commands.entity(entity).remove::<BallBehavior>();
                } else if state.kind == BallBehaviorKind::Boomerang && !state.returning && paddle.side != state.owner {
                    push_fireball_damage(&mut mechanics, &settings, state.owner);
                    state.returning = true;
                    ball.last_hit_by = state.owner;
                }
            }
            if boomerang_returned_to_owner { break; }

            let offset = ((transform.translation.y - paddle_transform.translation.y) / (paddle.height * 0.5)).clamp(-1.0, 1.0);
            let direction = if paddle.side == Side::Player { 1.0 } else { -1.0 };
            let natural_speed_cap = (MAX_BALL_SPEED / mechanics.speed_multiplier.max(0.1)).min(MAX_BALL_SPEED);
            let speed = if settings.special_ball_enabled {
                (ball.rally_speed * 1.035).min(natural_speed_cap)
            } else {
                ball.rally_speed
            };
            let angle = offset * 0.95;
            ball.rally_speed = speed;
            ball.velocity = Vec2::new(direction * angle.cos(), angle.sin()) * speed * mechanics.speed_multiplier;
            ball.curve = if settings.special_ball_enabled { (session.random_unit() - 0.5) * 1.2 } else { 0.0 };
            if behavior.as_ref().is_none_or(|state| state.kind != BallBehaviorKind::Boomerang) {
                ball.last_hit_by = paddle.side;
            }
            transform.translation.x = paddle_transform.translation.x + direction * (PADDLE_WIDTH * 0.5 + ball.radius + 1.0);
            audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Hit);
        }
        if remove_ball { commands.entity(entity).despawn(); continue; }

        for (obstacle_transform, obstacle) in &obstacles {
            let dx = (transform.translation.x - obstacle_transform.translation.x).abs();
            let dy = (transform.translation.y - obstacle_transform.translation.y).abs();
            if dx <= obstacle.width * 0.5 + ball.radius && dy <= obstacle.height * 0.5 + ball.radius {
                if ball.velocity.x > 0.0 {
                    transform.translation.x = obstacle_transform.translation.x - obstacle.width * 0.5 - ball.radius - 1.0;
                } else {
                    transform.translation.x = obstacle_transform.translation.x + obstacle.width * 0.5 + ball.radius + 1.0;
                }
                ball.velocity.x = -ball.velocity.x;
                ball.curve *= -0.6;
                audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Hit);
            }
        }
    }
}

fn push_fireball_damage(clock: &mut MechanicsClock, settings: &Settings, owner: Side) {
    let duration = settings.fireball_powerup.duration_secs.clamp(1.0, 15.0);
    let shrink_multiplier = settings.fireball_powerup.magnitude.clamp(0.5, 0.95);
    if let Some(effect) = clock.effects.iter_mut().find(|effect| effect.kind == PowerUpKind::Fireball && effect.owner == owner) {
        effect.remaining = duration;
        effect.magnitude = shrink_multiplier;
    } else {
        clock.effects.push(ActiveTimedEffect {
            kind: PowerUpKind::Fireball,
            owner,
            remaining: duration,
            magnitude: shrink_multiplier,
        });
    }
}

