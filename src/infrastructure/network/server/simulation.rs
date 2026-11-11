

use bevy::math::Vec2;
use crate::network::protocol::*;

const WORLD_HALF_WIDTH: f32 = 440.0;
const WORLD_HALF_HEIGHT: f32 = 228.0;
const PADDLE_X: f32 = 400.0;
const BALL_RADIUS: f32 = 6.5;


pub(super) use crate::network::protocol::wire_code::{
    effect as effect_code,
    power_up as power_up_code,
    special_ball as special_ball_code,
};

#[derive(Clone)]
pub(super) struct NetSlot {
    pub(super) kind: u8, 
    pub(super) charges: u8,
    pub(super) cooldown: f32,
}

#[derive(Clone)]
pub(super) struct NetEffect {
    pub(super) kind: u8,
    pub(super) team: u8,
    pub(super) remaining: f32,
    pub(super) magnitude: f32,
}

#[derive(Clone)]
pub(super) struct SimBall {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) vx: f32,
    pub(super) vy: f32,
    pub(super) radius: f32,
    pub(super) last_hit_team: u8,
    pub(super) primary: bool,
    pub(super) special_kind: u8, 
    pub(super) special_owner: u8,
    pub(super) special_hits: u8,
    pub(super) attached: bool,
    pub(super) returning: bool,
    pub(super) throws_left: u8,
    pub(super) offset_y: f32,
}

#[derive(Clone)]
pub(super) struct SimPickup {
    pub(super) id: u64,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) kind: u8,
}

pub(super) struct GameState {
    pub(super) tick: u64,
    pub(super) scores: [u32; 2],
    pub(super) paddle_y: [f32; 2],
    pub(super) paddle_height: [f32; 2],
    pub(super) balls: Vec<SimBall>,
    pub(super) pickups: Vec<SimPickup>,
    pub(super) inventories: [Vec<NetSlot>; 2],
    pub(super) effects: Vec<NetEffect>,
    pub(super) rally_timer: f32,
    pub(super) pickup_timer: f32,
    pub(super) random_state: u64,
    powerup_event_sequence: u64,
    powerup_events: Vec<PowerUpEventSnapshot>,
    pub(super) finished: bool,
    pub(super) winning_score: u32,
    settings: NetworkGameSettings,
}

pub(super) fn register_storm_block(ball: &mut SimBall, defending_team: u8) -> bool {
    
    
    if defending_team != ball.special_owner {
        ball.special_hits = ball.special_hits.saturating_sub(1);
    }
    if ball.special_hits == 0 { return true; }
    if defending_team == 0 {
        ball.x = -PADDLE_X + 13.0;
        ball.vx = ball.vx.abs();
    } else {
        ball.x = PADDLE_X - 13.0;
        ball.vx = -ball.vx.abs();
    }
    ball.last_hit_team = ball.special_owner;
    false
}

impl GameState {

    pub(super) fn with_settings(winning_score: u32, settings: NetworkGameSettings) -> Self {
        Self {
            tick: 0,
            scores: [0, 0],
            paddle_y: [0.0, 0.0],
            paddle_height: [88.0, 88.0],
            balls: vec![SimBall {
                x: 0.0, y: 0.0, vx: 365.0, vy: 90.0, radius: BALL_RADIUS,
                last_hit_team: 0, primary: true, special_kind: special_ball_code::NORMAL, special_owner: 0,
                special_hits: 0, attached: false, returning: false, throws_left: 0, offset_y: 0.0,
            }],
            pickups: Vec::new(),
            inventories: [Vec::new(), Vec::new()],
            effects: Vec::new(),
            rally_timer: 0.0,
            pickup_timer: settings.powerup_spawn_interval_secs.clamp(5.0, 30.0),
            random_state: 0xA11C_E5EED_5EED,
            powerup_event_sequence: 0,
            powerup_events: Vec::new(),
            finished: false,
            winning_score: winning_score.clamp(1, 99),
            settings,
        }
    }

    pub(super) fn random_unit(&mut self) -> f32 {
        crate::random::unit_f32(&mut self.random_state)
    }

    pub(super) fn speed_multiplier(&self) -> f32 {
        self.effects.iter().filter(|effect| effect.kind <= effect_code::SLOW)
            .fold(1.0, |value, effect| value * effect.magnitude).clamp(0.35, 2.5)
    }

    pub(super) fn team_paddle_height(&self, team: u8) -> f32 {
        let enlarge = self.effects.iter().filter(|effect| effect.kind == effect_code::ENLARGE && effect.team == team)
            .map(|effect| effect.magnitude).fold(1.0_f32, f32::max);
        let fire_shrink = self.effects.iter().filter(|effect| effect.kind == effect_code::FIREBALL_SHRINK && effect.team != team)
            .map(|effect| effect.magnitude).fold(1.0_f32, f32::min);
        (88.0 * enlarge * fire_shrink).clamp(42.0, 180.0)
    }

    pub(super) fn team_frozen(&self, team: u8) -> bool {
        self.effects.iter().any(|effect| effect.kind == effect_code::SNOWBALL_FREEZE && effect.team != team && effect.remaining > 0.0)
    }

    pub(super) fn snapshot(&self, active_team: [String; 2]) -> MatchSnapshot {
        MatchSnapshot {
            server_tick: self.tick,
            scores: self.scores,
            winning_score: self.winning_score,
            paddle_y: self.paddle_y,
            paddle_height: self.paddle_height,
            balls: self.balls.iter().map(|ball| BallSnapshot {
                x: ball.x, y: ball.y, vx: ball.vx, vy: ball.vy, radius: ball.radius,
                last_hit_team: ball.last_hit_team, primary: ball.primary,
                special_kind: ball.special_kind, special_owner: ball.special_owner,
                special_hits: ball.special_hits, attached: ball.attached,
                returning: ball.returning, throws_left: ball.throws_left, offset_y: ball.offset_y,
            }).collect(),
            pickups: self.pickups.iter().map(|pickup| PickupSnapshot {
                id: pickup.id, x: pickup.x, y: pickup.y, kind: pickup.kind,
            }).collect(),
            powerup_events: self.powerup_events.clone(),
            inventories: [0, 1].map(|team| self.inventories[team].iter().map(|slot| PowerUpSlotSnapshot {
                kind: slot.kind, charges: slot.charges, cooldown_remaining: slot.cooldown,
            }).collect()),
            effects: self.effects.iter().map(|effect| EffectSnapshot {
                kind: effect.kind, team: effect.team, remaining: effect.remaining, magnitude: effect.magnitude,
            }).collect(),
            phase: if self.finished { "finished".into() } else if self.rally_timer > 0.0 { "point_pause".into() } else { "rally".into() },
            active_team,
        }
    }

    fn item_settings(&self, kind: u8) -> Option<&NetworkPowerUpItemSettings> {
        match kind {
            power_up_code::FIREBALL => Some(&self.settings.fireball_powerup),
            power_up_code::SNOWBALL => Some(&self.settings.snowball_powerup),
            power_up_code::REVERSAL_BALL => Some(&self.settings.reversal_powerup),
            power_up_code::STICKY_BALL => Some(&self.settings.sticky_ball_powerup),
            power_up_code::BALL_STORM => Some(&self.settings.ball_storm_powerup),
            power_up_code::BOOMERANG_BALL => Some(&self.settings.boomerang_powerup),
            power_up_code::MAGNETIC_FIELD => Some(&self.settings.magnetic_field_powerup),
            power_up_code::TELEPORT_BALL => Some(&self.settings.teleport_ball_powerup),
            power_up_code::ENERGY_SHIELD => Some(&self.settings.energy_shield_powerup),
            power_up_code::OVERDRIVE => Some(&self.settings.overdrive_powerup),
            power_up_code::PHANTOM_BALL => Some(&self.settings.phantom_ball_powerup),
            _ => None,
        }
    }

    fn powerup_enabled(&self, kind: u8) -> bool {
        match kind {
            power_up_code::SPEED => self.settings.speed_powerup.enabled,
            power_up_code::SLOW => self.settings.slow_powerup.enabled,
            power_up_code::ENLARGE => self.settings.enlarge_powerup.enabled,
            power_up_code::DOUBLE_BALL => self.settings.double_ball_enabled,
            _ => self.item_settings(kind).is_some_and(|item| item.enabled),
        }
    }

    fn powerup_spawn_weight(&self, kind: u8) -> f32 {
        match kind {
            power_up_code::SPEED => self.settings.speed_powerup.spawn_weight,
            power_up_code::SLOW => self.settings.slow_powerup.spawn_weight,
            power_up_code::ENLARGE => self.settings.enlarge_powerup.spawn_weight,
            power_up_code::DOUBLE_BALL => self.settings.double_ball_spawn_probability,
            _ => self.item_settings(kind).map(|item| item.spawn_weight).unwrap_or(0.0),
        }
    }

    fn powerup_charges(&self, kind: u8) -> u8 {
        match kind {
            power_up_code::SPEED => self.settings.speed_powerup.max_charges,
            power_up_code::SLOW => self.settings.slow_powerup.max_charges,
            power_up_code::ENLARGE => self.settings.enlarge_powerup.max_charges,
            power_up_code::DOUBLE_BALL => self.settings.double_ball_max_charges,
            _ => self.item_settings(kind).map(|item| item.max_charges).unwrap_or(1),
        }.clamp(1, 5)
    }

    fn powerup_cooldown(&self, kind: u8) -> f32 {
        match kind {
            power_up_code::SPEED => self.settings.speed_powerup.cooldown_secs,
            power_up_code::SLOW => self.settings.slow_powerup.cooldown_secs,
            power_up_code::ENLARGE => self.settings.enlarge_powerup.cooldown_secs,
            power_up_code::DOUBLE_BALL => self.settings.double_ball_cooldown_secs,
            _ => self.item_settings(kind).map(|item| item.cooldown_secs).unwrap_or(0.0),
        }.clamp(0.0, 30.0)
    }

    fn timed_powerup_settings(&self, kind: u8) -> Option<&NetworkTimedPowerUpSettings> {
        match kind {
            power_up_code::SPEED => Some(&self.settings.speed_powerup),
            power_up_code::SLOW => Some(&self.settings.slow_powerup),
            power_up_code::ENLARGE => Some(&self.settings.enlarge_powerup),
            _ => None,
        }
    }

    fn requires_inventory(&self, kind: u8) -> bool {
        self.settings.manual_powerup_activation
            || matches!(kind, power_up_code::STICKY_BALL | power_up_code::BOOMERANG_BALL
                | power_up_code::MAGNETIC_FIELD | power_up_code::TELEPORT_BALL
                | power_up_code::ENERGY_SHIELD | power_up_code::OVERDRIVE | power_up_code::PHANTOM_BALL)
    }

    fn spawn_pickup(&mut self) {
        if !self.settings.powerups_enabled { return; }
        let choices: Vec<(u8, f32)> = (0..15_u8)
            .filter(|kind| self.powerup_enabled(*kind))
            .map(|kind| (kind, self.powerup_spawn_weight(kind).clamp(0.0, 1.0)))
            .filter(|(_, weight)| *weight > 0.0)
            .collect();
        let total_weight: f32 = choices.iter().map(|(_, weight)| *weight).sum();
        if total_weight <= f32::EPSILON { return; }

        let mut roll = self.random_unit() * total_weight;
        let mut selected_kind = choices.last().map(|(kind, _)| *kind).unwrap_or(power_up_code::SPEED);
        for (kind, weight) in choices {
            if roll <= weight {
                selected_kind = kind;
                break;
            }
            roll -= weight;
        }
        let x = self.random_unit() * 180.0 - 90.0;
        let y = self.random_unit() * 300.0 - 150.0;
        self.pickups.push(SimPickup { id: self.tick, x, y, kind: selected_kind });
        self.record_powerup_event(wire_code::power_up_event::RESPAWN, selected_kind);
    }

    fn record_powerup_event(&mut self, event: u8, kind: u8) {
        self.powerup_event_sequence = self.powerup_event_sequence.saturating_add(1);
        self.powerup_events.push(PowerUpEventSnapshot {
            id: self.powerup_event_sequence,
            event,
            kind,
        });
        
        if self.powerup_events.len() > 32 {
            self.powerup_events.remove(0);
        }
    }

    
    
    
    
    pub(super) fn release_ball(&mut self, team: u8) {
        let sticky_speed = self.settings.sticky_ball_powerup.magnitude.clamp(250.0, 700.0);
        let boomerang_speed = self.settings.boomerang_powerup.magnitude.clamp(300.0, 700.0);

        for ball in &mut self.balls {
            if !ball.attached
                || ball.special_owner != team
                || !matches!(ball.special_kind, special_ball_code::STICKY | special_ball_code::BOOMERANG)
            {
                continue;
            }

            ball.attached = false;
            ball.returning = false;
            let direction = if team == 0 { 1.0 } else { -1.0 };
            let speed = if ball.special_kind == special_ball_code::BOOMERANG {
                boomerang_speed
            } else {
                sticky_speed
            };
            ball.vx = direction * speed;
            ball.vy = (ball.offset_y * 0.35).clamp(-95.0, 95.0);
            ball.last_hit_team = team;
        }
    }

    pub(super) fn activate_powerup(&mut self, team: u8, slot_index: usize) {
        let Some(slot) = self.inventories.get(team as usize).and_then(|slots| slots.get(slot_index)) else { return; };
        if slot.cooldown > 0.0 { return; }
        let kind = slot.kind;
        if !self.powerup_enabled(kind) { return; }
        if kind == power_up_code::DOUBLE_BALL && self.balls.len() >= self.settings.max_active_balls.clamp(2, 4) as usize { return; }

        let cooldown = self.powerup_cooldown(kind);
        let Some(slot) = self.inventories.get_mut(team as usize).and_then(|slots| slots.get_mut(slot_index)) else { return; };
        slot.charges = slot.charges.saturating_sub(1);
        slot.cooldown = cooldown;
        if slot.charges == 0 { self.inventories[team as usize].remove(slot_index); }
        self.apply_powerup(team, kind, None);
    }

    fn apply_powerup(&mut self, team: u8, kind: u8, preferred_ball: Option<usize>) {
        if !self.powerup_enabled(kind) { return; }
        match kind {
            power_up_code::SPEED | power_up_code::SLOW | power_up_code::ENLARGE => {
                if let Some(config) = self.timed_powerup_settings(kind) {
                    self.push_or_refresh_effect(kind, team, config.duration_secs, config.magnitude);
                    self.recalculate_motion();
                }
            }
            power_up_code::DOUBLE_BALL => self.split_ball(),
            power_up_code::FIREBALL => {
                if let Some(index) = preferred_ball.or_else(|| self.balls.iter().position(|ball| ball.primary)) {
                    if let Some(ball) = self.balls.get_mut(index) {
                        ball.special_kind = special_ball_code::FIREBALL;
                        ball.special_owner = team;
                        ball.special_hits = 0;
                    }
                }
            }
            power_up_code::SNOWBALL => {
                let duration = self.settings.snowball_powerup.duration_secs.clamp(1.0, 15.0);
                self.push_or_refresh_effect(effect_code::SNOWBALL_FREEZE, team, duration, 0.0);
            }
            power_up_code::REVERSAL_BALL => {
                if let Some(index) = preferred_ball.or_else(|| self.balls.iter().position(|ball| ball.primary)) {
                    if let Some(ball) = self.balls.get_mut(index) {
                        ball.vx = -ball.vx;
                        ball.vy = -ball.vy;
                    }
                }
            }
            power_up_code::STICKY_BALL | power_up_code::BOOMERANG_BALL => {
                if let Some(index) = self.balls.iter().position(|ball| ball.primary) {
                    let ball = &mut self.balls[index];
                    ball.special_kind = if kind == power_up_code::STICKY_BALL { special_ball_code::STICKY } else { special_ball_code::BOOMERANG };
                    ball.special_owner = team;
                    ball.attached = true;
                    ball.returning = false;
                    ball.throws_left = if kind == power_up_code::BOOMERANG_BALL {
                        self.settings.boomerang_powerup.throws.clamp(1, 5)
                    } else { 1 };
                    ball.offset_y = 0.0;
                    ball.vx = 0.0;
                    ball.vy = 0.0;
                    ball.last_hit_team = team;
                }
            }
            power_up_code::BALL_STORM => self.spawn_ball_storm(team),
            power_up_code::MAGNETIC_FIELD => self.push_or_refresh_effect(
                effect_code::MAGNETIC_FIELD, team,
                self.settings.magnetic_field_powerup.duration_secs.clamp(1.0, 10.0),
                self.settings.magnetic_field_powerup.magnitude.clamp(0.1, 1.0),
            ),
            power_up_code::TELEPORT_BALL => {
                if let Some(index) = preferred_ball.or_else(|| self.balls.iter().position(|ball| ball.primary)) {
                    let radius = self.balls.get(index).map(|ball| ball.radius).unwrap_or(BALL_RADIUS);
                    let target_x = if team == 0 { WORLD_HALF_WIDTH * 0.48 } else { -WORLD_HALF_WIDTH * 0.48 };
                    let target_y = (self.random_unit() * (WORLD_HALF_HEIGHT - radius) * 1.7
                        - (WORLD_HALF_HEIGHT - radius) * 0.85)
                        .clamp(-WORLD_HALF_HEIGHT + radius + 2.0, WORLD_HALF_HEIGHT - radius - 2.0);
                    if let Some(ball) = self.balls.get_mut(index) {
                        ball.x = target_x;
                        ball.y = target_y;
                    }
                }
            }
            power_up_code::ENERGY_SHIELD => self.push_or_refresh_effect(
                effect_code::ENERGY_SHIELD, team,
                self.settings.energy_shield_powerup.duration_secs.clamp(1.0, 12.0), 1.0,
            ),
            power_up_code::OVERDRIVE => self.push_or_refresh_effect(
                effect_code::OVERDRIVE, team,
                self.settings.overdrive_powerup.duration_secs.clamp(1.0, 10.0),
                self.settings.overdrive_powerup.magnitude.clamp(1.1, 2.0),
            ),
            power_up_code::PHANTOM_BALL => self.push_or_refresh_effect(
                effect_code::PHANTOM_BALL, team,
                self.settings.phantom_ball_powerup.duration_secs.clamp(1.0, 10.0),
                self.settings.phantom_ball_powerup.magnitude.clamp(0.2, 0.8),
            ),
            _ => {}
        }
    }

    fn apply_fireball_damage(&mut self, owner_team: u8) {
        let duration = self.settings.fireball_powerup.duration_secs.clamp(1.0, 15.0);
        let shrink_multiplier = self.settings.fireball_powerup.magnitude.clamp(0.5, 0.95);
        self.push_or_refresh_effect(
            effect_code::FIREBALL_SHRINK,
            owner_team,
            duration,
            shrink_multiplier,
        );
        self.recalculate_motion();
    }

    fn push_or_refresh_effect(&mut self, kind: u8, team: u8, duration: f32, magnitude: f32) {
        if let Some(effect) = self.effects.iter_mut().find(|effect| effect.kind == kind && effect.team == team) {
            effect.remaining = duration;
            effect.magnitude = magnitude;
        } else {
            self.effects.push(NetEffect { kind, team, remaining: duration, magnitude });
        }
    }

    pub(super) fn collect_powerup(&mut self, ball_index: usize, team: u8, kind: u8) -> bool {
        if !self.powerups_available() || !self.powerup_enabled(kind) { return false; }
        if self.requires_inventory(kind) {
            let capacity = self.settings.inventory_slots.clamp(1, 5) as usize;
            let charges = self.powerup_charges(kind);
            let Some(inventory) = self.inventories.get_mut(team as usize) else { return false; };
            if inventory.len() >= capacity { return false; }
            inventory.push(NetSlot { kind, charges, cooldown: 0.0 });
        } else {
            self.apply_powerup(team, kind, Some(ball_index));
        }
        self.record_powerup_event(wire_code::power_up_event::COLLECT, kind);
        true
    }

    fn powerups_available(&self) -> bool {
        self.settings.powerups_enabled
    }

    pub(super) fn recalculate_motion(&mut self) {
        self.paddle_height = [self.team_paddle_height(0), self.team_paddle_height(1)];
        
        
    }

    pub(super) fn split_ball(&mut self) {
        if self.balls.len() >= self.settings.max_active_balls.clamp(2, 4) as usize { return; }
        let Some(index) = self.balls.iter().position(|ball| ball.primary) else { return; };
        let source = self.balls[index].clone();
        self.balls[index].vx = source.vx * 0.965 - source.vy * 0.24;
        self.balls[index].vy = source.vy * 0.965 + source.vx * 0.24;
        let mut second = source.clone();
        second.primary = false;
        second.vx = source.vx * 0.965 + source.vy * 0.24;
        second.vy = source.vy * 0.965 - source.vx * 0.24;
        second.y += 18.0;
        self.balls.push(second);
    }

    pub(super) fn spawn_ball_storm(&mut self, team: u8) {
        let min_balls = self.settings.ball_storm_powerup.min_projectiles.clamp(3, 5) as usize;
        let max_balls = self.settings.ball_storm_powerup.max_projectiles.clamp(min_balls as u8, 5) as usize;
        let count = min_balls + (self.random_unit() * (max_balls - min_balls + 1) as f32).floor() as usize;
        let projectile_speed = self.settings.ball_storm_powerup.magnitude.clamp(300.0, 750.0);
        let opponent_blocks = self.settings.ball_storm_powerup.opponent_blocks.clamp(1, 5);
        let direction = if team == 0 { 1.0 } else { -1.0 };
        let origin_x = if team == 0 { -PADDLE_X + 13.0 } else { PADDLE_X - 13.0 };
        let origin_y = self.paddle_y[team as usize];
        for index in 0..count {
            let spread = (index as f32 - (count as f32 - 1.0) * 0.5) * 0.19;
            let jitter = (self.random_unit() - 0.5) * 0.12;
            let angle = spread + jitter;
            self.balls.push(SimBall {
                x: origin_x, y: origin_y + spread * 34.0,
                vx: direction * angle.cos() * projectile_speed, vy: angle.sin() * projectile_speed,
                radius: BALL_RADIUS * 0.72, last_hit_team: team, primary: false,
                special_kind: special_ball_code::STORM_PROJECTILE, special_owner: team, special_hits: opponent_blocks,
                attached: false, returning: false, throws_left: 0, offset_y: 0.0,
            });
        }
    }

    pub(super) fn tick(&mut self, dt: f32, team_inputs: [Vec<i8>; 2], ai_takeover: [bool; 2]) -> Option<u8> {
        if self.finished { return None; }
        self.tick = self.tick.wrapping_add(1);
        for slots in &mut self.inventories {
            for slot in slots { slot.cooldown = (slot.cooldown - dt).max(0.0); }
        }
        let old_len = self.effects.len();
        for effect in &mut self.effects { effect.remaining = (effect.remaining - dt).max(0.0); }
        self.effects.retain(|effect| effect.remaining > 0.0);
        if old_len != self.effects.len() { self.recalculate_motion(); }

        if self.rally_timer > 0.0 {
            self.rally_timer = (self.rally_timer - dt).max(0.0);
            if self.rally_timer == 0.0 {
                let direction = if self.scores[0] > self.scores[1] { -1.0 } else { 1.0 };
                self.balls.clear();
                let vertical_velocity = (self.random_unit() - 0.5) * 180.0;
                self.balls.push(SimBall {
                    x: 0.0, y: 0.0, vx: direction * 365.0, vy: vertical_velocity,
                    radius: BALL_RADIUS, last_hit_team: if direction > 0.0 { 0 } else { 1 }, primary: true,
                    special_kind: special_ball_code::NORMAL, special_owner: 0, special_hits: 0,
                    attached: false, returning: false, throws_left: 0, offset_y: 0.0,
                });
            }
            return None;
        }

        for team in 0..2 {
            let direction = if self.team_frozen(team as u8) { 0.0 }
            else if !team_inputs[team].is_empty() {
                let sum: i32 = team_inputs[team].iter().map(|value| *value as i32).sum();
                if sum > 0 { 1.0 } else if sum < 0 { -1.0 } else { 0.0 }
            } else if ai_takeover[team] {
                let target = self.balls.iter().min_by(|a, b| {
                    (a.x.abs() + (a.y - self.paddle_y[team]).abs())
                        .total_cmp(&(b.x.abs() + (b.y - self.paddle_y[team]).abs()))
                }).map(|ball| ball.y).unwrap_or(0.0);
                if target > self.paddle_y[team] + 8.0 { 0.75 }
                else if target < self.paddle_y[team] - 8.0 { -0.75 } else { 0.0 }
            } else { 0.0 };
            let overdrive = self.effects.iter().filter(|effect| effect.kind == effect_code::OVERDRIVE && effect.team == team as u8)
                .map(|effect| effect.magnitude).fold(1.0_f32, f32::max);
            self.paddle_y[team] = (self.paddle_y[team] + direction * 440.0 * overdrive * dt)
                .clamp(-WORLD_HALF_HEIGHT + self.paddle_height[team] * 0.5, WORLD_HALF_HEIGHT - self.paddle_height[team] * 0.5);
        }

        let multiplier = self.speed_multiplier();
        let mut scored: Option<u8> = None;
        let mut removed = Vec::new();
        for index in 0..self.balls.len() {
            let mut ball = self.balls[index].clone();
            if ball.attached {
                let team = ball.special_owner as usize;
                let direction = if team == 0 { 1.0 } else { -1.0 };
                ball.x = if team == 0 { -PADDLE_X + 13.0 } else { PADDLE_X - 13.0 };
                ball.y = self.paddle_y[team] + ball.offset_y;
                ball.vx = 0.0;
                ball.vy = 0.0;
                let _ = direction;
                self.balls[index] = ball;
                continue;
            }

            if let Some(field) = self.effects.iter().find(|effect| effect.kind == effect_code::MAGNETIC_FIELD && effect.remaining > 0.0) {
                let target_x = if field.team == 0 { -PADDLE_X + 13.0 } else { PADDLE_X - 13.0 };
                let dx = target_x - ball.x;
                let dy = self.paddle_y[field.team as usize] - ball.y;
                let distance = Vec2::new(dx, dy).length();
                if distance < 205.0 && distance > 1.0 {
                    let speed = Vec2::new(ball.vx, ball.vy).length();
                    let current = Vec2::new(ball.vx, ball.vy).normalize_or_zero();
                    let desired = Vec2::new(dx, dy).normalize_or_zero();
                    let influence = (1.0 - distance / 205.0) * field.magnitude.clamp(0.1, 1.0) * 0.12;
                    let steered = current.lerp(desired, influence).normalize_or_zero() * speed;
                    ball.vx = steered.x;
                    ball.vy = steered.y;
                }
            }
            ball.x += ball.vx * dt * multiplier;
            ball.y += ball.vy * dt * multiplier;
            if ball.y > WORLD_HALF_HEIGHT - ball.radius {
                ball.y = WORLD_HALF_HEIGHT - ball.radius;
                ball.vy = -ball.vy.abs();
            } else if ball.y < -WORLD_HALF_HEIGHT + ball.radius {
                ball.y = -WORLD_HALF_HEIGHT + ball.radius;
                ball.vy = ball.vy.abs();
            }
            for shield in self.effects.iter().filter(|effect| effect.kind == effect_code::ENERGY_SHIELD && effect.remaining > 0.0) {
                let crossed = if shield.team == 0 {
                    ball.x - ball.radius < -WORLD_HALF_WIDTH && ball.vx < 0.0
                } else {
                    ball.x + ball.radius > WORLD_HALF_WIDTH && ball.vx > 0.0
                };
                if crossed {
                    ball.x = if shield.team == 0 { -WORLD_HALF_WIDTH + ball.radius + 1.0 } else { WORLD_HALF_WIDTH - ball.radius - 1.0 };
                    ball.vx = if shield.team == 0 { ball.vx.abs() } else { -ball.vx.abs() };
                    ball.last_hit_team = shield.team;
                }
            }

            let mut reattached = false;
            if ball.x < -PADDLE_X + 12.0 && ball.vx < 0.0
                && (ball.y - self.paddle_y[0]).abs() <= self.paddle_height[0] * 0.5 + ball.radius {
                if ball.special_kind == special_ball_code::STORM_PROJECTILE {
                    if register_storm_block(&mut ball, 0) { removed.push(index); continue; }
                } else {
                    if ball.special_kind == special_ball_code::FIREBALL && ball.special_owner != 0 {
                        self.apply_fireball_damage(ball.special_owner);
                        ball.special_kind = special_ball_code::NORMAL;
                    } else if ball.special_kind == special_ball_code::BOOMERANG && ball.special_owner != 0 && !ball.returning {
                        self.apply_fireball_damage(ball.special_owner);
                        ball.returning = true;
                    } else if ball.special_kind == special_ball_code::BOOMERANG && ball.special_owner == 0 && ball.returning {
                        if ball.throws_left > 1 {
                            ball.throws_left -= 1;
                            ball.attached = true;
                            ball.returning = false;
                            ball.vx = 0.0;
                            ball.vy = 0.0;
                            reattached = true;
                        } else { ball.special_kind = special_ball_code::NORMAL; }
                    }
                    if !reattached {
                        let offset = ((ball.y - self.paddle_y[0]) / (self.paddle_height[0] * 0.5)).clamp(-1.0, 1.0);
                        ball.x = -PADDLE_X + 13.0;
                        ball.vx = ball.vx.abs();
                        ball.vy += offset * 190.0;
                        ball.last_hit_team = 0;
                    }
                }
            } else if ball.x > PADDLE_X - 12.0 && ball.vx > 0.0
                && (ball.y - self.paddle_y[1]).abs() <= self.paddle_height[1] * 0.5 + ball.radius {
                if ball.special_kind == special_ball_code::STORM_PROJECTILE {
                    if register_storm_block(&mut ball, 1) { removed.push(index); continue; }
                } else {
                    if ball.special_kind == special_ball_code::FIREBALL && ball.special_owner != 1 {
                        self.apply_fireball_damage(ball.special_owner);
                        ball.special_kind = special_ball_code::NORMAL;
                    } else if ball.special_kind == special_ball_code::BOOMERANG && ball.special_owner != 1 && !ball.returning {
                        self.apply_fireball_damage(ball.special_owner);
                        ball.returning = true;
                    } else if ball.special_kind == special_ball_code::BOOMERANG && ball.special_owner == 1 && ball.returning {
                        if ball.throws_left > 1 {
                            ball.throws_left -= 1;
                            ball.attached = true;
                            ball.returning = false;
                            ball.vx = 0.0;
                            ball.vy = 0.0;
                            reattached = true;
                        } else { ball.special_kind = special_ball_code::NORMAL; }
                    }
                    if !reattached {
                        let offset = ((ball.y - self.paddle_y[1]) / (self.paddle_height[1] * 0.5)).clamp(-1.0, 1.0);
                        ball.x = PADDLE_X - 13.0;
                        ball.vx = -ball.vx.abs();
                        ball.vy += offset * 190.0;
                        ball.last_hit_team = 1;
                    }
                }
            }
            if reattached { self.balls[index] = ball; continue; }
            if removed.contains(&index) { continue; }
            self.balls[index] = ball.clone();
            if ball.x < -WORLD_HALF_WIDTH - 14.0 { scored = Some(1); break; }
            if ball.x > WORLD_HALF_WIDTH + 14.0 { scored = Some(0); break; }
        }
        if !removed.is_empty() {
            self.balls = self.balls.drain(..).enumerate().filter_map(|(index, ball)| {
                if removed.contains(&index) { None } else { Some(ball) }
            }).collect();
        }

        if let Some(winning_team) = scored {
            self.scores[winning_team as usize] += 1;
            self.rally_timer = 1.0;
            self.pickups.clear();
            self.balls.clear();
            if self.scores[winning_team as usize] >= self.winning_score {
                self.finished = true;
                return Some(winning_team);
            }
            return None;
        }

        if self.settings.powerups_enabled {
            self.pickup_timer -= dt;
            if self.pickup_timer <= 0.0 && self.pickups.is_empty() {
                self.spawn_pickup();
                self.pickup_timer = self.settings.powerup_spawn_interval_secs.clamp(5.0, 30.0);
            }
        } else {
            self.pickups.clear();
            self.inventories = [Vec::new(), Vec::new()];
            self.effects.clear();
            self.recalculate_motion();
        }
        if !self.pickups.is_empty() {
            let mut collected = None;
            'pickup: for (pickup_index, pickup) in self.pickups.iter().enumerate() {
                for (ball_index, ball) in self.balls.iter().enumerate() {
                    if (ball.x - pickup.x).hypot(ball.y - pickup.y) < 22.0 {
                        collected = Some((pickup_index, ball_index, ball.last_hit_team, pickup.kind));
                        break 'pickup;
                    }
                }
            }
            if let Some((pickup_index, ball_index, team, kind)) = collected {
                let requires_inventory = self.requires_inventory(kind);
                let capacity = self.settings.inventory_slots.clamp(1, 5) as usize;
                let inventory_full = self.inventories.get(team as usize).is_none_or(|slots| slots.len() >= capacity);
                
                
                if !requires_inventory || !inventory_full {
                    if self.collect_powerup(ball_index, team, kind) {
                        self.pickups.remove(pickup_index);
                    }
                }
            }
        }
        None
    }
}


