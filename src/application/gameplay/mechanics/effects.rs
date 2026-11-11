

use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;
use crate::game::components::*;

pub fn recalculate_effects<F: QueryFilter>(
    clock: &mut MechanicsClock,
    paddles: &mut Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
    balls: &mut Query<(Entity, &mut Transform, &mut Ball), F>,
) {
    let mut speed_multiplier = 1.0_f32;
    for effect in &clock.effects {
        match effect.kind {
            PowerUpKind::Speed | PowerUpKind::Slow => speed_multiplier *= effect.magnitude,
            _ => {},
        }
    }
    clock.speed_multiplier = speed_multiplier.clamp(0.35, 2.5);

    for (_, mut paddle, mut sprite) in paddles.iter_mut() {
        let enlarge = clock.effects.iter()
            .filter(|effect| effect.kind == PowerUpKind::Enlarge && effect.owner == paddle.side)
            .map(|effect| effect.magnitude)
            .fold(1.0_f32, f32::max);
        let fire_shrink = clock.effects.iter()
            .filter(|effect| effect.kind == PowerUpKind::Fireball && effect.owner != paddle.side)
            .map(|effect| effect.magnitude)
            .fold(1.0_f32, f32::min);
        paddle.height = (paddle.base_height * enlarge * fire_shrink).clamp(42.0, 180.0);
        sprite.custom_size = Some(Vec2::new(PADDLE_WIDTH, paddle.height));
    }

    for (_, _, mut ball) in balls.iter_mut() {
        if ball.velocity == Vec2::ZERO { continue; }
        let direction = ball.velocity.normalize_or_zero();
        if direction != Vec2::ZERO {
            ball.velocity = direction * ball.rally_speed * clock.speed_multiplier;
        }
    }
}

pub fn clear_timed_effect<F: QueryFilter>(
    clock: &mut MechanicsClock,
    paddles: &mut Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
    balls: &mut Query<(Entity, &mut Transform, &mut Ball), F>,
) {
    clock.effects.clear();
    clock.speed_multiplier = 1.0;
    recalculate_effects(clock, paddles, balls);
}

pub fn is_side_frozen(clock: &MechanicsClock, side: Side) -> bool {
    clock.effects.iter().any(|effect| {
        effect.kind == PowerUpKind::Snowball && effect.owner != side && effect.remaining > 0.0
    })
}
