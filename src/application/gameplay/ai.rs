use bevy::prelude::*;
use crate::{game::components::*, persistence::Settings};

pub fn move_ai(
    time: Res<Time>,
    settings: Res<Settings>,
    session: Res<MatchSession>,
    clock: Res<MechanicsClock>,
    arena: Res<ArenaState>,
    mut random: ResMut<AiRandom>,
    mut paddles: Query<(&mut Transform, &Paddle, &mut AiBrain), Without<Ball>>,
    ball_query: Query<(&Transform, &Ball), Without<Paddle>>,
) {
    if session.phase != MatchPhase::Rally || ![MatchMode::SinglePlayer, MatchMode::OpenPlay].contains(&session.mode) { return; }

    
    
    let mut target: Option<(f32, Vec2, Vec2, f32)> = None;
    for (transform, ball) in &ball_query {
        if ball.velocity.x <= 0.0 { continue; }
        let position = transform.translation.truncate();
        let travel_time = ((AI_X - position.x) / ball.velocity.x.max(1.0)).max(0.0);
        if target.as_ref().map(|current| travel_time < current.0).unwrap_or(true) {
            target = Some((travel_time, position, ball.velocity, ball.radius));
        }
    }

    let (reaction_interval, max_speed, error_range) = difficulty_tuning(settings.difficulty);
    let dt = time.delta_secs().min(0.05);

    for (mut transform, paddle, mut brain) in &mut paddles {
        if paddle.side != Side::Ai || crate::game::mechanics::is_side_frozen(&clock, Side::Ai) { continue; }
        brain.reaction_timer -= dt;
        if brain.reaction_timer <= 0.0 {
            brain.reaction_timer = reaction_interval;
            if let Some((travel_time, ball_position, ball_velocity, radius)) = target {
                let projected = ball_position.y + ball_velocity.y * travel_time;
                brain.target_y = reflected_y(projected, FIELD_HALF_H - radius);
                brain.error_offset = (random.unit() * 2.0 - 1.0) * error_range;
            } else {
                brain.target_y *= 0.8;
                brain.error_offset *= 0.65;
            }
        }
        let target_y = (brain.target_y + brain.error_offset)
            .clamp(-FIELD_HALF_H + paddle.height * 0.5, FIELD_HALF_H - paddle.height * 0.5);
        let delta = target_y - transform.translation.y;
        let step = max_speed * arena.paddle_speed_multiplier * dt * clock.effects.iter().filter(|effect| effect.kind == PowerUpKind::Overdrive && effect.owner == Side::Ai).map(|effect| effect.magnitude).fold(1.0_f32, f32::max);
        transform.translation.y += delta.clamp(-step, step);
    }
}

fn reflected_y(raw_y: f32, bound: f32) -> f32 {
    let bound = bound.max(1.0);
    let half_period = bound * 2.0;
    let wrapped = (raw_y + bound).rem_euclid(half_period * 2.0);
    if wrapped <= half_period { -bound + wrapped } else { (bound * 3.0) - wrapped }
}
