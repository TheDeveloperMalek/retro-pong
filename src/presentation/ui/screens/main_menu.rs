
use bevy::prelude::*;
use bevy::text::{LineBreak, TextBounds};
use crate::{
    game::components::{LanguageFlagPixel, MainMenuGithubBar, MainMenuLabel, MenuItem, MenuSelection, ScreenUi, LanguageDropdownItem, LanguageDropdownPanel},
    localization::tr,
    persistence::{Language, Settings},
    ui::UiFont,
};
use super::common::*;

pub fn spawn_main_menu(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    spawn_background(&mut commands);
    spawn_main_menu_label(&mut commands, ui_font.source.clone(), language, 0, "RETRO PONG", Vec2::new(0.0, 207.0), 28.0, CYAN);
    spawn_main_menu_label(&mut commands, ui_font.source.clone(), language, 1, "ARCADE EDITION  //  32-BIT CLASSICS", Vec2::new(0.0, 178.0), 20.0, GOLD);
    spawn_main_menu_label(&mut commands, ui_font.source.clone(), language, 2, "SOLO AI  //  MULTIPLAYER NETWORK", Vec2::new(0.0, 145.0), 10.5, DIM);
    let items = [
        "PLAY VS AI", "LOCAL 2 PLAYER", "DIFFICULTY", "SCORE LIMIT", "OPEN PLAY MODE", "ARENA SELECT",
        "CONTROLS", "SETTINGS", "POWER-UP RULES", "HIGH SCORES", "PLAYER PROGRESS", "CREDITS / ABOUT",
        "NETWORK MULTIPLAYER", "EXIT",
    ].map(|key| tr(language, key));
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 80.0, 17.0, 8.5);

    
    spawn_rect(&mut commands, Vec2::new(154.0, 27.0), Vec3::new(-326.0, 126.0, 18.0), Color::srgb(0.055, 0.10, 0.16));
    spawn_rect(&mut commands, Vec2::new(154.0, 1.5), Vec3::new(-326.0, 112.5, 19.0), CYAN);
    spawn_flag(&mut commands, language, Vec2::new(-382.0, 126.0));
    commands.spawn((
        ScreenUi, MenuItem(14), Text2d::new(language.native_name()),
        TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(7.4), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(WHITE),
        Transform::from_xyz(-326.0, 126.0, 20.0),
    ));
    
    
    commands.spawn((
        ScreenUi, LanguageDropdownPanel, Visibility::Hidden,
        Sprite::from_color(Color::srgb(0.035, 0.065, 0.11), Vec2::new(154.0, 111.0)),
        Transform::from_xyz(-326.0, 53.0, 24.0),
    ));
    for (index, option) in Language::ALL.iter().enumerate() {
        commands.spawn((
            ScreenUi, MenuItem(20 + index), LanguageDropdownItem(index), Visibility::Hidden,
            Text2d::new(option.native_name()),
            TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(6.8), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Center), TextColor(WHITE),
            Transform::from_xyz(-326.0, 96.0 - index as f32 * 17.0, 26.0),
        ));
    }

    
    
    let github_label = tr(language, "STAR ON GITHUB");
    let github_width = ((github_label.chars().count() as f32 * 5.2) + 90.0).clamp(184.0, 360.0);
    
    
    
    commands.spawn((
        ScreenUi,
        MainMenuGithubBar { height: 25.0 },
        Sprite::from_color(Color::srgb(0.055, 0.10, 0.16), Vec2::new(github_width, 25.0)),
        Transform::from_xyz(0.0, -203.0, 18.0),
    ));
    commands.spawn((
        ScreenUi,
        MainMenuGithubBar { height: 1.5 },
        Sprite::from_color(CYAN, Vec2::new(github_width, 1.5)),
        Transform::from_xyz(0.0, -190.0, 19.0),
    ));
    commands.spawn((
        ScreenUi,
        MainMenuGithubBar { height: 1.5 },
        Sprite::from_color(CYAN, Vec2::new(github_width, 1.5)),
        Transform::from_xyz(0.0, -216.0, 19.0),
    ));
    spawn_star_pixel_icon(&mut commands, Vec2::new(-62.0, -203.0), 2.5, 21.0, GOLD);
    let github_text_center = 22.0;
    commands.spawn((
        ScreenUi, MenuItem(15), Text2d::new(github_label),
        TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(8.5), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD),
        Transform::from_xyz(github_text_center, -203.0, 20.0),
    ));
    
    spawn_main_menu_label(&mut commands, ui_font.source.clone(), language, 3, "UP / DOWN SELECT  //  ENTER OPENS LANGUAGE  //  ENTER CONFIRMS", Vec2::new(0.0, -181.0), 7.0, DIM);
}

fn spawn_main_menu_label(
    commands: &mut Commands,
    font: FontSource,
    language: Language,
    index: usize,
    key: &'static str,
    position: Vec2,
    size: f32,
    color: Color,
) {
    commands.spawn((
        ScreenUi,
        MainMenuLabel(index),
        Text2d::new(tr(language, key)),
        TextFont { font: font.clone(), font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(if index == 1 { 640.0 } else { 880.0 }, if index == 1 { 28.0 } else { 42.0 })),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    ));
}

pub(super) fn spawn_star_pixel_icon(commands: &mut Commands, center: Vec2, cell: f32, z: f32, color: Color) {
    const PIXELS: [&str; 9] = [
        "....#....",
        "....#....",
        "...###...",
        "#########",
        ".#######.",
        "..#####..",
        ".##.#.##.",
        "##..#..##",
        "....#....",
    ];
    for (row, line) in PIXELS.iter().enumerate() {
        for (column, pixel) in line.chars().enumerate() {
            if pixel != '#' { continue; }
            let x = center.x + (column as f32 - 4.0) * cell;
            let y = center.y + (4.0 - row as f32) * cell;
            spawn_rect(commands, Vec2::splat(cell), Vec3::new(x, y, z), color);
        }
    }
}

pub(super) fn spawn_flag(commands: &mut Commands, language: Language, center: Vec2) {
    let rows: [&str; 5] = match language {
        Language::English => ["BBWWWWBB", "BWRRRWBB", "WWRRRRWW", "BWRRRWBB", "BBWWWWBB"],
        Language::German => ["KKKKKKKK", "KKKKKKKK", "RRRRRRRR", "RRRRRRRR", "GGGGGGGG"],
        Language::French => ["BBWWRRBB", "BBWWRRBB", "BBWWRRBB", "BBWWRRBB", "BBWWRRBB"],
        Language::Swedish => ["BBBBBBBB", "BBYYYYBB", "YYYYYYYY", "BBYYYYBB", "BBBBBBBB"],
        Language::Spanish => ["RRRRRRRR", "RRRRRRRR", "YYYYYYYY", "YYYYYYYY", "RRRRRRRR"],
        Language::Russian => ["WWWWWWWW", "WWWWWWWW", "BBBBBBBB", "BBBBBBBB", "RRRRRRRR"],
    };
    let cell = 4.0;
    let palette = |code| match code {
        'B' => Color::srgb(0.08, 0.22, 0.66), 'W' => Color::srgb(0.92, 0.94, 0.98),
        'R' => Color::srgb(0.78, 0.12, 0.16), 'Y' => Color::srgb(0.98, 0.78, 0.12),
        'K' => Color::srgb(0.08, 0.08, 0.10), 'G' => Color::srgb(0.86, 0.62, 0.14),
        _ => WHITE,
    };
    for (row, line) in rows.iter().enumerate() {
        for (column, code) in line.chars().enumerate() {
            let x = center.x + (column as f32 - 4.0) * cell + cell * 0.5;
            let y = center.y + (2.0 - row as f32) * cell + cell * 0.5;
            commands.spawn((ScreenUi, LanguageFlagPixel, Sprite::from_color(palette(code), Vec2::splat(cell)), Transform::from_xyz(x, y, 21.0)));
        }
    }
    
    for y in [center.y + 11.0, center.y - 11.0] {
        commands.spawn((
            ScreenUi,
            LanguageFlagPixel,
            Sprite::from_color(CYAN, Vec2::new(34.0, 1.5)),
            Transform::from_xyz(center.x, y, 22.0),
        ));
    }
}

