

use super::NetworkRuntime;
use crate::{
    audio::{self, Sfx},
    game::components::*,
    network::protocol::{
        wire_code::{effect as effect_code, power_up as power_up_code, power_up_event as power_up_event_code, special_ball as special_ball_code},
        BallSnapshot, EffectSnapshot, MatchSnapshot, PowerUpSlotSnapshot,
    },
    persistence::Settings,
};
use bevy::prelude::*;

pub fn apply_snapshot(
    mut commands: Commands,
    mut runtime: ResMut<NetworkRuntime>,
    mut session: ResMut<MatchSession>,
    mut inventory: ResMut<PowerUpInventory>,
    mut clock: ResMut<MechanicsClock>,
    mut paddles: Query<(&mut Transform, &mut Paddle, &mut Sprite), (Without<Ball>, Without<PowerUp>)>,
    mut balls: Query<(
        Entity,
        &mut Transform,
        &mut Ball,
        &mut Sprite,
        Option<&StormProjectile>,
    ), (With<Ball>, Without<Paddle>, Without<PowerUp>)>,
    mut pickups: Query<
        (Entity, &mut Transform, &mut PowerUp, &mut Sprite),
        (With<PowerUp>, Without<Ball>, Without<Paddle>),
    >,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    
    
    let Some(snapshot) = runtime.latest_snapshot.clone() else {
        return;
    };

    for sound in unseen_powerup_sounds(&snapshot.powerup_events, &mut runtime.last_powerup_event_id) {
        audio::play_sfx(&mut commands, &asset_server, &settings, sound);
    }

    apply_match_state(&mut session, &snapshot);
    apply_paddle_state(&mut paddles, &snapshot);

    let mut ball_index = 0;
    for (entity, mut transform, mut ball, mut sprite, storm) in &mut balls {
        let Some(state) = snapshot.balls.get(ball_index) else {
            if storm.is_some() {
                crate::game::systems::spawn_storm_burst(&mut commands, transform.translation);
            }
            commands.entity(entity).despawn();
            continue;
        };

        update_ball_presentation(&mut transform, &mut ball, &mut sprite, state, &settings, &asset_server);
        apply_special_ball_components(&mut commands, entity, state);
        ball_index += 1;
    }

    for state in snapshot.balls.iter().skip(ball_index) {
        spawn_snapshot_ball(&mut commands, state, &settings, &asset_server);
    }

    apply_pickup_state(&mut commands, &asset_server, time.delta_secs(), &settings, &mut pickups, &snapshot);
    inventory.player_one = convert_inventory_slots(&snapshot.inventories[0]);
    inventory.player_two = convert_inventory_slots(&snapshot.inventories[1]);
    clock.effects = convert_effects(&snapshot.effects);
    clock.speed_multiplier = clock
        .effects
        .iter()
        .filter(|effect| matches!(effect.kind, PowerUpKind::Speed | PowerUpKind::Slow))
        .fold(1.0_f32, |value, effect| value * effect.magnitude)
        .clamp(0.35, 2.5);
    let phantom_active = clock.effects.iter().any(|effect| effect.kind == PowerUpKind::PhantomBall && effect.remaining > 0.0);
    for (_, _, _, mut sprite, _) in &mut balls {
        if phantom_active { sprite.color.set_alpha(0.5); }
        else { sprite.color.set_alpha(1.0); }
    }
}

fn apply_match_state(session: &mut MatchSession, snapshot: &MatchSnapshot) {
    session.player_score = snapshot.scores[0];
    session.ai_score = snapshot.scores[1];
    session.winning_score = snapshot.winning_score;
    session.phase = match snapshot.phase.as_str() {
        "finished" => MatchPhase::Finished,
        "point_pause" => MatchPhase::PointPause,
        _ => MatchPhase::Rally,
    };
}

fn apply_paddle_state(
    paddles: &mut Query<(&mut Transform, &mut Paddle, &mut Sprite), (Without<Ball>, Without<PowerUp>)>,
    snapshot: &MatchSnapshot,
) {
    for (mut transform, mut paddle, mut sprite) in paddles.iter_mut() {
        let team_index = team_index(paddle.side);
        transform.translation.y = snapshot.paddle_y[team_index];
        paddle.height = snapshot.paddle_height[team_index];
        sprite.custom_size = Some(Vec2::new(PADDLE_WIDTH, paddle.height));
    }
}

fn update_ball_presentation(
    transform: &mut Transform,
    ball: &mut Ball,
    sprite: &mut Sprite,
    state: &BallSnapshot,
    settings: &Settings,
    asset_server: &AssetServer,
) {
    transform.translation.x = state.x;
    transform.translation.y = state.y;
    ball.velocity = Vec2::new(state.vx, state.vy);
    ball.rally_speed = ball.velocity.length();
    ball.radius = state.radius;
    ball.curve = 0.0;
    ball.last_hit_by = side_from_team(state.last_hit_team);
    ball.is_primary = state.primary;
    sprite.custom_size = Some(Vec2::splat(state.radius * 2.0));
    if state.primary && state.special_kind == special_ball_code::NORMAL {
        *sprite = crate::game::systems::ball_skin_sprite(settings, asset_server);
        sprite.custom_size = Some(Vec2::splat(state.radius * 2.0));
    } else {
        sprite.color = snapshot_ball_color(state.special_kind, state.primary, settings.ball_skin, settings.ball_shade);
    }
}

fn spawn_snapshot_ball(commands: &mut Commands, state: &BallSnapshot, settings: &Settings, asset_server: &AssetServer) {
    let velocity = Vec2::new(state.vx, state.vy);
    let mut sprite = if state.primary && state.special_kind == special_ball_code::NORMAL {
        crate::game::systems::ball_skin_sprite(settings, asset_server)
    } else {
        Sprite::from_color(
            snapshot_ball_color(state.special_kind, state.primary, settings.ball_skin, settings.ball_shade),
            Vec2::splat(state.radius * 2.0),
        )
    };
    sprite.custom_size = Some(Vec2::splat(state.radius * 2.0));
    let entity = commands
        .spawn((
            GameEntity,
            Ball {
                velocity,
                curve: 0.0,
                radius: state.radius,
                rally_speed: velocity.length(),
                last_hit_by: side_from_team(state.last_hit_team),
                is_primary: state.primary,
            },
            sprite,
            Transform::from_xyz(state.x, state.y, 5.0),
        ))
        .id();

    apply_special_ball_components(commands, entity, state);
}

fn apply_special_ball_components(commands: &mut Commands, entity: Entity, state: &BallSnapshot) {
    let mut entity_commands = commands.entity(entity);

    if let Some(behavior) = ball_behavior(state) {
        entity_commands.insert(behavior);
        entity_commands.remove::<StormProjectile>();
        return;
    }

    if state.special_kind == special_ball_code::STORM_PROJECTILE {
        entity_commands.insert(StormProjectile {
            owner: side_from_team(state.special_owner),
            remaining_blocks: state.special_hits,
        });
        entity_commands.remove::<BallBehavior>();
        return;
    }

    entity_commands.remove::<BallBehavior>();
    entity_commands.remove::<StormProjectile>();
}

fn ball_behavior(state: &BallSnapshot) -> Option<BallBehavior> {
    let kind = match state.special_kind {
        special_ball_code::FIREBALL => BallBehaviorKind::Fireball,
        special_ball_code::STICKY => BallBehaviorKind::Sticky,
        special_ball_code::BOOMERANG => BallBehaviorKind::Boomerang,
        _ => return None,
    };

    Some(BallBehavior {
        kind,
        owner: side_from_team(state.special_owner),
        attached: kind != BallBehaviorKind::Fireball && state.attached,
        returning: kind == BallBehaviorKind::Boomerang && state.returning,
        remaining_throws: match kind {
            BallBehaviorKind::Sticky => 1,
            BallBehaviorKind::Boomerang => state.throws_left,
            BallBehaviorKind::Fireball => 0,
        },
        offset_y: state.offset_y,
    })
}

fn apply_pickup_state(
    commands: &mut Commands,
    asset_server: &AssetServer,
    delta_secs: f32,
    settings: &Settings,
    pickups: &mut Query<
        (Entity, &mut Transform, &mut PowerUp, &mut Sprite),
        (With<PowerUp>, Without<Ball>, Without<Paddle>),
    >,
    snapshot: &MatchSnapshot,
) {
    let mut pickup_index = 0;

    for (entity, mut transform, mut pickup, mut sprite) in pickups.iter_mut() {
        let Some(state) = snapshot.pickups.get(pickup_index) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Some(kind) = powerup_kind(state.kind) else {
            commands.entity(entity).despawn();
            pickup_index += 1;
            continue;
        };

        if pickup.kind != kind {
            pickup.kind = kind;
            pickup.frame_index = 0;
            sprite.image = asset_server.load(format!("powerups/{}/frame_0.png", kind.pixel_art_key()));
        }
        pickup.age += delta_secs.min(0.05);
        if !settings.reduce_motion {
            let frame_count = kind.pixel_art_frame_count().max(1);
            let frame_duration = kind.pixel_art_frame_duration().max(0.02);
            let frame = ((pickup.age / frame_duration).floor() as u8) % frame_count;
            if frame != pickup.frame_index {
                pickup.frame_index = frame;
                sprite.image = asset_server.load(format!("powerups/{}/frame_{}.png", kind.pixel_art_key(), frame));
            }
        }
        transform.translation.x = state.x;
        transform.translation.y = state.y;
        sprite.custom_size = Some(Vec2::splat(30.0));
        pickup_index += 1;
    }

    for state in snapshot.pickups.iter().skip(pickup_index) {
        let Some(kind) = powerup_kind(state.kind) else {
            continue;
        };
        let mut sprite = Sprite::from_image(
            asset_server.load(format!("powerups/{}/frame_0.png", kind.pixel_art_key())),
        );
        sprite.custom_size = Some(Vec2::splat(30.0));
        commands.spawn((
            GameEntity,
            PowerUp { kind, age: 0.0, frame_index: 0 },
            sprite,
            Transform::from_xyz(state.x, state.y, 4.0),
        ));
    }
}

fn convert_inventory_slots(slots: &[PowerUpSlotSnapshot]) -> Vec<PowerUpSlot> {
    slots
        .iter()
        .filter_map(|slot| {
            powerup_kind(slot.kind).map(|kind| PowerUpSlot {
                kind,
                charges: slot.charges,
                cooldown_remaining: slot.cooldown_remaining,
            })
        })
        .collect()
}

fn convert_effects(snapshots: &[EffectSnapshot]) -> Vec<ActiveTimedEffect> {
    snapshots
        .iter()
        .filter_map(|effect| {
            effect_kind(effect.kind).map(|kind| ActiveTimedEffect {
                kind,
                owner: side_from_team(effect.team),
                remaining: effect.remaining,
                magnitude: effect.magnitude,
            })
        })
        .collect()
}

fn side_from_team(team: u8) -> Side {
    if team == 0 { Side::Player } else { Side::PlayerTwo }
}

fn team_index(side: Side) -> usize {
    if side == Side::Player { 0 } else { 1 }
}

fn snapshot_ball_color(special_kind: u8, primary: bool, skin: u8, shade: i8) -> Color {
    match special_kind {
        special_ball_code::FIREBALL => Color::srgb(1.0, 0.25, 0.05),
        special_ball_code::STICKY => Color::srgb(0.96, 0.40, 0.76),
        special_ball_code::BOOMERANG => Color::srgb(0.34, 1.0, 0.72),
        special_ball_code::STORM_PROJECTILE => Color::srgb(1.0, 0.52, 0.13),
        _ if primary => crate::game::systems::ball_color_with_shade(skin, shade),
        _ => Color::srgb(1.0, 0.73, 0.28),
    }
}

fn powerup_kind(value: u8) -> Option<PowerUpKind> {
    match value {
        power_up_code::SPEED => Some(PowerUpKind::Speed),
        power_up_code::SLOW => Some(PowerUpKind::Slow),
        power_up_code::ENLARGE => Some(PowerUpKind::Enlarge),
        power_up_code::DOUBLE_BALL => Some(PowerUpKind::DoubleBall),
        power_up_code::FIREBALL => Some(PowerUpKind::Fireball),
        power_up_code::SNOWBALL => Some(PowerUpKind::Snowball),
        power_up_code::REVERSAL_BALL => Some(PowerUpKind::ReversalBall),
        power_up_code::STICKY_BALL => Some(PowerUpKind::StickyBall),
        power_up_code::BALL_STORM => Some(PowerUpKind::BallStorm),
        power_up_code::BOOMERANG_BALL => Some(PowerUpKind::BoomerangBall),
        power_up_code::MAGNETIC_FIELD => Some(PowerUpKind::MagneticField),
        power_up_code::TELEPORT_BALL => Some(PowerUpKind::TeleportBall),
        power_up_code::ENERGY_SHIELD => Some(PowerUpKind::EnergyShield),
        power_up_code::OVERDRIVE => Some(PowerUpKind::Overdrive),
        power_up_code::PHANTOM_BALL => Some(PowerUpKind::PhantomBall),
        _ => None,
    }
}

fn effect_kind(value: u8) -> Option<PowerUpKind> {
    match value {
        effect_code::SPEED => Some(PowerUpKind::Speed),
        effect_code::SLOW => Some(PowerUpKind::Slow),
        effect_code::ENLARGE => Some(PowerUpKind::Enlarge),
        effect_code::FIREBALL_SHRINK => Some(PowerUpKind::Fireball),
        effect_code::SNOWBALL_FREEZE => Some(PowerUpKind::Snowball),
        effect_code::MAGNETIC_FIELD => Some(PowerUpKind::MagneticField),
        effect_code::ENERGY_SHIELD => Some(PowerUpKind::EnergyShield),
        effect_code::OVERDRIVE => Some(PowerUpKind::Overdrive),
        effect_code::PHANTOM_BALL => Some(PowerUpKind::PhantomBall),
        _ => None,
    }
}



fn unseen_powerup_sounds(events: &[crate::network::protocol::PowerUpEventSnapshot], last_event_id: &mut u64) -> Vec<Sfx> {
    let mut sounds = Vec::new();
    let previously_seen = *last_event_id;
    for event in events.iter().filter(|event| event.id > previously_seen) {
        let sound = match event.event {
            power_up_event_code::RESPAWN => Some(Sfx::PowerUpRespawn),
            power_up_event_code::COLLECT => powerup_kind(event.kind).map(Sfx::PowerUpCollect),
            _ => None,
        };
        if let Some(sound) = sound {
            sounds.push(sound);
        }
        *last_event_id = (*last_event_id).max(event.id);
    }
    sounds
}

