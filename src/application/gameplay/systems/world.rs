

use bevy::prelude::*;
use bevy::camera::ScalingMode;
use bevy::text::TextBounds;
use crate::{
    game::components::*,
    localization::tr,
    persistence::Settings,
};

pub fn setup_world(mut commands: Commands) {
    let projection = OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin { min_width: WORLD_WIDTH, min_height: WORLD_HEIGHT },
        ..OrthographicProjection::default_2d()
    };
    commands.spawn((Camera2d, Projection::Orthographic(projection)));
}

pub fn enter_playing(
    mut commands: Commands,
    settings: Res<Settings>,
    ui_font: Res<crate::ui::UiFont>,
    asset_server: Res<AssetServer>,
    mut session: ResMut<MatchSession>,
    mut mechanics: ResMut<MechanicsClock>,
    mut ai_random: ResMut<AiRandom>,
    mut inventory: ResMut<PowerUpInventory>,
    mut arena: ResMut<ArenaState>,
    open_play_duration: Res<OpenPlayDuration>,
    mut pending_pickup: ResMut<PendingPowerUp>,
    existing: Query<Entity, With<GameEntity>>,
    existing_ball: Query<(), With<Ball>>,
) {
    if existing_ball.iter().next().is_some() && !session.restart_requested {
        return; 
    }
    for entity in &existing { commands.entity(entity).despawn(); }

    session.player_score = 0;
    session.ai_score = 0;
    session.winning_score = settings.winning_score.max(1);
    session.active = true;
    session.phase = MatchPhase::Rally;
    session.phase_timer = 0.0;
    session.last_scorer = None;
    session.rally_points = 0;
    session.result_saved = false;
    session.restart_requested = false;
    session.open_play_remaining = open_play_duration.total_seconds().max(1.0);
    session.open_play_sudden_death = false;
    let mut arena_state = *arena;
    let arena_roll = session.random_unit();
    arena_state.activate(arena_roll);
    *arena = arena_state;

    mechanics.reset();
    ai_random.state = session.rng_state ^ 0xA11C_E5EED;
    inventory.player_one.clear();
    inventory.player_two.clear();
    inventory.ai.clear();
    pending_pickup.kind = None;
    pending_pickup.owner = Side::Player;

    let (arena_background, frame, arena_secondary) = ArenaState::palette(arena_state.active);
    commands.insert_resource(ClearColor(arena_background));
    spawn_arena_backdrop(&mut commands, arena_state.active, frame, arena_secondary);
    spawn_rect(&mut commands, Vec2::new(900.0, 4.0), Vec3::new(0.0, FIELD_HALF_H + 10.0, 1.0), frame);
    spawn_rect(&mut commands, Vec2::new(900.0, 4.0), Vec3::new(0.0, -FIELD_HALF_H - 10.0, 1.0), frame);
    spawn_rect(&mut commands, Vec2::new(4.0, 480.0), Vec3::new(-FIELD_HALF_W - 12.0, 0.0, 1.0), frame);
    spawn_rect(&mut commands, Vec2::new(4.0, 480.0), Vec3::new(FIELD_HALF_W + 12.0, 0.0, 1.0), frame);
    for y in (-200..=200).step_by(34) {
        spawn_rect(&mut commands, Vec2::new(4.0, 17.0), Vec3::new(0.0, y as f32, 1.0), arena_secondary);
    }

    commands.spawn((
        GameEntity,
        Paddle { side: Side::Player, height: PADDLE_HEIGHT, base_height: PADDLE_HEIGHT },
        paddle_skin_sprite(&settings, &asset_server),
        Transform::from_xyz(PLAYER_X, 0.0, 3.0),
    ));
    match session.mode {
        MatchMode::SinglePlayer | MatchMode::OpenPlay => {
            commands.spawn((
                GameEntity,
                Paddle { side: Side::Ai, height: PADDLE_HEIGHT, base_height: PADDLE_HEIGHT },
                AiBrain { reaction_timer: 0.0, target_y: 0.0, error_offset: 0.0 },
                Sprite::from_color(Color::srgb(1.0, 0.35, 0.49), Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                Transform::from_xyz(AI_X, 0.0, 3.0),
            ));
        }
        MatchMode::LocalVersus | MatchMode::Networked => {
            commands.spawn((
                GameEntity,
                Paddle { side: Side::PlayerTwo, height: PADDLE_HEIGHT, base_height: PADDLE_HEIGHT },
                paddle_skin_sprite(&settings, &asset_server),
                Transform::from_xyz(AI_X, 0.0, 3.0),
            ));
        }
    }
    commands.spawn((
        GameEntity,
        Ball {
            velocity: Vec2::new(BALL_START_SPEED, 90.0),
            curve: 0.0,
            radius: BALL_SIZE * 0.5,
            rally_speed: BALL_START_SPEED,
            last_hit_by: Side::Player,
            is_primary: true,
        },
        ball_skin_sprite(&settings, &asset_server),
        Transform::from_xyz(0.0, 0.0, 5.0),
    ));
    if settings.obstacles_enabled {
        commands.spawn((
            GameEntity,
            Obstacle { phase: 0.0, width: 16.0, height: 108.0 },
            Sprite::from_color(Color::srgb(0.97, 0.72, 0.27), Vec2::new(16.0, 108.0)),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));
    }

    commands.spawn((
        GameEntity,
        HudScore,
        Text2d::new("00     :     00"),
        TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(42.0), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(Color::srgb(0.93, 0.96, 1.0)),
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0.0, FIELD_HALF_H - 34.0, 8.0),
    ));
    commands.spawn((
        GameEntity,
        HudStatus,
        Text2d::new(tr(settings.language, "FIRST TO WIN")),
        TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(15.0), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(Color::srgb(0.36, 0.92, 0.82)),
        TextLayout::new(Justify::Center, bevy::text::LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(850.0, 38.0)),
        Transform::from_xyz(0.0, -FIELD_HALF_H - 27.0, 8.0),
    ));

    {
        let capacity = settings.inventory_slots.clamp(1, 5) as usize;
        let right_side = if matches!(session.mode, MatchMode::SinglePlayer | MatchMode::OpenPlay) { Side::Ai } else { Side::PlayerTwo };
        for side in [Side::Player, right_side] {
            commands.spawn((
                GameEntity,
                HudSideLabel(side),
                Text2d::new(match side { Side::Player => "P1", Side::Ai => "AI", Side::PlayerTwo => "P2" }),
                TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(10.0), font_smoothing: FontSmoothing::None, ..default() },
                TextColor(Color::srgb(0.34, 0.95, 0.88)),
                TextLayout::justify(Justify::Center),
                Transform::from_xyz(if side == Side::Player { -330.0 } else { 330.0 }, FIELD_HALF_H - 26.0, 9.0),
            ));
            for index in 0..capacity {
                let cluster_half_width = (capacity.saturating_sub(1) as f32 * 25.0) * 0.5;
                let x = if side == Side::Player { -330.0 - cluster_half_width + index as f32 * 25.0 }
                    else { 330.0 + cluster_half_width - index as f32 * 25.0 };
                let mut sprite = Sprite::from_image(asset_server.load("powerups/slot_empty.png"));
                sprite.custom_size = Some(Vec2::splat(22.0));
                commands.spawn((
                    GameEntity,
                    PowerUpHudSlot { side, index, displayed_kind: None },
                    sprite,
                    Transform::from_xyz(x, FIELD_HALF_H - 55.0, 9.0),
                ));
                let key = match side {
                    Side::Ai => "AI".to_owned(),
                    Side::Player => settings.controls.binding(2 + index).map(compact_key_label).unwrap_or_else(|| "?".to_owned()),
                    Side::PlayerTwo => settings.controls.binding(9 + index).map(compact_key_label).unwrap_or_else(|| "?".to_owned()),
                };
                commands.spawn((
                    GameEntity,
                    HudInventoryKeyHint { side, index },
                    Text2d::new(key),
                    TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(7.5), font_smoothing: FontSmoothing::None, ..default() },
                    TextColor(Color::srgb(0.58, 0.78, 0.9)),
                    TextLayout::justify(Justify::Center),
                    Transform::from_xyz(x, FIELD_HALF_H - 73.0, 9.0),
                ));
            }
        }
    }
}

fn compact_key_label(key: &str) -> String {
    match key {
        "ArrowUp" => "UP".into(), "ArrowDown" => "DN".into(),
        "ArrowLeft" => "LT".into(), "ArrowRight" => "RT".into(),
        "Space" => "SPC".into(), "Enter" => "ENT".into(),
        "Escape" => "ESC".into(), "Backspace" => "BKSP".into(),
        value if value.starts_with("Digit") => value.trim_start_matches("Digit").to_owned(),
        value if value.starts_with("Key") => value.trim_start_matches("Key").to_owned(),
        value => value.to_owned(),
    }
}

pub(crate) fn spawn_storm_burst(commands: &mut Commands, center: Vec3) {
    for index in 0..12 {
        let angle = std::f32::consts::TAU * index as f32 / 12.0;
        let direction = Vec2::from_angle(angle);
        let speed = 85.0 + (index % 3) as f32 * 28.0;
        let color = if index % 2 == 0 { Color::srgb(1.0, 0.48, 0.12) } else { Color::srgb(1.0, 0.86, 0.32) };
        commands.spawn((
            GameEntity,
            CelebrationParticle { velocity: direction * speed, remaining: 0.42 },
            Sprite::from_color(color, Vec2::splat(if index % 3 == 0 { 5.0 } else { 3.0 })),
            Transform::from_translation(center + Vec3::new(0.0, 0.0, 1.0)),
        ));
    }
}

pub(super) fn spawn_score_burst(commands: &mut Commands, scorer: Side) {
    let center_x = if scorer == Side::Player { 300.0 } else { -300.0 };
    let center = Vec3::new(center_x, 0.0, 12.0);
    for index in 0..18 {
        let angle = std::f32::consts::TAU * index as f32 / 18.0;
        let direction = Vec2::from_angle(angle);
        let speed = 105.0 + (index % 4) as f32 * 24.0;
        let color = if index % 3 == 0 { Color::srgb(1.0, 0.77, 0.34) }
            else if index % 3 == 1 { Color::srgb(0.34, 0.95, 0.88) }
            else { Color::srgb(1.0, 0.43, 0.57) };
        commands.spawn((
            GameEntity,
            CelebrationParticle { velocity: direction * speed, remaining: 0.72 },
            Sprite::from_color(color, Vec2::splat(if index % 2 == 0 { 6.0 } else { 4.0 })),
            Transform::from_translation(center),
        ));
    }
}

pub fn animate_celebration(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<Settings>,
    mut particles: Query<(Entity, &mut Transform, &mut CelebrationParticle)>,
) {
    if settings.reduce_motion {
        for (entity, _, _) in &mut particles { commands.entity(entity).despawn(); }
        return;
    }
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut particle) in &mut particles {
        particle.remaining -= dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity *= 0.96;
        let scale = (particle.remaining / 0.72).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(scale.max(0.05));
        if particle.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}


pub fn paddle_skin_asset_path(settings: &Settings) -> Option<String> {
    settings.paddle_custom_image.clone().or_else(|| match settings.paddle_skin {
        10 => Some("skins/paddle_tiger.png".to_owned()),
        11 => Some("skins/paddle_sport_stripes.png".to_owned()),
        12 => Some("skins/paddle_circuit.png".to_owned()),
        _ => None,
    })
}


pub fn ball_skin_asset_path(settings: &Settings) -> Option<String> {
    settings.ball_custom_image.clone().or_else(|| match settings.ball_skin {
        10 => Some("skins/ball_neon_orbit.png".to_owned()),
        11 => Some("skins/ball_magma.png".to_owned()),
        12 => Some("skins/ball_checker.png".to_owned()),
        _ => None,
    })
}

pub fn paddle_skin_sprite(settings: &Settings, asset_server: &AssetServer) -> Sprite {
    if let Some(path) = paddle_skin_asset_path(settings) {
        let mut sprite = Sprite::from_image(asset_server.load(path));
        sprite.custom_size = Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT));
        sprite
    } else {
        Sprite::from_color(
            paddle_color_with_shade(settings.paddle_skin, settings.paddle_shade),
            Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        )
    }
}

pub fn ball_skin_sprite(settings: &Settings, asset_server: &AssetServer) -> Sprite {
    if let Some(path) = ball_skin_asset_path(settings) {
        let mut sprite = Sprite::from_image(asset_server.load(path));
        sprite.custom_size = Some(Vec2::splat(BALL_SIZE));
        sprite
    } else {
        Sprite::from_color(ball_color_with_shade(settings.ball_skin, settings.ball_shade), Vec2::splat(BALL_SIZE))
    }
}


fn spawn_arena_backdrop(commands: &mut Commands, index: usize, accent: Color, _secondary: Color) {
    draw_arena_backdrop(commands, index, accent, 1.0, Vec2::ZERO, 0.0, false);
}

pub(crate) fn spawn_arena_preview_backdrop(
    commands: &mut Commands,
    index: usize,
    accent: Color,
    scale: f32,
    offset: Vec2,
    z_offset: f32,
) {
    draw_arena_backdrop(commands, index, accent, scale, offset, z_offset, true);
}

fn draw_arena_backdrop(
    commands: &mut Commands,
    index: usize,
    accent: Color,
    scale: f32,
    offset: Vec2,
    z_offset: f32,
    preview: bool,
) {
    let color = match index.min(19) {
        0 => [0.10, 0.95, 0.88], 1 => [1.0, 0.69, 0.25], 2 => [0.40, 0.56, 1.0],
        3 => [1.0, 0.42, 0.68], 4 => [0.43, 0.78, 1.0], 5 => [0.40, 0.85, 1.0],
        6 => [1.0, 0.45, 0.22], 7 => [0.25, 0.90, 0.87], 8 => [0.74, 0.94, 1.0],
        9 => [1.0, 0.32, 0.12], 10 => [0.35, 0.92, 0.48], 11 => [1.0, 0.79, 0.35],
        12 => [0.30, 0.92, 1.0], 13 => [0.76, 0.55, 1.0], 14 => [0.92, 0.69, 0.36],
        15 => [0.98, 0.77, 0.38], 16 => [0.52, 0.88, 1.0], 17 => [0.72, 0.58, 1.0],
        18 => [0.35, 0.91, 1.0], _ => [0.84, 0.55, 1.0],
    };
    let glow = Color::srgba(color[0], color[1], color[2], 0.16);
    let soft = Color::srgba(color[0] * 0.56, color[1] * 0.56, color[2] * 0.56, 0.20);
    let scale = scale.clamp(0.05, 1.0);
    let mut rect = |size: Vec2, x: f32, y: f32, z: f32, tint: Color| {
        let size = size * scale;
        let position = Vec3::new(x * scale + offset.x, y * scale + offset.y, z + z_offset);
        if preview {
            commands.spawn((
                ScreenUi,
                ArenaPreviewVisual,
                Sprite::from_color(tint, size),
                Transform::from_translation(position),
            ));
        } else {
            spawn_rect(commands, size, position, tint);
        }
    };
    match index.min(19) {
        0 => {
            for x in (-420..=420).step_by(60) { rect(Vec2::new(1.5, 470.0), x as f32, 0.0, 0.15, glow); }
            for y in (-210..=210).step_by(45) { rect(Vec2::new(880.0, 1.5), 0.0, y as f32, 0.15, glow); }
        }
        1 => {
            for (i, y) in [-190.0, -168.0, -146.0, 146.0, 168.0, 190.0].into_iter().enumerate() {
                rect(Vec2::new(850.0 - i as f32 * 28.0, 2.0), 0.0, y, 0.15, if i % 2 == 0 { glow } else { soft });
            }
            for x in [-390.0, -360.0, 360.0, 390.0] { rect(Vec2::new(3.0, 360.0), x, 0.0, 0.15, soft); }
        }
        2 => {
            for (i, x) in (-400..=400).step_by(100).enumerate() {
                let h = 105.0 + (i % 4) as f32 * 28.0;
                rect(Vec2::new(58.0, h), x as f32, -238.0 + h * 0.5, 0.15, soft);
                for row in 0..4 { rect(Vec2::new(8.0, 5.0), x as f32, -220.0 + h * 0.5 + row as f32 * 18.0, 0.16, glow); }
            }
        }
        3 => {
            rect(Vec2::new(110.0, 76.0), 0.0, 175.0, 0.15, soft);
            for (i, y) in [126.0, 110.0, 92.0, 72.0, 48.0].into_iter().enumerate() {
                rect(Vec2::new(190.0 + i as f32 * 42.0, 3.0), 0.0, y, 0.16, glow);
            }
            rect(Vec2::new(900.0, 2.0), 0.0, -190.0, 0.15, soft);
        }
        4 | 19 => {
            for i in 0..88 {
                let x = ((i * 97) % 860) as f32 - 430.0;
                let y = ((i * 61) % 450) as f32 - 225.0;
                let size = if i % 7 == 0 { 4.0 } else { 2.0 };
                rect(Vec2::splat(size), x, y, 0.15, if i % 3 == 0 { glow } else { soft });
            }
            if index == 19 {
                for i in 0..5 { rect(Vec2::new(280.0 - i as f32 * 34.0, 3.0), 0.0, 120.0 - i as f32 * 22.0, 0.16, glow); }
            }
        }
        5 => {
            for x in [-360.0, -250.0, 250.0, 360.0] { rect(Vec2::new(74.0, 24.0), x, -202.0, 0.15, soft); }
            for (i, x) in [-330.0, -110.0, 110.0, 330.0].into_iter().enumerate() {
                rect(Vec2::new(3.0, 84.0 + (i % 2) as f32 * 24.0), x, -186.0, 0.16, glow);
            }
        }
        6 | 9 => {
            for (i, y) in [-215.0, -198.0, -178.0, -154.0].into_iter().enumerate() {
                rect(Vec2::new(900.0 - i as f32 * 42.0, 4.0), 0.0, y, 0.15, if i % 2 == 0 { glow } else { soft });
            }
            for i in 0..28 { rect(Vec2::splat(3.0), ((i * 67) % 850) as f32 - 425.0, -130.0 + ((i * 29) % 100) as f32, 0.16, glow); }
        }
        7 => {
            for (i, y) in [-205.0, -170.0, 170.0, 205.0].into_iter().enumerate() {
                rect(Vec2::new(900.0, 2.0), 0.0, y, 0.15, if i % 2 == 0 { glow } else { soft });
            }
            for i in 0..34 { rect(Vec2::new(3.0, 3.0), ((i * 53) % 850) as f32 - 425.0, ((i * 79) % 410) as f32 - 205.0, 0.16, glow); }
        }
        8 => {
            for x in (-420_i32..=420_i32).step_by(70) {
                rect(Vec2::new(3.0, 40.0 + (x.abs() % 4) as f32 * 13.0), x as f32, 210.0, 0.15, glow);
                rect(Vec2::new(3.0, 32.0 + (x.abs() % 3) as f32 * 11.0), x as f32, -210.0, 0.15, soft);
            }
        }
        10 => {
            for x in [-420.0, -365.0, -310.0, 310.0, 365.0, 420.0] {
                rect(Vec2::new(4.0, 480.0), x, 0.0, 0.15, soft);
                for y in [-160.0, -60.0, 40.0, 140.0] { rect(Vec2::new(22.0, 6.0), x + 12.0, y, 0.16, glow); }
            }
        }
        11 => {
            for x in [-390.0, -300.0, -210.0, 210.0, 300.0, 390.0] {
                rect(Vec2::new(26.0, 330.0), x, 0.0, 0.15, soft);
                rect(Vec2::new(42.0, 8.0), x, 166.0, 0.16, glow);
                rect(Vec2::new(42.0, 8.0), x, -166.0, 0.16, glow);
            }
        }
        12 => {
            for x in (-400..=400).step_by(120) {
                rect(Vec2::new(28.0, 2.0), x as f32, -150.0, 0.15, glow);
                rect(Vec2::new(2.0, 84.0), x as f32, -108.0, 0.15, glow);
                rect(Vec2::new(26.0, 2.0), x as f32 + 13.0, -66.0, 0.15, glow);
                rect(Vec2::splat(7.0), x as f32, -150.0, 0.16, accent);
            }
        }
        13 => {
            for size in [840.0, 720.0, 600.0, 480.0] {
                rect(Vec2::new(size, 1.5), 0.0, size * 0.24, 0.15, glow);
                rect(Vec2::new(size, 1.5), 0.0, -size * 0.24, 0.15, glow);
                rect(Vec2::new(1.5, size * 0.48), -size * 0.5, 0.0, 0.15, soft);
                rect(Vec2::new(1.5, size * 0.48), size * 0.5, 0.0, 0.15, soft);
            }
        }
        14 => {
            for x in [-410.0, -340.0, 340.0, 410.0] {
                rect(Vec2::new(24.0, 480.0), x, 0.0, 0.15, soft);
                rect(Vec2::new(42.0, 5.0), x, 120.0, 0.16, glow);
                rect(Vec2::new(42.0, 5.0), x, -120.0, 0.16, glow);
            }
            rect(Vec2::new(900.0, 3.0), 0.0, 205.0, 0.15, glow);
            rect(Vec2::new(900.0, 3.0), 0.0, -205.0, 0.15, glow);
        }
        15 => {
            for (i, y) in [-205.0, -190.0, -172.0, -150.0, -126.0].into_iter().enumerate() {
                rect(Vec2::new(900.0 - i as f32 * 82.0, 3.0), 0.0, y, 0.15, if i % 2 == 0 { glow } else { soft });
            }
            rect(Vec2::new(72.0, 72.0), 340.0, 170.0, 0.15, glow);
        }
        16 => {
            for (x, y, w) in [(-350.0, 180.0, 110.0), (-290.0, 188.0, 75.0), (330.0, -180.0, 130.0), (390.0, -172.0, 82.0)] {
                rect(Vec2::new(w, 13.0), x, y, 0.15, glow);
                rect(Vec2::new(w * 0.65, 9.0), x + 14.0, y - 11.0, 0.15, soft);
            }
        }
        17 => {
            for (i, x) in [-360.0, -240.0, 240.0, 360.0].into_iter().enumerate() {
                let h = 120.0 + i as f32 * 18.0;
                rect(Vec2::new(22.0, h), x, 0.0, 0.15, soft);
                rect(Vec2::new(9.0, h * 0.72), x + 19.0, 0.0, 0.16, glow);
            }
        }
        18 => {
            for i in 0..9 {
                let x = -400.0 + i as f32 * 100.0;
                rect(Vec2::new(4.0, 44.0), x, 150.0 - (i % 3) as f32 * 34.0, 0.15, glow);
                rect(Vec2::new(4.0, 38.0), x + 16.0, 115.0 - (i % 3) as f32 * 34.0, 0.15, glow);
                rect(Vec2::new(4.0, 30.0), x + 31.0, 82.0 - (i % 3) as f32 * 34.0, 0.15, soft);
            }
        }
        _ => {}
    }
}

fn spawn_rect(commands: &mut Commands, size: Vec2, position: Vec3, color: Color) {
    commands.spawn((GameEntity, Sprite::from_color(color, size), Transform::from_translation(position)));
}

pub fn paddle_color_with_shade(skin: u8, shade: i8) -> Color {
    const COLORS: [[f32; 3]; 10] = [
        [0.25, 0.96, 0.84], [1.00, 0.83, 0.31], [0.62, 0.55, 1.00],
        [1.00, 0.48, 0.72], [0.35, 0.82, 0.42], [1.00, 0.38, 0.24],
        [0.35, 0.62, 1.00], [0.95, 0.95, 0.95], [0.88, 0.48, 0.20],
        [0.72, 0.86, 0.95],
    ];
    shade_palette_color(COLORS[(skin as usize) % COLORS.len()], shade)
}

pub fn ball_color_with_shade(skin: u8, shade: i8) -> Color {
    const COLORS: [[f32; 3]; 10] = [
        [1.00, 0.94, 0.67], [1.00, 0.42, 0.27], [0.46, 0.93, 1.00],
        [0.85, 0.59, 1.00], [0.49, 1.00, 0.53], [1.00, 0.72, 0.27],
        [0.38, 0.66, 1.00], [0.97, 0.97, 0.97], [0.82, 0.50, 0.27],
        [0.70, 0.91, 1.00],
    ];
    shade_palette_color(COLORS[(skin as usize) % COLORS.len()], shade)
}

fn shade_palette_color(rgb: [f32; 3], shade: i8) -> Color {
    let factor = (1.0 + shade.clamp(-4, 4) as f32 * 0.075).clamp(0.70, 1.30);
    Color::srgb(
        (rgb[0] * factor).clamp(0.0, 1.0),
        (rgb[1] * factor).clamp(0.0, 1.0),
        (rgb[2] * factor).clamp(0.0, 1.0),
    )
}
