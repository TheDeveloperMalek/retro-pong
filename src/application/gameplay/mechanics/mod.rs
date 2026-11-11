




use bevy::prelude::*;
use crate::{
    audio::{self, Sfx},
    game::components::*,
    persistence::Settings,
};

mod effects;
mod inventory;
mod spawning;

pub use effects::{clear_timed_effect, is_side_frozen, recalculate_effects};
pub use inventory::{activate_ai_inventory_items, activate_inventory_items, powerup_max_charges};
use spawning::{apply_powerup, spawn_random_powerup};

const PICKUP_LIFETIME_SECONDS: f32 = 13.0;



fn inventory_has_capacity(inventory: &PowerUpInventory, owner: Side, configured_capacity: u8) -> bool {
    inventory.for_side(owner).len() < configured_capacity.clamp(1, 5) as usize
}

pub fn tick_mechanics(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    settings: Res<Settings>,
    mut clock: ResMut<MechanicsClock>,
    mut session: ResMut<MatchSession>,
    mut inventory: ResMut<PowerUpInventory>,
    mut pending_pickup: ResMut<PendingPowerUp>,
    mut balls: Query<(Entity, &mut Transform, &mut Ball), (With<Ball>, Without<Obstacle>, Without<PowerUp>)>,
    mut paddles: Query<(&mut Transform, &mut Paddle, &mut Sprite), (With<Paddle>, Without<Ball>)>,
    mut powerups: Query<(Entity, &mut PowerUp, &Transform, &mut Sprite), (With<PowerUp>, Without<Ball>, Without<Obstacle>, Without<Paddle>)>,
    mut obstacles: Query<(Entity, &Obstacle, &mut Transform), (With<Obstacle>, Without<Ball>, Without<PowerUp>, Without<Paddle>)>,
    existing_obstacles: Query<(), With<Obstacle>>,
) {
    if session.mode == MatchMode::Networked || session.phase == MatchPhase::Finished { return; }
    let dt = time.delta_secs().min(0.05);

    if !settings.powerups_enabled {
        for (entity, _, _, _) in &mut powerups { commands.entity(entity).despawn(); }
        clock.powerup_timer = 0.0;
        clear_timed_effect(&mut clock, &mut paddles, &mut balls);
        inventory.player_one.clear();
        inventory.player_two.clear();
        inventory.ai.clear();
        pending_pickup.kind = None;
    }

    if session.phase != MatchPhase::Rally {
        if settings.powerups_enabled && settings.powerup_event_drops
            && session.rally_points > clock.last_score_event {
            clock.last_score_event = session.rally_points;
            if powerups.is_empty() {
                if spawn_random_powerup(&mut commands, &asset_server, &settings, &mut session) {
                    audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUpRespawn);
                }
                clock.powerup_timer = 0.0;
            }
        }
        return;
    }

    for slot in &mut inventory.player_one {
        slot.cooldown_remaining = (slot.cooldown_remaining - dt).max(0.0);
    }
    for slot in &mut inventory.player_two {
        slot.cooldown_remaining = (slot.cooldown_remaining - dt).max(0.0);
    }
    for slot in &mut inventory.ai {
        slot.cooldown_remaining = (slot.cooldown_remaining - dt).max(0.0);
    }

    clock.powerup_timer += dt;
    clock.obstacle_clock += dt;

    if settings.obstacles_enabled && existing_obstacles.is_empty() {
        commands.spawn((
            GameEntity,
            Obstacle { phase: 0.0, width: 16.0, height: 108.0 },
            Sprite::from_color(Color::srgb(0.97, 0.72, 0.27), Vec2::new(16.0, 108.0)),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    }
    for (entity, obstacle, mut transform) in &mut obstacles {
        if settings.obstacles_enabled {
            transform.translation.y = (clock.obstacle_clock * 1.35 + obstacle.phase).sin() * 118.0;
        } else {
            commands.entity(entity).despawn();
        }
    }

    if !settings.powerups_enabled { return; }

    if settings.powerup_event_drops && session.rally_points > clock.last_score_event {
        clock.last_score_event = session.rally_points;
        if powerups.is_empty() {
            if spawn_random_powerup(&mut commands, &asset_server, &settings, &mut session) {
                audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUpRespawn);
            }
            clock.powerup_timer = 0.0;
        }
    }

    if clock.powerup_timer >= settings.powerup_spawn_interval_secs.clamp(5.0, 30.0) && powerups.is_empty() {
        clock.powerup_timer = 0.0;
        if spawn_random_powerup(&mut commands, &asset_server, &settings, &mut session) {
            audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUpRespawn);
        }
    }

    let mut collected: Option<(Entity, Entity, PowerUpKind, Side)> = None;
    'pickups: for (powerup_entity, mut powerup, powerup_transform, mut sprite) in &mut powerups {
        powerup.age += dt;
        if !settings.reduce_motion {
            let frame_count = powerup.kind.pixel_art_frame_count().max(1);
            let frame_duration = powerup.kind.pixel_art_frame_duration().max(0.02);
            let frame = ((powerup.age / frame_duration).floor() as u8) % frame_count;
            if frame != powerup.frame_index {
                powerup.frame_index = frame;
                let path = format!("powerups/{}/frame_{}.png", powerup.kind.pixel_art_key(), frame);
                sprite.image = asset_server.load(path);
            }
        }
        if powerup.age > PICKUP_LIFETIME_SECONDS {
            commands.entity(powerup_entity).despawn();
            continue;
        }
        let pickup_position = powerup_transform.translation.truncate();
        for (ball_entity, ball_transform, ball) in &mut balls {
            if pickup_position.distance(ball_transform.translation.truncate()) < 24.0 {
                let owner = ball.last_hit_by;
                collected = Some((powerup_entity, ball_entity, powerup.kind, owner));
                break 'pickups;
            }
        }
    }

    if let Some((pickup_entity, ball_entity, kind, owner)) = collected {
        let kind_enabled = match kind {
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
        if kind_enabled {
            
            
            
            
            
            let manual = session.mode == MatchMode::LocalVersus
                || kind.requires_manual_activation()
                || settings.manual_powerup_activation;
            if manual {
                
                
                if inventory_has_capacity(&inventory, owner, settings.inventory_slots) {
                    inventory.for_side_mut(owner).push(PowerUpSlot::new(kind, powerup_max_charges(&settings, kind)));
                    commands.entity(pickup_entity).despawn();
                    audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUpCollect(kind));
                }
            } else {
                commands.entity(pickup_entity).despawn();
                apply_powerup(
                    kind,
                    owner,
                    Some(ball_entity),
                    &mut commands,
                    &settings,
                    &mut clock,
                    &mut session,
                    &mut paddles,
                    &mut balls,
                );
                audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::PowerUpCollect(kind));
            }
        }
    }

    let previous_effect_count = clock.effects.len();
    for effect in &mut clock.effects {
        effect.remaining = (effect.remaining - dt).max(0.0);
    }
    clock.effects.retain(|effect| effect.remaining > 0.0);
    if clock.effects.len() != previous_effect_count || !clock.effects.is_empty() {
        recalculate_effects(&mut clock, &mut paddles, &mut balls);
    }
}

