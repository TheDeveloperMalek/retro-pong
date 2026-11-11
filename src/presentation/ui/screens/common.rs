
use bevy::prelude::*;
use bevy::text::{LineBreak, TextBounds};
use bevy::window::PrimaryWindow;

pub(super) const WHITE: Color = Color::srgb(0.88, 0.94, 1.0);
pub(super) const DIM: Color = Color::srgb(0.36, 0.51, 0.68);
pub(super) const CYAN: Color = Color::srgb(0.34, 0.95, 0.88);
pub(super) const GOLD: Color = Color::srgb(1.0, 0.77, 0.34);
pub(super) const PINK: Color = Color::srgb(1.0, 0.43, 0.57);
pub(super) const PANEL: Color = Color::srgb(0.035, 0.055, 0.105);

use crate::{
    game::{components::{
        ActiveInputDevice, ClickBurstParticle, ClickEffectCooldown, ClickRippleDot, ContextHelpBody, ContextHelpTitle,
        HudScore, HudStatus, LanguageFlagPixel, NavigationHint, NavigationState, PauseOverlay, MenuItem,
        PowerUpHudSlot, PowerUpHudTooltip, ScreenUi, StarField, TransientNotice, UiScaleBaseline, UiScaleSliderKnob,
    }, Screen},
    localization::tr,
    persistence::{Language, Settings},
};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub(crate) struct NetworkLobbyStatusText;
#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub(crate) struct NetworkLobbyStatusIndicator;
#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub(crate) struct NetworkLobbyDetailsText;



pub fn capture_ui_scale_baselines(
    mut commands: Commands,
    settings: Res<Settings>,
    entities: Query<
        (
            Entity,
            &Transform,
            Option<&TextFont>,
            Option<&Sprite>,
            Option<&TextBounds>,
            Option<&PowerUpHudTooltip>,
        ),
        (
            Or<(
                Added<ScreenUi>,
                Added<HudScore>,
                Added<HudStatus>,
                Added<PowerUpHudSlot>,
                Added<PowerUpHudTooltip>,
            )>,
            Without<UiScaleBaseline>,
        ),
    >,
) {
    let scale = settings.ui_scale.clamp(0.80, 1.10);
    for (entity, transform, font, sprite, bounds, tooltip) in &entities {
        
        
        
        let font_size_px = font.and_then(|font| {
            if let bevy::text::FontSize::Px(value) = &font.font_size {
                Some(*value)
            } else {
                None
            }
        });
        let mut baseline_transform = *transform;
        
        
        
        if tooltip.is_some() {
            baseline_transform.translation.x /= scale;
            baseline_transform.translation.y /= scale;
        }
        commands.entity(entity).insert(UiScaleBaseline {
            transform: baseline_transform,
            font_size_px,
            sprite_custom_size: sprite.and_then(|sprite| sprite.custom_size),
            text_bounds: bounds.map(|bounds| (bounds.width, bounds.height)),
        });
    }
}





pub fn apply_ui_scale(
    settings: Res<Settings>,
    mut entities: Query<(
        Ref<UiScaleBaseline>,
        &mut Transform,
        Option<&mut TextFont>,
        Option<&mut Sprite>,
        Option<&mut TextBounds>,
    )>,
) {
    let factor = settings.ui_scale.clamp(0.80, 1.10);
    for (baseline, mut transform, font, sprite, bounds) in &mut entities {
        if !settings.is_changed() && !baseline.is_added() {
            continue;
        }
        let mut scaled = baseline.transform;
        scaled.translation.x *= factor;
        scaled.translation.y *= factor;
        
        
        
        if baseline.sprite_custom_size.is_none() && baseline.font_size_px.is_none() {
            scaled.scale *= factor;
        }
        *transform = scaled;
        if let (Some(size), Some(mut sprite)) = (baseline.sprite_custom_size, sprite) {
            sprite.custom_size = Some(size * factor);
        }
        if let (Some(size), Some(mut font)) = (baseline.font_size_px, font) {
            font.font_size = bevy::text::FontSize::Px(size * factor);
        }
        if let (Some((width, height)), Some(mut bounds)) = (baseline.text_bounds, bounds) {
            bounds.width = width.map(|value| value * factor);
            bounds.height = height.map(|value| value * factor);
        }
    }
}



pub fn update_ui_scale_slider(
    screen: Res<State<Screen>>,
    settings: Res<Settings>,
    mut knobs: Query<&mut Transform, With<UiScaleSliderKnob>>,
) {
    if *screen.get() != Screen::Settings { return; }
    let factor = settings.ui_scale.clamp(0.80, 1.10);
    let fraction = ((factor - 0.80) / 0.30).clamp(0.0, 1.0);
    let scale = factor;
    for mut transform in &mut knobs {
        transform.translation.x = (-172.0 + fraction * 132.0) * scale;
        transform.translation.y = (126.0 - 9.0 * 22.0) * scale;
    }
}

pub fn cleanup_screen(mut commands: Commands, entities: Query<Entity, With<ScreenUi>>) {
    for entity in &entities { commands.entity(entity).despawn(); }
}

pub fn cleanup_pause_overlay(mut commands: Commands, overlays: Query<Entity, With<PauseOverlay>>) {
    for entity in &overlays { commands.entity(entity).despawn(); }
}

pub(super) fn spawn_rect(commands: &mut Commands, size: Vec2, position: Vec3, color: Color) {
    commands.spawn((ScreenUi, Sprite::from_color(color, size), Transform::from_translation(position)));
}

pub(super) fn spawn_background(commands: &mut Commands) {
    
    spawn_rect(commands, Vec2::new(890.0, 470.0), Vec3::new(0.0, 0.0, -10.0), PANEL);
    spawn_rect(commands, Vec2::new(890.0, 2.0), Vec3::new(0.0, 233.0, -9.0), Color::srgb(0.12, 0.35, 0.42));
    spawn_rect(commands, Vec2::new(890.0, 2.0), Vec3::new(0.0, -233.0, -9.0), Color::srgb(0.12, 0.35, 0.42));
    spawn_rect(commands, Vec2::new(2.0, 466.0), Vec3::new(-445.0, 0.0, -9.0), Color::srgb(0.12, 0.35, 0.42));
    spawn_rect(commands, Vec2::new(2.0, 466.0), Vec3::new(445.0, 0.0, -9.0), Color::srgb(0.12, 0.35, 0.42));
    for index in 0..18 {
        let y = -210.0 + index as f32 * 24.0;
        spawn_rect(commands, Vec2::new(886.0, 1.0), Vec3::new(0.0, y, -8.0), Color::srgba(0.22, 0.36, 0.48, 0.10));
    }
    for (x, y, size) in [(-420.0, 210.0, 5.0), (420.0, 210.0, 5.0), (-420.0, -210.0, 5.0), (420.0, -210.0, 5.0)] {
        spawn_rect(commands, Vec2::splat(size), Vec3::new(x, y, -7.0), GOLD);
    }
    for i in 0..24 {
        let x = ((i * 73) % 840) as f32 - 420.0;
        let y = ((i * 47) % 420) as f32 - 210.0;
        commands.spawn((
            ScreenUi,
            StarField {
                speed: 4.0 + (i % 5) as f32 * 2.0,
            },
            Sprite::from_color(
                Color::srgba(0.36, 0.77, 0.88, 0.45),
                Vec2::splat(2.0),
            ),
            Transform::from_xyz(x, y, -6.0),
        ));
    }
}

pub fn animate_background(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<Settings>,
    
    
    mut stars: Query<
        (&StarField, &mut Transform),
        (Without<ClickRippleDot>, Without<ClickBurstParticle>),
    >,
    mut ripples: Query<
        (Entity, &mut ClickRippleDot, &mut Transform, &mut Sprite),
        (Without<StarField>, Without<ClickBurstParticle>),
    >,
    mut particles: Query<
        (Entity, &mut ClickBurstParticle, &mut Transform, &mut Sprite),
        (Without<StarField>, Without<ClickRippleDot>),
    >,
    mut notices: Query<(Entity, &mut TransientNotice)>,
) {
    let dt = time.delta_secs().min(0.05);
    if !settings.reduce_motion {
        for (star, mut transform) in &mut stars {
            transform.translation.y -= star.speed * dt;
            if transform.translation.y < -224.0 { transform.translation.y = 224.0; }
        }
    }
    for (entity, mut ripple, mut transform, mut sprite) in &mut ripples {
        ripple.remaining -= dt;
        if ripple.remaining <= 0.0 || settings.reduce_motion { commands.entity(entity).despawn(); continue; }
        let progress = 1.0 - ripple.remaining / ripple.lifetime;
        let radius = 8.0 + progress * 84.0;
        transform.translation.x = ripple.origin.x + ripple.angle.cos() * radius;
        transform.translation.y = ripple.origin.y + ripple.angle.sin() * radius;
        let alpha = (1.0 - progress) * 0.8;
        sprite.color = Color::srgba(0.34, 0.95, 0.88, alpha);
    }
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        particle.remaining -= dt;
        if particle.remaining <= 0.0 || settings.reduce_motion { commands.entity(entity).despawn(); continue; }
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity *= 0.965_f32.powf(dt * 60.0);
        let alpha = (particle.remaining / particle.lifetime).clamp(0.0, 1.0);
        sprite.color = Color::srgba(1.0, 0.77, 0.34, alpha);
    }
    for (entity, mut notice) in &mut notices {
        notice.remaining -= dt;
        if notice.remaining <= 0.0 { commands.entity(entity).despawn(); }
    }
}

pub fn show_notice(commands: &mut Commands, font: FontSource, message: impl Into<String>, success: bool) {
    commands.spawn((
        ScreenUi,
        TransientNotice { remaining: 2.8 },
        Text2d::new(message),
        TextFont { font: font.clone(), font_size: FontSize::Px(7.8), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center),
        TextColor(if success { GOLD } else { PINK }),
        Transform::from_xyz(0.0, -246.0, 35.0),
    ));
}

pub fn background_click_effects(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    screen: Res<State<Screen>>,
    settings: Res<Settings>,
    mut cooldown: ResMut<ClickEffectCooldown>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);
    if settings.reduce_motion || screen.get() == &Screen::Playing || !buttons.just_pressed(MouseButton::Left) || cooldown.0 > 0.0 {
        return;
    }
    let Ok(window) = windows.single() else { return; };
    let Some(cursor) = window.cursor_position() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };
    let Ok(origin) = camera.viewport_to_world_2d(camera_transform, cursor) else { return; };

    cooldown.0 = 0.68;
    for index in 0..24 {
        let angle = std::f32::consts::TAU * index as f32 / 24.0;
        commands.spawn((
            ScreenUi,
            ClickRippleDot { origin, angle, remaining: 0.62, lifetime: 0.62 },
            Sprite::from_color(Color::srgba(0.34, 0.95, 0.88, 0.8), Vec2::splat(4.0)),
            Transform::from_xyz(origin.x, origin.y, 28.0),
        ));
    }
    for index in 0..12 {
        let angle = std::f32::consts::TAU * index as f32 / 12.0 + 0.12;
        let speed = 62.0 + (index % 4) as f32 * 20.0;
        commands.spawn((
            ScreenUi,
            ClickBurstParticle { velocity: Vec2::from_angle(angle) * speed, remaining: 0.48, lifetime: 0.48 },
            Sprite::from_color(Color::srgba(1.0, 0.77, 0.34, 1.0), Vec2::splat(if index % 3 == 0 { 5.0 } else { 3.0 })),
            Transform::from_xyz(origin.x, origin.y, 29.0),
        ));
    }
}

pub fn refresh_language_flag(commands: &mut Commands, entities: &mut Query<Entity, With<LanguageFlagPixel>>, language: Language) {
    for entity in entities.iter() { commands.entity(entity).despawn(); }
    super::main_menu::spawn_flag(commands, language, Vec2::new(-382.0, 126.0));
}

pub(super) fn spawn_navigation_hint(
    commands: &mut Commands,
    font: FontSource,
    language: Language,
    position: Vec2,
    size: f32,
    color: Color,
) {
    commands.spawn((
        ScreenUi,
        NavigationHint,
        Text2d::new(tr(language, "NAVIGATION HINT KEYBOARD")),
        TextFont { font: font.clone(), font_size: FontSize::Px((size * 1.12).max(6.0)), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    ));
}

pub fn update_navigation_hint_text(
    settings: Res<Settings>,
    navigation: Res<NavigationState>,
    mut hints: Query<&mut Text2d, With<NavigationHint>>,
) {
    if !settings.is_changed() && !navigation.is_changed() { return; }
    let key = match navigation.active_device {
        ActiveInputDevice::Gamepad => "NAVIGATION HINT GAMEPAD",
        ActiveInputDevice::Mouse => "NAVIGATION HINT MOUSE",
        ActiveInputDevice::Keyboard => "NAVIGATION HINT KEYBOARD",
    };
    let label = tr(settings.language, key);
    for mut text in &mut hints {
        if text.0 != label { text.0 = label.clone(); }
    }
}

pub(super) fn spawn_text(
    commands: &mut Commands,
    font: FontSource,
    text: impl Into<String>,
    position: Vec2,
    size: f32,
    color: Color,
) {
    commands.spawn((
        ScreenUi,
        Text2d::new(text),
        TextFont { font: font.clone(), font_size: FontSize::Px((size * 1.12).max(6.2)), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    ));
}

pub(super) fn spawn_menu_items(commands: &mut Commands, font: FontSource, items: &[String], y_start: f32, gap: f32, font_size: f32) {
    for (index, item) in items.iter().enumerate() {
        commands.spawn((
            ScreenUi,
            MenuItem(index),
            Text2d::new(item.clone()),
            TextFont { font: font.clone(), font_size: FontSize::Px((font_size * 1.12).max(7.0)), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Center),
            TextColor(WHITE),
            Transform::from_xyz(0.0, y_start - index as f32 * gap, 20.0),
        ));
    }
}

pub(super) fn spawn_menu_items_at_x(commands: &mut Commands, font: FontSource, items: &[String], x: f32, y_start: f32, gap: f32, font_size: f32) {
    for (index, item) in items.iter().enumerate() {
        commands.spawn((
            ScreenUi,
            MenuItem(index),
            Text2d::new(item.clone()),
            TextFont { font: font.clone(), font_size: FontSize::Px((font_size * 1.12).max(7.0)), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Left),
            TextColor(WHITE),
            Transform::from_xyz(x, y_start - index as f32 * gap, 20.0),
        ));
    }
}

pub(super) fn spawn_menu_items_right_fixed(commands: &mut Commands, font: FontSource, items: &[String], right_edge: f32, y_start: f32, gap: f32, font_size: f32) {
    for (index, item) in items.iter().enumerate() {
        commands.spawn((
            ScreenUi,
            MenuItem(index),
            Text2d::new(item.clone()),
            TextFont { font: font.clone(), font_size: FontSize::Px((font_size * 1.12).max(7.0)), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Right),
            TextColor(WHITE),
            Transform::from_xyz(right_edge, y_start - index as f32 * gap, 20.0),
        ));
    }
}

pub(super) fn spawn_context_help_panel(commands: &mut Commands, font: FontSource, language: Language) {
    spawn_context_help_panel_at(commands, font, language, Vec2::new(225.0, -5.0), Vec2::new(390.0, 320.0));
}

pub(super) fn spawn_context_help_panel_at(
    commands: &mut Commands,
    font: FontSource,
    language: Language,
    center: Vec2,
    size: Vec2,
) {
    let top = center.y + size.y * 0.5;
    let bottom = center.y - size.y * 0.5;
    spawn_rect(commands, size, Vec3::new(center.x, center.y, 10.0), Color::srgb(0.025, 0.065, 0.105));
    spawn_rect(commands, Vec2::new(size.x, 2.0), Vec3::new(center.x, top, 11.0), CYAN);
    spawn_rect(commands, Vec2::new(2.0, size.y), Vec3::new(center.x - size.x * 0.5, center.y, 11.0), CYAN);
    commands.spawn((
        ScreenUi, ContextHelpTitle, Text2d::new(tr(language, "CONTEXT HELP")),
        TextFont { font: font.clone(), font_size: FontSize::Px(if size.y < 180.0 { 8.0 } else { 9.2 }), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD), Transform::from_xyz(center.x, top - 32.0, 20.0),
    ));
    commands.spawn((
        ScreenUi, ContextHelpBody, Text2d::new(crate::localization::contextual_help(language, "HELP_DEFAULT")),
        TextFont { font: font.clone(), font_size: FontSize::Px(if size.y < 180.0 { 7.0 } else { 7.6 }), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new((size.x - 28.0).max(80.0), (size.y - 62.0).max(30.0))),
        TextColor(WHITE), Transform::from_xyz(center.x, center.y - 7.0, 20.0),
    ));
    spawn_text(commands, font, tr(language, "HOVER OR FOCUS AN OPTION"), Vec2::new(center.x, bottom + 10.0), 6.2, DIM);
}

pub(super) fn screen_header(commands: &mut Commands, font: FontSource, title: &str, subtitle: &str, language: Language) {
    spawn_text(commands, font.clone(), tr(language, title), Vec2::new(0.0, 198.0), 24.0, CYAN);
    spawn_text(commands, font, tr(language, subtitle), Vec2::new(0.0, 160.0), 9.5, DIM);
}

