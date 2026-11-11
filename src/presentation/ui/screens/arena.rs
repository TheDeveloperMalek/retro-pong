use bevy::prelude::*;
use crate::{
    game::components::{
        ArenaPreviewAccent, ArenaPreviewCounter, ArenaPreviewNext, ArenaPreviewPanel,
        ArenaPreviewPrevious, ArenaPreviewSurface, ArenaPreviewTitle, ArenaPreviewVisual, ArenaState,
        FIELD_HALF_H, FIELD_HALF_W, MenuSelection, PADDLE_HEIGHT, PADDLE_WIDTH,
        ScreenUi, BALL_SIZE,
    },
    localization::tr,
    persistence::Settings,
    ui::UiFont,
};
use super::common::*;

const PREVIEW_SCALE: f32 = 0.40;
const PREVIEW_CENTER_Y: f32 = 8.0;

#[derive(Component)]
pub(crate) struct ArenaPreviewBall {
    velocity: Vec2,
}

#[derive(Component)]
pub(crate) struct ArenaPreviewPaddle {
    side: f32,
    follow_speed: f32,
}

#[derive(Component)]
pub(crate) struct ArenaPreviewBallGlow;

pub fn spawn_arena_select(
    mut commands: Commands,
    settings: Res<Settings>,
    arena: Res<ArenaState>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "SELECT ARENA", "LEFT / RIGHT TO BROWSE // ENTER TO CONFIRM", language);

    spawn_arena_label(
        &mut commands,
        font.clone(),
        ArenaPreviewTitle,
        ArenaState::name(arena.selected),
        Vec2::new(0.0, 132.0),
        13.0,
        Color::srgb(0.34, 0.95, 0.88),
    );
    spawn_arena_label(
        &mut commands,
        font.clone(),
        ArenaPreviewCounter,
        arena_counter(arena.selected),
        Vec2::new(0.0, 116.0),
        7.0,
        GOLD,
    );

    let previous = if arena.selected == 0 {
        ArenaState::name(ArenaState::RANDOM_INDEX)
    } else {
        ArenaState::name(arena.selected - 1)
    };
    let next = ArenaState::name((arena.selected + 1) % ArenaState::COUNT);
    spawn_arena_label(
        &mut commands,
        font.clone(),
        ArenaPreviewPrevious,
        format!("< {previous}"),
        Vec2::new(-230.0, PREVIEW_CENTER_Y),
        6.5,
        Color::srgb(0.48, 0.62, 0.78),
    );
    spawn_arena_label(
        &mut commands,
        font.clone(),
        ArenaPreviewNext,
        format!("{next} >"),
        Vec2::new(230.0, PREVIEW_CENTER_Y),
        6.5,
        Color::srgb(0.48, 0.62, 0.78),
    );

    spawn_text(
        &mut commands,
        font.clone(),
        tr(language, "ARENA EFFECTS ARE SYMMETRIC AND BALANCED"),
        Vec2::new(0.0, -105.0),
        6.5,
        Color::srgb(0.62, 0.72, 0.86),
    );
    let labels = [tr(language, "CONFIRM ARENA"), tr(language, "BACK")];
    spawn_menu_items(&mut commands, font.clone(), &labels, -132.0, 22.0, 8.0);
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -208.0), 7.0, DIM);
}

pub fn update_arena_preview(
    mut commands: Commands,
    arena: Res<ArenaState>,
    time: Res<Time>,
    mut rendered_index: Local<Option<usize>>,
    preview_entities: Query<Entity, With<ArenaPreviewVisual>>,
    mut titles: Query<&mut Text2d, (With<ArenaPreviewTitle>, Without<ArenaPreviewCounter>, Without<ArenaPreviewPrevious>, Without<ArenaPreviewNext>)>,
    mut counters: Query<&mut Text2d, (With<ArenaPreviewCounter>, Without<ArenaPreviewTitle>, Without<ArenaPreviewPrevious>, Without<ArenaPreviewNext>)>,
    mut previous_labels: Query<&mut Text2d, (With<ArenaPreviewPrevious>, Without<ArenaPreviewTitle>, Without<ArenaPreviewCounter>, Without<ArenaPreviewNext>)>,
    mut next_labels: Query<&mut Text2d, (With<ArenaPreviewNext>, Without<ArenaPreviewTitle>, Without<ArenaPreviewCounter>, Without<ArenaPreviewPrevious>)>,
) {
    let selected = arena.selected.min(ArenaState::RANDOM_INDEX);
    let preview_index = if selected == ArenaState::RANDOM_INDEX {
        ((time.elapsed_secs() / 2.4).floor() as usize) % ArenaState::RANDOM_INDEX
    } else {
        selected
    };

    if *rendered_index != Some(preview_index) || preview_entities.is_empty() {
        for entity in &preview_entities {
            commands.entity(entity).despawn();
        }
        spawn_arena_preview_scene(&mut commands, preview_index);
        *rendered_index = Some(preview_index);
    }

    let title = ArenaState::name(selected).to_owned();
    let counter = arena_counter(selected);
    let previous = if selected == 0 {
        ArenaState::name(ArenaState::RANDOM_INDEX)
    } else {
        ArenaState::name(selected - 1)
    };
    let next = ArenaState::name((selected + 1) % ArenaState::COUNT);

    for mut text in &mut titles {
        if text.0 != title {
            text.0 = title.clone();
        }
    }
    for mut text in &mut counters {
        if text.0 != counter {
            text.0 = counter.clone();
        }
    }
    for mut text in &mut previous_labels {
        let label = format!("< {previous}");
        if text.0 != label {
            text.0 = label;
        }
    }
    for mut text in &mut next_labels {
        let label = format!("{next} >");
        if text.0 != label {
            text.0 = label;
        }
    }
}

pub fn animate_arena_preview(
    time: Res<Time>,
    settings: Res<Settings>,
    mut balls: Query<
        (&mut Transform, &mut ArenaPreviewBall),
        (With<ArenaPreviewBall>, Without<ArenaPreviewPaddle>),
    >,
    mut glows: Query<(&mut Transform, &mut Sprite), (With<ArenaPreviewBallGlow>, Without<ArenaPreviewBall>, Without<ArenaPreviewPaddle>)>,
    mut paddles: Query<
        (&mut Transform, &ArenaPreviewPaddle),
        (With<ArenaPreviewPaddle>, Without<ArenaPreviewBall>, Without<ArenaPreviewBallGlow>),
    >,
) {
    if settings.reduce_motion {
        return;
    }

    let delta = time.delta_secs().min(0.05);
    let pulse = 1.0 + (time.elapsed_secs() * 5.0).sin() * 0.10;
    let mut target_y = 0.0;

    for (mut transform, mut ball) in &mut balls {
        transform.translation.x += ball.velocity.x * delta;
        transform.translation.y += ball.velocity.y * delta;
        if transform.translation.x >= 154.0 || transform.translation.x <= -154.0 {
            ball.velocity.x = -ball.velocity.x;
            transform.translation.x = transform.translation.x.clamp(-154.0, 154.0);
        }
        let relative_y = transform.translation.y - PREVIEW_CENTER_Y;
        if relative_y >= 86.0 || relative_y <= -86.0 {
            ball.velocity.y = -ball.velocity.y;
            transform.translation.y = PREVIEW_CENTER_Y + relative_y.clamp(-86.0, 86.0);
        }
        target_y = transform.translation.y - PREVIEW_CENTER_Y;
        for (mut glow, mut sprite) in &mut glows {
            glow.translation.x = transform.translation.x;
            glow.translation.y = transform.translation.y;
            glow.scale = Vec3::splat(pulse);
            sprite.color = Color::srgba(0.92, 0.98, 1.0, 0.16 + pulse * 0.04);
        }
    }

    for (mut transform, paddle) in &mut paddles {
        let desired_y = PREVIEW_CENTER_Y + (target_y * if paddle.side < 0.0 { 0.72 } else { 0.88 }).clamp(-50.0, 50.0);
        let difference = desired_y - transform.translation.y;
        let step = paddle.follow_speed * delta;
        transform.translation.y += difference.clamp(-step, step);
    }
}

fn spawn_arena_preview_scene(commands: &mut Commands, index: usize) {
    let (background, accent, secondary) = ArenaState::palette(index);
    let scale = PREVIEW_SCALE;
    let center_y = PREVIEW_CENTER_Y;

    spawn_preview_rect(
        commands,
        Vec2::new(560.0, 205.0),
        Vec3::new(0.0, center_y, 10.0),
        Color::srgb(0.012, 0.020, 0.040),
        ArenaPreviewPanel,
    );
    spawn_preview_rect(
        commands,
        Vec2::new(560.0, 2.0),
        Vec3::new(0.0, center_y + 101.0, 11.0),
        accent,
        ArenaPreviewAccent,
    );
    spawn_preview_rect(
        commands,
        Vec2::new(900.0 * scale, 480.0 * scale),
        Vec3::new(0.0, center_y, 12.8),
        background,
        ArenaPreviewSurface,
    );

    crate::game::systems::spawn_arena_preview_backdrop(
        commands,
        index,
        accent,
        scale,
        Vec2::new(0.0, center_y),
        13.0,
    );

    let frame = accent;
    let half_width = (FIELD_HALF_W + 12.0) * scale;
    let half_height = (FIELD_HALF_H + 10.0) * scale;
    spawn_preview_rect(
        commands,
        Vec2::new(900.0 * scale, 4.0 * scale),
        Vec3::new(0.0, center_y + half_height, 14.0),
        frame,
        ArenaPreviewSurface,
    );
    spawn_preview_rect(
        commands,
        Vec2::new(900.0 * scale, 4.0 * scale),
        Vec3::new(0.0, center_y - half_height, 14.0),
        frame,
        ArenaPreviewSurface,
    );
    spawn_preview_rect(
        commands,
        Vec2::new(4.0 * scale, 480.0 * scale),
        Vec3::new(-half_width, center_y, 14.0),
        frame,
        ArenaPreviewSurface,
    );
    spawn_preview_rect(
        commands,
        Vec2::new(4.0 * scale, 480.0 * scale),
        Vec3::new(half_width, center_y, 14.0),
        frame,
        ArenaPreviewSurface,
    );

    for y in (-200..=200).step_by(34) {
        spawn_preview_rect(
            commands,
            Vec2::new(4.0 * scale, 17.0 * scale),
            Vec3::new(0.0, center_y + y as f32 * scale, 14.2),
            secondary,
            ArenaPreviewSurface,
        );
    }

    spawn_preview_rect(
        commands,
        Vec2::new(PADDLE_WIDTH * scale, PADDLE_HEIGHT * scale),
        Vec3::new(-400.0 * scale, center_y, 15.0),
        accent,
        ArenaPreviewPaddle { side: -1.0, follow_speed: 150.0 },
    );
    spawn_preview_rect(
        commands,
        Vec2::new(PADDLE_WIDTH * scale, PADDLE_HEIGHT * scale),
        Vec3::new(400.0 * scale, center_y, 15.0),
        secondary,
        ArenaPreviewPaddle { side: 1.0, follow_speed: 112.0 },
    );
    spawn_preview_rect(
        commands,
        Vec2::splat(13.0 * scale),
        Vec3::new(0.0, center_y, 15.8),
        Color::srgba(0.92, 0.98, 1.0, 0.20),
        ArenaPreviewBallGlow,
    );
    spawn_preview_rect(
        commands,
        Vec2::splat(BALL_SIZE * scale),
        Vec3::new(0.0, center_y, 16.0),
        Color::srgb(0.98, 0.99, 1.0),
        ArenaPreviewBall { velocity: Vec2::new(128.0, 92.0) },
    );
}

fn spawn_preview_rect<M: Component>(
    commands: &mut Commands,
    size: Vec2,
    position: Vec3,
    color: Color,
    marker: M,
) {
    commands.spawn((
        ScreenUi,
        ArenaPreviewVisual,
        marker,
        Sprite::from_color(color, size),
        Transform::from_translation(position),
    ));
}

fn spawn_arena_label<M: Component>(
    commands: &mut Commands,
    font: FontSource,
    marker: M,
    text: impl Into<String>,
    position: Vec2,
    size: f32,
    color: Color,
) {
    commands.spawn((
        ScreenUi,
        marker,
        Text2d::new(text),
        TextFont {
            font,
            font_size: FontSize::Px(size),
            font_smoothing: FontSmoothing::None,
            ..default()
        },
        TextLayout::justify(Justify::Center),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    ));
}

fn arena_counter(selected: usize) -> String {
    if selected == ArenaState::RANDOM_INDEX {
        "21 / 21 // RANDOM".to_owned()
    } else {
        format!("{:02} / 20", selected + 1)
    }
}
