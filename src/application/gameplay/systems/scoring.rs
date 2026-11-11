

use bevy::prelude::*;
use crate::{
    audio::{self, Sfx},
    game::{components::*, Screen},
    persistence::Settings,
};
use super::world::spawn_score_burst;

pub fn check_scoring(
    time: Res<Time>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut session: ResMut<MatchSession>,
    clock: Res<MechanicsClock>,
    mut ball_query: Query<(Entity, &mut Transform, &mut Ball)>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if session.mode == MatchMode::Networked { return; }
    if session.mode == MatchMode::OpenPlay && !session.open_play_sudden_death && session.phase != MatchPhase::Finished {
        session.open_play_remaining = (session.open_play_remaining - time.delta_secs()).max(0.0);
        if session.open_play_remaining <= 0.0 {
            if session.player_score == session.ai_score {
                session.open_play_sudden_death = true;
            } else {
                session.phase = MatchPhase::Finished;
                session.phase_timer = 1.15;
            }
        }
    }
    match session.phase {
        MatchPhase::Rally => {
            let mut scorer = None;
            for (_, transform, _) in &mut ball_query {
                if transform.translation.x < -FIELD_HALF_W - 14.0 {
                    scorer = Some(if session.mode == MatchMode::LocalVersus { Side::PlayerTwo } else { Side::Ai });
                    break;
                }
                if transform.translation.x > FIELD_HALF_W + 14.0 {
                    scorer = Some(Side::Player);
                    break;
                }
            }

            if let Some(scorer) = scorer {
                match scorer {
                    Side::Player => session.player_score += 1,
                    Side::Ai | Side::PlayerTwo => session.ai_score += 1,
                }
                session.last_scorer = Some(scorer);
                session.rally_points += 1;
                audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Score);
                spawn_score_burst(&mut commands, scorer);
                
                
                let mut kept_primary = false;
                for (entity, mut transform, mut ball) in &mut ball_query {
                    if ball.is_primary && !kept_primary {
                        kept_primary = true;
                        transform.translation = Vec3::new(0.0, 0.0, 5.0);
                        ball.velocity = Vec2::ZERO;
                        ball.curve = 0.0;
                        ball.rally_speed = BALL_START_SPEED;
                    } else {
                        commands.entity(entity).despawn();
                    }
                }
                if (session.mode == MatchMode::OpenPlay && session.open_play_sudden_death)
                    || (session.mode != MatchMode::OpenPlay && (session.player_score >= session.winning_score || session.ai_score >= session.winning_score)) {
                    session.phase = MatchPhase::Finished;
                    session.phase_timer = 1.15;
                } else {
                    session.phase = MatchPhase::PointPause;
                    session.phase_timer = 1.35;
                }
            }
        }
        MatchPhase::PointPause => {
            session.phase_timer -= time.delta_secs();
            if session.phase_timer <= 0.0 {
                for (_, mut transform, mut ball) in &mut ball_query {
                    if ball.is_primary {
                        transform.translation = Vec3::new(0.0, 0.0, 5.0);
                        ball.rally_speed = BALL_START_SPEED;
                        let direction = if matches!(session.last_scorer, Some(Side::Ai | Side::PlayerTwo)) { -1.0 } else { 1.0 };
                        let angle = (session.random_unit() - 0.5) * 0.85;
                        ball.velocity = Vec2::new(direction * angle.cos(), angle.sin()) * BALL_START_SPEED * clock.speed_multiplier;
                        ball.curve = 0.0;
                    }
                }
                session.phase = MatchPhase::Rally;
            }
        }
        MatchPhase::Finished => {
            session.phase_timer -= time.delta_secs();
            if session.phase_timer <= 0.0 {
                next_state.set(Screen::Results);
            }
        }
    }
}

