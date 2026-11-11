
use bevy::prelude::*;
use bevy::text::{LineBreak, TextBounds};
use crate::{
    game::components::{
        ContextHelpBody, ContextHelpTitle, MatchSession, MenuItem, MenuSelection, PendingPowerUp,
        PowerUpEditorSelection, PowerUpGridPreview, PowerUpGridPreviewLabel, PowerUpInventory,
        PowerUpKind, ScreenUi, Side, SkinPreview, SkinSelection, InventoryDiscardState,
    },
    localization::tr,
    persistence::{GamepadControlBindings, Language, Settings},
    ui::UiFont,
};
use super::common::*;

pub fn spawn_difficulty(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = settings.difficulty.index();
    let language = settings.language;
    spawn_background(&mut commands);
    screen_header(&mut commands, ui_font.source.clone(), "SELECT DIFFICULTY", "THE RIVAL ADAPTS ITS AIM AND REACTION SPEED", language);
    let mut items: Vec<String> = ["EASY", "MEDIUM", "HARD", "EXPERT"].iter().map(|s| tr(language, s)).collect();
    items.push(tr(language, "BACK"));
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 62.0, 47.0, 13.0);
    spawn_text(&mut commands, ui_font.source.clone(), tr(language, "EASY IS FOR PRACTICE. EXPERT IS FOR LEGENDS."), Vec2::new(0.0, -174.0), 8.0, DIM);
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), language, Vec2::new(0.0, -208.0), 7.0, DIM);
}

pub fn spawn_open_play_setup(
    mut commands: Commands,
    settings: Res<Settings>,
    duration: Res<crate::game::components::OpenPlayDuration>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    let language = settings.language;
    spawn_background(&mut commands);
    screen_header(&mut commands, ui_font.source.clone(), "OPEN PLAY MODE", "SET A MATCH DURATION", language);
    let items = vec![
        format!("MINUTES  //  {:02}", duration.minutes),
        format!("SECONDS  //  {:02}", duration.seconds),
        tr(language, "START OPEN PLAY"),
        tr(language, "BACK"),
    ];
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 62.0, 38.0, 11.0);
    spawn_text(&mut commands, ui_font.source.clone(), tr(language, "WHEN TIME EXPIRES, TIED MATCHES ENTER SUDDEN DEATH"), Vec2::new(0.0, -154.0), 7.0, GOLD);
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), language, Vec2::new(0.0, -208.0), 7.0, DIM);
}

pub fn spawn_score_limit(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    spawn_background(&mut commands);
    screen_header(&mut commands, ui_font.source.clone(), "MATCH LENGTH", "CHOOSE HOW MANY POINTS WIN THE MATCH", language);
    let first_to = tr(language, "FIRST TO");
    let items = vec![
        format!("{first_to} 05"),
        format!("{first_to} 07"),
        format!("{first_to} 10"),
        format!("{first_to} 15"),
        format!("{}{:02}", tr(language, "CUSTOM SCORE: "), settings.winning_score),
        tr(language, "BACK"),
    ];
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 74.0, 42.0, 12.0);
    spawn_text(&mut commands, ui_font.source.clone(), tr(language, "CUSTOM: SELECT THE CUSTOM ROW, THEN USE LEFT / RIGHT"), Vec2::new(0.0, -190.0), 7.0, DIM);
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), language, Vec2::new(0.0, -222.0), 6.5, DIM);
}

pub fn controls_items(settings: &Settings) -> Vec<String> {
    let controls = &settings.controls;
    let menu = &settings.menu_navigation;
    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    let mut items = Vec::with_capacity(16 + capacity * 2);
    let rows = [(0usize, "P1 MOVE UP"), (1, "P1 MOVE DOWN")];
    for (action, label) in rows {
        items.push(format!("{}  //  {}", tr(settings.language, label), controls.binding(action).unwrap_or("?")));
    }
    for slot in 0..capacity {
        let action = 2 + slot;
        items.push(format!("{} {}  //  {}", tr(settings.language, "P1 POWER-UP SLOT"), slot + 1, controls.binding(action).unwrap_or("?")));
    }
    items.push(format!("{}  //  {}", tr(settings.language, "P2 MOVE UP"), controls.binding(7).unwrap_or("?")));
    items.push(format!("{}  //  {}", tr(settings.language, "P2 MOVE DOWN"), controls.binding(8).unwrap_or("?")));
    for slot in 0..capacity {
        let action = 9 + slot;
        items.push(format!("{} {}  //  {}", tr(settings.language, "P2 POWER-UP SLOT"), slot + 1, controls.binding(action).unwrap_or("?")));
    }
    items.push(format!("{}  //  {}", tr(settings.language, "P1 RELEASE BALL"), controls.binding(14).unwrap_or("?")));
    items.push(format!("{}  //  {}", tr(settings.language, "P2 RELEASE BALL"), controls.binding(15).unwrap_or("?")));
    items.push(format!("{}  //  {}", tr(settings.language, "PAUSE GAME"), controls.binding(16).unwrap_or("?")));
    for action in 0..6 {
        items.push(format!("{}  //  {}", tr(settings.language, crate::persistence::MenuNavigationBindings::action_label(action)), menu.binding(action).unwrap_or("?")));
    }
    items.push(tr(settings.language, "GAMEPAD CONTROLS"));
    items.push(tr(settings.language, "RESTORE DEFAULT CONTROLS"));
    items.push(tr(settings.language, "BACK"));
    items
}


pub fn control_action_for_row(row: usize, capacity: usize) -> Option<usize> {
    let capacity = capacity.clamp(1, 5);
    match row {
        0 | 1 => Some(row),
        2..=6 if row < 2 + capacity => Some(row),
        _ if row == 2 + capacity => Some(7),
        _ if row == 3 + capacity => Some(8),
        _ if row >= 4 + capacity && row < 4 + capacity * 2 => Some(9 + row - (4 + capacity)),
        _ if row == 4 + capacity * 2 => Some(14),
        _ if row == 5 + capacity * 2 => Some(15),
        _ if row == 6 + capacity * 2 => Some(16),
        _ => None,
    }
}

pub fn menu_navigation_action_for_row(row: usize, capacity: usize) -> Option<usize> {
    let first = 7 + capacity.clamp(1, 5) * 2;
    (row >= first && row < first + 6).then_some(row - first)
}

pub fn spawn_controls(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    let items = controls_items(&settings);
    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    let player_two_start = capacity + 2;
    let player_one_release = 4 + capacity * 2;
    let player_two_release = player_one_release + 1;
    let pause_row = player_two_release + 1;
    let menu_start = pause_row + 1;
    let gamepad_row = menu_start + 6;
    let restore_row = gamepad_row + 1;
    let back_row = restore_row + 1;

    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "CONTROLS", "SELECT AN ACTION AND PRESS ENTER TO REMAP IT", language);

    for (x, accent) in [(-218.0_f32, CYAN), (218.0_f32, GOLD)] {
        spawn_rect(&mut commands, Vec2::new(400.0, 218.0), Vec3::new(x, 18.0, 10.0), Color::srgb(0.025, 0.065, 0.105));
        spawn_rect(&mut commands, Vec2::new(400.0, 2.0), Vec3::new(x, 127.0, 11.0), accent);
    }
    spawn_text(&mut commands, font.clone(), tr(language, "PLAYER 1 CONTROLS"), Vec2::new(-218.0, 108.0), 9.0, CYAN);
    spawn_text(&mut commands, font.clone(), tr(language, "PLAYER 2 CONTROLS"), Vec2::new(218.0, 108.0), 9.0, GOLD);

    for index in 0..player_two_start {
        spawn_control_row(&mut commands, font.clone(), index, &items[index], -218.0, 80.0 - index as f32 * 20.0, CYAN);
    }
    for index in player_two_start..(player_two_start * 2) {
        let local_row = index - player_two_start;
        spawn_control_row(&mut commands, font.clone(), index, &items[index], 218.0, 80.0 - local_row as f32 * 20.0, GOLD);
    }
    spawn_control_row(&mut commands, font.clone(), player_one_release, &items[player_one_release], -218.0, -70.0, CYAN);
    spawn_control_row(&mut commands, font.clone(), player_two_release, &items[player_two_release], 218.0, -70.0, GOLD);

    
    
    spawn_rect(&mut commands, Vec2::new(704.0, 142.0), Vec3::new(0.0, -164.0, 10.0), Color::srgb(0.025, 0.055, 0.095));
    spawn_rect(&mut commands, Vec2::new(704.0, 2.0), Vec3::new(0.0, -96.0, 11.0), GOLD);
    spawn_text(&mut commands, font.clone(), tr(language, "MENU NAVIGATION CONTROLS"), Vec2::new(0.0, -105.0), 7.8, GOLD);
    spawn_control_row(&mut commands, font.clone(), pause_row, &items[pause_row], 0.0, -125.0, GOLD);
    for action in 0..6 {
        let row = menu_start + action;
        let column = action % 3;
        let line = action / 3;
        let x = [-225.0, 0.0, 225.0][column];
        let y = if line == 0 { -148.0 } else { -171.0 };
        spawn_control_row(&mut commands, font.clone(), row, &items[row], x, y, if action < 3 { CYAN } else { GOLD });
    }
    spawn_text(&mut commands, font.clone(), tr(language, "GAMEPAD BUTTONS ARE CONFIGURABLE PER PLAYER"), Vec2::new(0.0, -194.0), 5.9, DIM);

    for (index, x, accent) in [(gamepad_row, -255.0_f32, CYAN), (restore_row, 0.0_f32, GOLD), (back_row, 255.0_f32, CYAN)] {
        spawn_rect(&mut commands, Vec2::new(210.0, 19.0), Vec3::new(x, -218.0, 10.0), Color::srgb(0.04, 0.075, 0.12));
        spawn_control_row(&mut commands, font.clone(), index, &items[index], x, -218.0, accent);
    }
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -237.0), 5.7, DIM);
}

pub fn gamepad_controls_items(settings: &Settings) -> Vec<String> {
    let mut items = (0..17).map(|action| {
        format!("{}  //  {}", tr(settings.language, GamepadControlBindings::action_label(action)), settings.gamepad_controls.binding(action).unwrap_or("?"))
    }).collect::<Vec<_>>();
    items.push(tr(settings.language, "RESTORE GAMEPAD DEFAULTS"));
    items.push(tr(settings.language, "BACK"));
    items
}

pub fn spawn_gamepad_controls(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    let items = gamepad_controls_items(&settings);
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "GAMEPAD CONTROLS", "SELECT AN ACTION AND PRESS A CONTROLLER BUTTON", language);
    for (x, title, accent) in [(-218.0_f32, "PLAYER 1 GAMEPAD", CYAN), (218.0_f32, "PLAYER 2 GAMEPAD", GOLD)] {
        spawn_rect(&mut commands, Vec2::new(400.0, 300.0), Vec3::new(x, 0.0, 10.0), Color::srgb(0.025, 0.065, 0.105));
        spawn_rect(&mut commands, Vec2::new(400.0, 2.0), Vec3::new(x, 145.0, 11.0), accent);
        spawn_text(&mut commands, font.clone(), tr(language, title), Vec2::new(x, 124.0), 8.5, accent);
    }
    let player_one = [0usize, 1, 2, 3, 4, 5, 6, 14];
    let player_two = [7usize, 8, 9, 10, 11, 12, 13, 15];
    for (line, action) in player_one.iter().enumerate() {
        spawn_control_row(&mut commands, font.clone(), *action, &items[*action], -218.0, 91.0 - line as f32 * 27.0, CYAN);
    }
    for (line, action) in player_two.iter().enumerate() {
        spawn_control_row(&mut commands, font.clone(), *action, &items[*action], 218.0, 91.0 - line as f32 * 27.0, GOLD);
    }
    spawn_rect(&mut commands, Vec2::new(360.0, 30.0), Vec3::new(0.0, -176.0, 10.0), Color::srgb(0.035, 0.07, 0.11));
    spawn_control_row(&mut commands, font.clone(), 16, &items[16], 0.0, -176.0, GOLD);
    for (index, x, accent) in [(17usize, -128.0_f32, GOLD), (18usize, 128.0_f32, CYAN)] {
        spawn_rect(&mut commands, Vec2::new(240.0, 22.0), Vec3::new(x, -218.0, 10.0), Color::srgb(0.04, 0.075, 0.12));
        spawn_control_row(&mut commands, font.clone(), index, &items[index], x, -218.0, accent);
    }
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -239.0), 5.7, DIM);
}

fn spawn_control_row(commands: &mut Commands, font: FontSource, index: usize, label: &str, x: f32, y: f32, accent: Color) {
    commands.spawn((
        ScreenUi,
        MenuItem(index),
        Text2d::new(label.to_owned()),
        TextFont { font, font_size: FontSize::Px(if x.abs() < 1.0 { 7.2 } else { 6.8 }), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(if x.abs() < 1.0 { 440.0 } else { 214.0 }, 18.0)),
        TextColor(accent),
        Transform::from_xyz(x, y, 20.0),
    ));
}

pub fn skin_items(settings: &Settings, selection: &SkinSelection) -> Vec<String> {
    let language = settings.language;
    let (skin, shade, label, custom) = if selection.tab == 0 {
        (settings.paddle_skin, settings.paddle_shade, "PADDLE", settings.paddle_custom_image.is_some())
    } else {
        (settings.ball_skin, settings.ball_shade, "BALL", settings.ball_custom_image.is_some())
    };
    let skin_name = if custom {
        tr(language, "CUSTOM IMAGE")
    } else {
        match skin {
            0..=9 => format!("{} {}/10", tr(language, "SOLID COLOR"), skin + 1),
            10 if selection.tab == 0 => tr(language, "TIGER STRIPE"),
            11 if selection.tab == 0 => tr(language, "ARCADE STRIPES"),
            12 if selection.tab == 0 => tr(language, "CIRCUIT GRID"),
            10 => tr(language, "NEON ORBIT"),
            11 => tr(language, "MAGMA"),
            _ => tr(language, "CHECKER BALL"),
        }
    };
    vec![
        format!("{}: {}", tr(language, "SKIN TYPE"), tr(language, label)),
        format!("{}: {}", tr(language, "SKIN DESIGN"), skin_name),
        if custom || skin >= 10 {
            tr(language, "SHADE NOT USED FOR IMAGE SKINS")
        } else {
            format!("{}: {:+}", tr(language, "SHADE"), shade)
        },
        tr(language, "IMPORT CUSTOM IMAGE"),
        tr(language, "CLEAR CUSTOM IMAGE"),
        tr(language, "RESET SHADE"),
        tr(language, "BACK"),
    ]
}

pub fn spawn_skins(
    mut commands: Commands,
    settings: Res<Settings>,
    skin_selection: Res<SkinSelection>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
    asset_server: Res<AssetServer>,
) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "PADDLE & BALL SKINS", "TEN SOLID COLORS AND THREE ORIGINAL ARCADE DESIGNS", language);
    spawn_menu_items_right_fixed(&mut commands, font.clone(), &skin_items(&settings, &skin_selection), -58.0, 104.0, 28.0, 7.1);
    spawn_rect(&mut commands, Vec2::new(320.0, 248.0), Vec3::new(205.0, 9.0, 10.0), Color::srgb(0.025, 0.07, 0.11));
    spawn_rect(&mut commands, Vec2::new(320.0, 2.0), Vec3::new(205.0, 133.0, 11.0), if skin_selection.tab == 0 { CYAN } else { GOLD });
    let mut paddle_preview = Sprite::from_color(crate::game::systems::paddle_color_with_shade(settings.paddle_skin, settings.paddle_shade), Vec2::new(15.0, 88.0));
    if let Some(path) = crate::game::systems::paddle_skin_asset_path(&settings) {
        paddle_preview = Sprite::from_image(asset_server.load(path));
        paddle_preview.custom_size = Some(Vec2::new(15.0, 88.0));
    }
    let mut ball_preview = Sprite::from_color(crate::game::systems::ball_color_with_shade(settings.ball_skin, settings.ball_shade), Vec2::splat(13.0));
    if let Some(path) = crate::game::systems::ball_skin_asset_path(&settings) {
        ball_preview = Sprite::from_image(asset_server.load(path));
        ball_preview.custom_size = Some(Vec2::splat(13.0));
    }
    commands.spawn((ScreenUi, SkinPreview(false), paddle_preview, Transform::from_xyz(145.0, 18.0, 14.0)));
    commands.spawn((ScreenUi, SkinPreview(true), ball_preview, Transform::from_xyz(265.0, 18.0, 14.0)));
    spawn_text(&mut commands, font.clone(), tr(language, "PADDLE"), Vec2::new(145.0, -55.0), 6.0, CYAN);
    spawn_text(&mut commands, font.clone(), tr(language, "BALL"), Vec2::new(265.0, -55.0), 6.0, GOLD);
    let dimensions = if skin_selection.tab == 0 { "PADDLE IMAGE: EXACTLY 15 x 88 PX" } else { "BALL IMAGE: EXACTLY 13 x 13 PX" };
    spawn_text(&mut commands, font.clone(), tr(language, dimensions), Vec2::new(205.0, -95.0), 6.0, GOLD);
    spawn_text(&mut commands, font.clone(), tr(language, "PNG JPG WEBP SUPPORTED // NO CROP OR STRETCH"), Vec2::new(205.0, -114.0), 5.4, DIM);
    spawn_context_help_panel(&mut commands, font.clone(), language);
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -238.0), 6.0, DIM);
}

pub fn update_skin_preview(
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut previews: Query<(&mut Sprite, &SkinPreview)>,
) {
    for (mut sprite, preview) in &mut previews {
        if preview.0 {
            if let Some(path) = crate::game::systems::ball_skin_asset_path(&settings) {
                sprite.image = asset_server.load(path);
                sprite.color = Color::WHITE;
                sprite.custom_size = Some(Vec2::splat(13.0));
            } else {
                sprite.image = Handle::default();
                sprite.color = crate::game::systems::ball_color_with_shade(settings.ball_skin, settings.ball_shade);
                sprite.custom_size = Some(Vec2::splat(13.0));
            }
        } else if let Some(path) = crate::game::systems::paddle_skin_asset_path(&settings) {
            sprite.image = asset_server.load(path);
            sprite.color = Color::WHITE;
            sprite.custom_size = Some(Vec2::new(15.0, 88.0));
        } else {
            sprite.image = Handle::default();
            sprite.color = crate::game::systems::paddle_color_with_shade(settings.paddle_skin, settings.paddle_shade);
            sprite.custom_size = Some(Vec2::new(15.0, 88.0));
        }
    }
}

pub fn spawn_settings(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    spawn_background(&mut commands);
    screen_header(&mut commands, ui_font.source.clone(), "SYSTEM SETTINGS", "LEFT / RIGHT OR ENTER TO ADJUST", settings.language);
    spawn_menu_items_at_x(&mut commands, ui_font.source.clone(), &settings_items(&settings), -350.0, 126.0, 22.0, 7.0);
    
    let slider_y = 126.0 - 9.0 * 22.0;
    spawn_rect(&mut commands, Vec2::new(132.0, 3.0), Vec3::new(-106.0, slider_y, 17.0), Color::srgb(0.15, 0.29, 0.40));
    for tick in 0..=30 {
        let x = -172.0 + tick as f32 * 4.4;
        spawn_rect(&mut commands, Vec2::new(1.0, if tick % 5 == 0 { 9.0 } else { 4.0 }), Vec3::new(x, slider_y, 18.0), if tick % 5 == 0 { GOLD } else { DIM });
    }
    let fraction = ((settings.ui_scale - 0.80) / 0.30).clamp(0.0, 1.0);
    commands.spawn((
        ScreenUi, crate::game::components::UiScaleSliderKnob,
        Sprite::from_color(CYAN, Vec2::new(8.0, 14.0)),
        Transform::from_xyz(-172.0 + fraction * 132.0, slider_y, 19.0),
    ));
    spawn_text(&mut commands, ui_font.source.clone(), "80%                         110%", Vec2::new(-106.0, slider_y - 11.0), 5.0, DIM);
    spawn_context_help_panel_at(&mut commands, ui_font.source.clone(), settings.language, Vec2::new(225.0, -5.0), Vec2::new(390.0, 154.0));
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), settings.language, Vec2::new(0.0, -206.0), 7.0, DIM);
}

pub fn settings_items(settings: &Settings) -> Vec<String> {
    let l = settings.language;
    vec![
        format!("{:<17} {}", tr(l, "MUSIC"), on_off(settings.music_enabled, l)),
        format!("{:<17} {}", tr(l, "SOUND FX"), on_off(settings.sfx_enabled, l)),
        format!("{:<17} {:>3}%", tr(l, "MUSIC VOLUME"), (settings.music_volume * 100.0).round() as u32),
        format!("{:<17} {:>3}%", tr(l, "FX VOLUME"), (settings.sfx_volume * 100.0).round() as u32),
        format!("{:<17} {}", tr(l, "POWER-UPS"), on_off(settings.powerups_enabled, l)),
        format!("{:<17} {}", tr(l, "SPECIAL BALL"), on_off(settings.special_ball_enabled, l)),
        format!("{:<17} {}", tr(l, "MOVING OBSTACLES"), on_off(settings.obstacles_enabled, l)),
        format!("{:<17} {}", tr(l, "FULLSCREEN"), on_off(settings.fullscreen, l)),
        format!("{:<17} {}", tr(l, "REDUCED MOTION"), on_off(settings.reduce_motion, l)),
        format!("{:<17} {:>3}%", tr(l, "UI SCALE"), (settings.ui_scale * 100.0).round() as u32),
        format!("{:<17} {}", tr(l, "PADDLE SKIN"), settings.paddle_skin + 1),
        format!("{:<17} {}", tr(l, "BALL SKIN"), settings.ball_skin + 1),
        tr(l, "POWER-UP RULES"), tr(l, "BACK"),
    ]
}

pub fn powerup_settings_items(settings: &Settings) -> Vec<String> {
    let l = settings.language;
    vec![
        format!("{:<18} {}", tr(l, "MANUAL INVENTORY"), on_off(settings.manual_powerup_activation, l)),
        format!("{:<18} {}", tr(l, "DOUBLE BALL"), on_off(settings.double_ball_enabled, l)),
        format!("{:<18} {}", tr(l, "MAX ACTIVE BALLS"), settings.max_active_balls),
        format!("{:<18} {}", tr(l, "INVENTORY SLOTS"), settings.inventory_slots),
        format!("{:<18} {:.0}s", tr(l, "SPAWN INTERVAL"), settings.powerup_spawn_interval_secs),
        format!("{:<18} {:>3}%", tr(l, "DOUBLE BALL WEIGHT"), (settings.double_ball_spawn_probability * 100.0).round() as u32),
        format!("{:<18} {}", tr(l, "EVENT DROPS"), on_off(settings.powerup_event_drops, l)),
        tr(l, "PER-ITEM CONFIG"),
        tr(l, "BACK"),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowerUpEditorField {
    Duration,
    Magnitude,
    SpawnWeight,
    MaxCharges,
    Cooldown,
    MaxActiveBalls,
    MinProjectiles,
    MaxProjectiles,
    OpponentBlocks,
    Throws,
}





pub(crate) fn powerup_editor_field(selected: usize, row: usize) -> Option<PowerUpEditorField> {
    use PowerUpEditorField::*;

    match row {
        2 => match selected {
            0 | 1 | 2 | 4 | 5 | 10 | 12 | 13 | 14 => Some(Duration),
            3 => Some(MaxActiveBalls),
            7 | 8 | 9 => Some(Magnitude),
            _ => None,
        },
        3 => match selected {
            0 | 1 | 2 | 4 | 10 | 13 | 14 => Some(Magnitude),
            _ => None,
        },
        4 => Some(SpawnWeight),
        5 => Some(MaxCharges),
        6 => Some(Cooldown),
        7 => match selected {
            8 => Some(MinProjectiles),
            9 => Some(Throws),
            _ => None,
        },
        8 if selected == 8 => Some(MaxProjectiles),
        9 if selected == 8 => Some(OpponentBlocks),
        _ => None,
    }
}

pub(crate) fn is_powerup_editor_row_selectable(selected: usize, row: usize) -> bool {
    matches!(row, 0 | 1 | 10 | 11) || powerup_editor_field(selected, row).is_some()
}

pub fn powerup_editor_items(settings: &Settings, selected: usize) -> Vec<String> {
    use PowerUpEditorField::*;

    let language = settings.language;
    let selected = selected.min(14);
    let name = match selected {
        0 => "SPEED BOOST",
        1 => "SLOW FIELD",
        2 => "BIG PADDLE",
        3 => "DOUBLE BALL",
        4 => "FIREBALL",
        5 => "SNOWBALL",
        6 => "REVERSAL BALL",
        7 => "STICKY BALL",
        8 => "BALL STORM",
        9 => "BOOMERANG BALL",
        10 => "MAGNETIC FIELD",
        11 => "TELEPORT BALL",
        12 => "ENERGY SHIELD",
        13 => "OVERDRIVE",
        _ => "PHANTOM BALL",
    };

    let enabled = match selected {
        0 => settings.speed_powerup.enabled,
        1 => settings.slow_powerup.enabled,
        2 => settings.enlarge_powerup.enabled,
        3 => settings.double_ball_enabled,
        4 => settings.fireball_powerup.enabled,
        5 => settings.snowball_powerup.enabled,
        6 => settings.reversal_powerup.enabled,
        7 => settings.sticky_ball_powerup.enabled,
        8 => settings.ball_storm_powerup.enabled,
        9 => settings.boomerang_powerup.enabled,
        10 => settings.magnetic_field_powerup.enabled,
        11 => settings.teleport_ball_powerup.enabled,
        12 => settings.energy_shield_powerup.enabled,
        13 => settings.overdrive_powerup.enabled,
        _ => settings.phantom_ball_powerup.enabled,
    };

    let mut items = vec![
        format!("{}  //  {}/15  {}", tr(language, "POWER-UP ITEM"), selected + 1, tr(language, name)),
        format!("{}: {}", tr(language, "ENABLED"), on_off(enabled, language)),
    ];

    for row in 2..=9 {
        let Some(field) = powerup_editor_field(selected, row) else {
            items.push(String::new());
            continue;
        };

        let (label, value) = match (selected, field) {
            (0, Duration) => ("DURATION", format!("{:.1}s", settings.speed_powerup.duration_secs)),
            (0, Magnitude) => ("EFFECT STRENGTH", format!("{:.2}x", settings.speed_powerup.magnitude)),
            (1, Duration) => ("DURATION", format!("{:.1}s", settings.slow_powerup.duration_secs)),
            (1, Magnitude) => ("EFFECT STRENGTH", format!("{:.2}x", settings.slow_powerup.magnitude)),
            (2, Duration) => ("DURATION", format!("{:.1}s", settings.enlarge_powerup.duration_secs)),
            (2, Magnitude) => ("EFFECT STRENGTH", format!("{:.2}x", settings.enlarge_powerup.magnitude)),
            (3, MaxActiveBalls) => ("MAX ACTIVE BALLS", settings.max_active_balls.to_string()),
            (4, Duration) => ("DAMAGE DURATION", format!("{:.1}s", settings.fireball_powerup.duration_secs)),
            (4, Magnitude) => ("SHRINK MULTIPLIER", format!("{:.2}x", settings.fireball_powerup.magnitude)),
            (5, Duration) => ("FREEZE DURATION", format!("{:.1}s", settings.snowball_powerup.duration_secs)),
            (7, Magnitude) => ("RELEASE SPEED", format!("{:.0}", settings.sticky_ball_powerup.magnitude)),
            (8, Magnitude) => ("PROJECTILE SPEED", format!("{:.0}", settings.ball_storm_powerup.magnitude)),
            (9, Magnitude) => ("RETURN SPEED", format!("{:.0}", settings.boomerang_powerup.magnitude)),
            (10, Duration) => ("FIELD DURATION", format!("{:.1}s", settings.magnetic_field_powerup.duration_secs)),
            (10, Magnitude) => ("ATTRACTION", format!("{:.2}", settings.magnetic_field_powerup.magnitude)),
            (12, Duration) => ("SHIELD DURATION", format!("{:.1}s", settings.energy_shield_powerup.duration_secs)),
            (13, Duration) => ("OVERDRIVE DURATION", format!("{:.1}s", settings.overdrive_powerup.duration_secs)),
            (13, Magnitude) => ("PADDLE SPEED", format!("{:.2}x", settings.overdrive_powerup.magnitude)),
            (14, Duration) => ("PHANTOM DURATION", format!("{:.1}s", settings.phantom_ball_powerup.duration_secs)),
            (14, Magnitude) => ("VISIBILITY", format!("{:.0}%", settings.phantom_ball_powerup.magnitude * 100.0)),
            (0, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.speed_powerup.spawn_weight * 100.0)),
            (1, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.slow_powerup.spawn_weight * 100.0)),
            (2, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.enlarge_powerup.spawn_weight * 100.0)),
            (3, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.double_ball_spawn_probability * 100.0)),
            (4, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.fireball_powerup.spawn_weight * 100.0)),
            (5, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.snowball_powerup.spawn_weight * 100.0)),
            (6, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.reversal_powerup.spawn_weight * 100.0)),
            (7, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.sticky_ball_powerup.spawn_weight * 100.0)),
            (8, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.ball_storm_powerup.spawn_weight * 100.0)),
            (9, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.boomerang_powerup.spawn_weight * 100.0)),
            (10, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.magnetic_field_powerup.spawn_weight * 100.0)),
            (11, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.teleport_ball_powerup.spawn_weight * 100.0)),
            (12, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.energy_shield_powerup.spawn_weight * 100.0)),
            (13, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.overdrive_powerup.spawn_weight * 100.0)),
            (14, SpawnWeight) => ("SPAWN WEIGHT", format!("{:.0}%", settings.phantom_ball_powerup.spawn_weight * 100.0)),
            (0, MaxCharges) => ("MAX CHARGES", settings.speed_powerup.max_charges.to_string()),
            (1, MaxCharges) => ("MAX CHARGES", settings.slow_powerup.max_charges.to_string()),
            (2, MaxCharges) => ("MAX CHARGES", settings.enlarge_powerup.max_charges.to_string()),
            (3, MaxCharges) => ("MAX CHARGES", settings.double_ball_max_charges.to_string()),
            (4, MaxCharges) => ("MAX CHARGES", settings.fireball_powerup.max_charges.to_string()),
            (5, MaxCharges) => ("MAX CHARGES", settings.snowball_powerup.max_charges.to_string()),
            (6, MaxCharges) => ("MAX CHARGES", settings.reversal_powerup.max_charges.to_string()),
            (7, MaxCharges) => ("MAX CHARGES", settings.sticky_ball_powerup.max_charges.to_string()),
            (8, MaxCharges) => ("MAX CHARGES", settings.ball_storm_powerup.max_charges.to_string()),
            (9, MaxCharges) => ("MAX CHARGES", settings.boomerang_powerup.max_charges.to_string()),
            (10, MaxCharges) => ("MAX CHARGES", settings.magnetic_field_powerup.max_charges.to_string()),
            (11, MaxCharges) => ("MAX CHARGES", settings.teleport_ball_powerup.max_charges.to_string()),
            (12, MaxCharges) => ("MAX CHARGES", settings.energy_shield_powerup.max_charges.to_string()),
            (13, MaxCharges) => ("MAX CHARGES", settings.overdrive_powerup.max_charges.to_string()),
            (14, MaxCharges) => ("MAX CHARGES", settings.phantom_ball_powerup.max_charges.to_string()),
            (0, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.speed_powerup.cooldown_secs)),
            (1, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.slow_powerup.cooldown_secs)),
            (2, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.enlarge_powerup.cooldown_secs)),
            (3, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.double_ball_cooldown_secs)),
            (4, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.fireball_powerup.cooldown_secs)),
            (5, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.snowball_powerup.cooldown_secs)),
            (6, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.reversal_powerup.cooldown_secs)),
            (7, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.sticky_ball_powerup.cooldown_secs)),
            (8, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.ball_storm_powerup.cooldown_secs)),
            (9, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.boomerang_powerup.cooldown_secs)),
            (10, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.magnetic_field_powerup.cooldown_secs)),
            (11, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.teleport_ball_powerup.cooldown_secs)),
            (12, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.energy_shield_powerup.cooldown_secs)),
            (13, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.overdrive_powerup.cooldown_secs)),
            (14, Cooldown) => ("COOLDOWN", format!("{:.1}s", settings.phantom_ball_powerup.cooldown_secs)),
            (8, MinProjectiles) => ("MIN PROJECTILES", settings.ball_storm_powerup.min_projectiles.to_string()),
            (8, MaxProjectiles) => ("MAX PROJECTILES", settings.ball_storm_powerup.max_projectiles.to_string()),
            (8, OpponentBlocks) => ("OPPONENT BLOCKS", settings.ball_storm_powerup.opponent_blocks.to_string()),
            (9, Throws) => ("THROWS", settings.boomerang_powerup.throws.to_string()),
            _ => ("", String::new()),
        };
        items.push(format!("{}: {}", tr(language, label), value));
    }

    items.push(tr(language, "RESET ITEM DEFAULTS"));
    items.push(tr(language, "BACK"));
    items
}

pub fn spawn_powerup_editor(
    mut commands: Commands,
    settings: Res<Settings>,
    selection: Res<crate::game::components::PowerUpEditorSelection>,
    mut menu_selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
    asset_server: Res<AssetServer>,
) {
    menu_selection.selected = 0;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "POWER-UP ITEM CONFIG", "SELECT AN ITEM, THEN TUNE ITS BEHAVIOR", settings.language);
    spawn_menu_items_at_x(&mut commands, font.clone(), &powerup_editor_items(&settings, selection.0), -350.0, 108.0, 19.0, 6.2);

    let kinds = [PowerUpKind::Speed, PowerUpKind::Slow, PowerUpKind::Enlarge, PowerUpKind::DoubleBall,
        PowerUpKind::Fireball, PowerUpKind::Snowball, PowerUpKind::ReversalBall, PowerUpKind::StickyBall,
        PowerUpKind::BallStorm, PowerUpKind::BoomerangBall, PowerUpKind::MagneticField,
        PowerUpKind::TeleportBall, PowerUpKind::EnergyShield, PowerUpKind::Overdrive, PowerUpKind::PhantomBall];
    spawn_text(&mut commands, font.clone(), tr(settings.language, "POWER-UP SELECT GRID"), Vec2::new(245.0, 136.0), 6.8, GOLD);
    for (index, kind) in kinds.iter().copied().enumerate() {
        let column = index % 5;
        let row = index / 5;
        let x = 85.0 + column as f32 * 79.0;
        let y = 108.0 - row as f32 * 47.0;
        let mut icon = Sprite::from_image(asset_server.load(format!("powerups/{}/frame_0.png", kind.pixel_art_key())));
        icon.custom_size = Some(Vec2::splat(25.0));
        commands.spawn((ScreenUi, icon, Transform::from_xyz(x, y, 14.0)));
        commands.spawn((
            ScreenUi, MenuItem(12 + index), Text2d::new(tr(settings.language, kind.label())),
            TextFont { font: font.clone(), font_size: FontSize::Px(4.5), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Center), TextColor(if selection.0 == index { GOLD } else { WHITE }),
            Transform::from_xyz(x, y - 24.0, 15.0),
        ));
    }
    let preview_kind = kinds[selection.0.min(14)];
    let mut preview = Sprite::from_image(asset_server.load(format!("powerups/{}/frame_0.png", preview_kind.pixel_art_key())));
    preview.custom_size = Some(Vec2::splat(48.0));
    commands.spawn((ScreenUi, PowerUpGridPreview { displayed_index: selection.0.min(14) }, preview, Transform::from_xyz(38.0, -95.0, 15.0)));
    commands.spawn((
        ScreenUi, PowerUpGridPreviewLabel,
        Text2d::new(tr(settings.language, preview_kind.label())),
        TextFont { font: font.clone(), font_size: FontSize::Px(5.8), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD),
        Transform::from_xyz(38.0, -128.0, 16.0),
    ));
    spawn_context_help_panel_at(&mut commands, font.clone(), settings.language, Vec2::new(246.0, -151.0), Vec2::new(360.0, 118.0));
    spawn_navigation_hint(&mut commands, font, settings.language, Vec2::new(-150.0, -225.0), 6.0, DIM);
}

pub fn update_powerup_editor_preview(
    selection: Res<PowerUpEditorSelection>,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut preview: Query<(&mut Sprite, &mut PowerUpGridPreview)>,
    mut preview_label: Query<&mut Text2d, With<PowerUpGridPreviewLabel>>,
) {
    let kinds = [PowerUpKind::Speed, PowerUpKind::Slow, PowerUpKind::Enlarge, PowerUpKind::DoubleBall,
        PowerUpKind::Fireball, PowerUpKind::Snowball, PowerUpKind::ReversalBall, PowerUpKind::StickyBall,
        PowerUpKind::BallStorm, PowerUpKind::BoomerangBall, PowerUpKind::MagneticField,
        PowerUpKind::TeleportBall, PowerUpKind::EnergyShield, PowerUpKind::Overdrive, PowerUpKind::PhantomBall];
    let index = selection.0.min(14);
    for (mut sprite, mut preview) in &mut preview {
        if preview.displayed_index != index {
            sprite.image = asset_server.load(format!("powerups/{}/frame_0.png", kinds[index].pixel_art_key()));
            preview.displayed_index = index;
            for mut label in &mut preview_label {
                label.0 = tr(settings.language, kinds[index].label());
            }
        }
    }
}

pub fn spawn_powerup_settings(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    spawn_background(&mut commands);
    screen_header(&mut commands, ui_font.source.clone(), "POWER-UP RULES", "TUNE SPAWNS, DOUBLE BALL AND INVENTORY", settings.language);
    spawn_menu_items_at_x(&mut commands, ui_font.source.clone(), &powerup_settings_items(&settings), -350.0, 100.0, 28.0, 7.4);
    spawn_context_help_panel_at(&mut commands, ui_font.source.clone(), settings.language, Vec2::new(218.0, -8.0), Vec2::new(380.0, 286.0));
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), settings.language, Vec2::new(0.0, -205.0), 7.0, DIM);
}

pub fn update_menu_hover(
    screen: Res<State<crate::game::Screen>>,
    mut dropdown: ResMut<crate::game::components::LanguageDropdownState>,
    mut selection: ResMut<MenuSelection>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    items: Query<(&MenuItem, &Transform)>,
    settings: Res<Settings>,
) {
    let language_menu_open = *screen.get() == crate::game::Screen::MainMenu && dropdown.open;
    if !language_menu_open && !matches!(screen.get(), crate::game::Screen::Settings | crate::game::Screen::PowerUpSettings | crate::game::Screen::PowerUpEditor | crate::game::Screen::NetworkLobby | crate::game::Screen::NetworkGuide | crate::game::Screen::HighScores | crate::game::Screen::Skins | crate::game::Screen::Credits) {
        return;
    }
    let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) else { return; };
    let Some(cursor) = window.cursor_position() else { return; };
    let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) else { return; };
    let ui_scale = settings.ui_scale.clamp(0.80, 1.10);
    for (item, transform) in &items {
        if language_menu_open && !(20..26).contains(&item.0) { continue; }
        if !language_menu_open && *screen.get() == crate::game::Screen::MainMenu { continue; }
        let width = match screen.get() {
            crate::game::Screen::Credits => 46.0,
            crate::game::Screen::NetworkLobby if item.0 < 4 => 190.0,
            crate::game::Screen::NetworkLobby if item.0 == 30 => 120.0,
            crate::game::Screen::NetworkLobby => 98.0,
            crate::game::Screen::NetworkGuide => 150.0,
            crate::game::Screen::HighScores => 135.0,
            crate::game::Screen::PowerUpEditor if item.0 >= 12 => 42.0,
            _ => 360.0,
        };
        let height = if matches!(*screen.get(), crate::game::Screen::NetworkGuide | crate::game::Screen::HighScores) { 14.0 }
            else if *screen.get() == crate::game::Screen::PowerUpEditor && item.0 >= 12 { 11.0 }
            else { 13.0 };
        if (world.x - transform.translation.x).abs() <= width * ui_scale
            && (world.y - transform.translation.y).abs() <= height * ui_scale
        {
            selection.selected = item.0;
            if language_menu_open {
                dropdown.selected = item.0.saturating_sub(20).min(crate::persistence::Language::ALL.len() - 1);
            }
            break;
        }
    }
}

pub fn update_contextual_help(
    screen: Res<State<crate::game::Screen>>,
    selection: Res<MenuSelection>,
    editor: Res<PowerUpEditorSelection>,
    settings: Res<Settings>,
    menu_labels: Query<(&MenuItem, &Text2d), (Without<ContextHelpTitle>, Without<ContextHelpBody>)>,
    mut titles: Query<&mut Text2d, (With<ContextHelpTitle>, Without<ContextHelpBody>)>,
    mut bodies: Query<&mut Text2d, (With<ContextHelpBody>, Without<ContextHelpTitle>)>,
) {
    use crate::game::Screen;
    let current = *screen.get();
    if current == Screen::NetworkLobby {
        let (title, body) = crate::localization::network_context_help(settings.language, selection.selected.min(30));
        for mut text in &mut titles { text.0 = title.clone(); }
        for mut text in &mut bodies { text.0 = body.clone(); }
        return;
    }
    if !matches!(current, Screen::Settings | Screen::PowerUpSettings | Screen::PowerUpEditor | Screen::Skins) { return; }
    let (title, key) = match current {
        Screen::Settings => (match selection.selected {
            0 => "MUSIC", 1 => "SOUND FX", 2 => "MUSIC VOLUME", 3 => "FX VOLUME",
            4 => "POWER-UPS", 5 => "SPECIAL BALL", 6 => "MOVING OBSTACLES", 7 => "FULLSCREEN",
            8 => "REDUCED MOTION", 9 => "UI SCALE", 10 => "PADDLE SKIN", 11 => "BALL SKIN", 12 => "POWER-UP RULES", _ => "BACK",
        }, match selection.selected {
            0 => "HELP_MUSIC", 1 => "HELP_SFX", 2 => "HELP_MUSIC_VOLUME", 3 => "HELP_SFX_VOLUME",
            4 => "HELP_POWERUPS", 5 => "HELP_SPECIAL_BALL", 6 => "HELP_OBSTACLES", 7 => "HELP_FULLSCREEN",
            8 => "HELP_REDUCED_MOTION", 9 => "HELP_UI_SCALE", 10 => "HELP_PADDLE_SKIN", 11 => "HELP_BALL_SKIN", 12 => "HELP_POWERUP_RULES", _ => "HELP_BACK",
        }),
        Screen::PowerUpSettings => (match selection.selected {
            0 => "MANUAL INVENTORY", 1 => "DOUBLE BALL", 2 => "MAX ACTIVE BALLS", 3 => "INVENTORY SLOTS",
            4 => "SPAWN INTERVAL", 5 => "DOUBLE BALL WEIGHT", 6 => "EVENT DROPS", 7 => "PER-ITEM CONFIG", _ => "BACK",
        }, match selection.selected {
            0 => "HELP_MANUAL_INVENTORY", 1 => "HELP_DOUBLE_BALL", 2 => "HELP_MAX_BALLS", 3 => "HELP_INVENTORY_SLOTS",
            4 => "HELP_SPAWN_INTERVAL", 5 => "HELP_DOUBLE_BALL_WEIGHT", 6 => "HELP_EVENT_DROPS", 7 => "HELP_EDITOR", _ => "HELP_BACK",
        }),
        Screen::Skins => (match selection.selected {
            0 => "SKIN TYPE", 1 => "SKIN DESIGN", 2 => "SHADE", 3 => "IMPORT CUSTOM IMAGE",
            4 => "CLEAR CUSTOM IMAGE", 5 => "RESET SHADE", _ => "BACK",
        }, match selection.selected {
            0 => "HELP_SKIN_TYPE", 1 => "HELP_SKIN_DESIGN", 2 => "HELP_SKIN_SHADE", 3 => "HELP_SKIN_IMPORT",
            4 => "HELP_SKIN_CLEAR", 5 => "HELP_SKIN_RESET", _ => "HELP_BACK",
        }),
        Screen::PowerUpEditor => {
            let key = if (12..27).contains(&selection.selected) { "HELP_ITEM_SELECT" } else { match selection.selected {
                0 => "HELP_ITEM_SELECT",
                1 => "HELP_ITEM_ENABLED",
                2 if editor.0 == 3 => "HELP_MAX_BALLS",
                2 => "HELP_DURATION",
                3 => "HELP_MAGNITUDE",
                4 => "HELP_SPAWN_WEIGHT",
                5 => "HELP_MAX_CHARGES",
                6 => "HELP_COOLDOWN",
                7 if editor.0 == 8 => "HELP_MIN_PROJECTILES",
                7 if editor.0 == 9 => "HELP_THROWS",
                7 => "HELP_MAGNITUDE",
                8 => "HELP_MAX_PROJECTILES",
                9 => "HELP_OPPONENT_BLOCKS",
                _ => "HELP_BACK",
            }};
            let title = match selection.selected {
                0 => "POWER-UP ITEM", 1 => "ENABLED", 2 => "DURATION", 3 => "STRENGTH",
                4 => "SPAWN WEIGHT", 5 => "MAX CHARGES", 6 => "COOLDOWN", 7 => "PROJECTILE SETTINGS", _ => "BACK",
            };
            (title, key)
        }
        _ => ("CONTEXT HELP", "HELP_BACK"),
    };
    let selected_label = menu_labels.iter().find(|(item, _)| item.0 == selection.selected).map(|(_, text)| text.0.clone());
    let title_text = selected_label.unwrap_or_else(|| tr(settings.language, title));
    for mut text in &mut titles { text.0 = title_text.clone(); }
    for mut text in &mut bodies { text.0 = crate::localization::contextual_help(settings.language, key); }
}

pub fn inventory_management_items(
    inventory: &PowerUpInventory,
    pending: &PendingPowerUp,
    discard: &InventoryDiscardState,
    session: &MatchSession,
    inventory_capacity: u8,
    language: Language,
) -> Vec<String> {
    let mut items = Vec::with_capacity(12);
    let capacity = inventory_capacity.clamp(1, 5) as usize;
    for (side, prefix) in [(Side::Player, "P1"), (Side::PlayerTwo, "P2")] {
        let slots = inventory.for_side(side);
        let side_capacity = if side == Side::PlayerTwo && session.mode != crate::game::components::MatchMode::LocalVersus {
            0
        } else {
            
            capacity.min(5)
        };
        for slot in 0..5 {
            let unavailable = side == Side::PlayerTwo && session.mode != crate::game::components::MatchMode::LocalVersus;
            let item = if unavailable {
                tr(language, "LOCAL 2P ONLY")
            } else if let Some(item) = slots.get(slot) {
                format!("{}  x{}{}", tr(language, item.kind.label()), item.charges,
                    if item.cooldown_remaining > 0.0 { format!("  /  {:.1}s", item.cooldown_remaining) } else { String::new() })
            } else if slot >= side_capacity {
                tr(language, "LOCKED")
            } else {
                tr(language, "EMPTY")
            };
            let selected = discard.selected == Some((side, slot));
            let action = if !unavailable && slots.get(slot).is_some() {
                if pending.kind.is_some() && pending.owner == side { format!("  //  {}", tr(language, "ENTER: REPLACE")) }
                else if selected { format!("  //  {}", tr(language, "DELETE: HOLD 0.5S")) }
                else { format!("  //  {}", tr(language, "ENTER: SELECT / DELETE: DISCARD")) }
            } else if !unavailable && pending.kind.is_some() && pending.owner == side
                && slot == slots.len() && slot < side_capacity {
                format!("  //  {}", tr(language, "ENTER: ADD TO NEXT SLOT"))
            } else { String::new() };
            let marker = if selected { "> " } else { "  " };
            items.push(format!("{marker}{prefix} SLOT {}  //  {item}{action}", slot + 1));
        }
    }
    items.push(if pending.kind.is_some() {
        tr(language, "DISCARD PENDING PICKUP")
    } else {
        tr(language, "NO PENDING PICKUP")
    });
    items.push(tr(language, "BACK"));
    items
}

pub fn spawn_inventory_management(
    mut commands: Commands,
    inventory: Res<PowerUpInventory>,
    pending: Res<PendingPowerUp>,
    session: Res<MatchSession>,
    discard: Res<InventoryDiscardState>,
    settings: Res<Settings>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    spawn_background(&mut commands);
    let language = settings.language;
    screen_header(&mut commands, ui_font.source.clone(), "POWER-UP INVENTORY", "SELECT A SLOT TO DISCARD AN ITEM", language);
    let items = inventory_management_items(&inventory, &pending, &discard, &session, settings.inventory_slots, language);
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 103.0, 20.0, 7.5);
    if let Some(kind) = pending.kind {
        let owner = if pending.owner == Side::PlayerTwo { "P2" } else { "P1" };
        spawn_text(
            &mut commands,
            ui_font.source.clone(),
            format!("{} [{owner}]: {}", tr(language, "PENDING PICKUP"), tr(language, kind.label())),
            Vec2::new(0.0, -157.0),
            7.5,
            GOLD,
        );
    }
    spawn_text(
        &mut commands,
        ui_font.source.clone(),
        tr(language, "ENTER: SELECT OR REPLACE  //  HOLD DELETE 0.5S TO DISCARD  //  ESC: RETURN"),
        Vec2::new(0.0, -193.0),
        7.0,
        DIM,
    );
}

fn on_off(value: bool, language: Language) -> String {
    if value { tr(language, "ON") } else { tr(language, "OFF") }
}



