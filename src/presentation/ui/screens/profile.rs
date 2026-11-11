
use bevy::prelude::*;
use bevy::text::{LineBreak, TextBounds};
use crate::{
    game::components::{MenuSelection, HighScoreFilters},
    localization::tr,
    persistence::{PersistentData, Settings, Difficulty, SavedMatchType},
    progression,
    ui::UiFont,
};
use super::common::*;

#[derive(Component)]
pub(crate) struct HighScoreTooltipSource {
    full_text: String,
    bounds: Vec2,
}

#[derive(Component)]
pub(crate) struct HighScoreTooltip;

pub fn spawn_high_scores(
    mut commands: Commands,
    data: Res<PersistentData>,
    settings: Res<Settings>,
    mut filters: ResMut<HighScoreFilters>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "HALL OF FAME", "FILTER YOUR SAVED MATCH HISTORY", language);

    let today = chrono::Utc::now().date_naive();
    let cutoff = match filters.period {
        1 => Some(today - chrono::Duration::days(7)),
        2 => Some(today - chrono::Duration::days(30)),
        _ => None,
    };
    let wanted_type = match filters.match_type {
        1 => Some(SavedMatchType::SoloAi),
        2 => Some(SavedMatchType::LocalVersus),
        3 => Some(SavedMatchType::Network),
        _ => None,
    };
    let wanted_difficulty = if filters.difficulty == 0 { None } else { Some(Difficulty::from_index((filters.difficulty - 1) as usize)) };
    let records: Vec<_> = data.0.high_scores.iter().filter(|record| {
        let date_matches = cutoff.is_none_or(|minimum| chrono::NaiveDate::parse_from_str(&record.date, "%Y-%m-%d").is_ok_and(|date| date >= minimum && date <= today));
        wanted_type.is_none_or(|kind| kind == record.match_type)
            && wanted_difficulty.is_none_or(|difficulty| difficulty == record.difficulty)
            && date_matches
    }).collect();
    let page_size = 10usize;
    let page_count = records.len().div_ceil(page_size).max(1);
    let current_page = filters.page.min(page_count - 1);
    filters.page = current_page;
    let page_records: Vec<_> = records.iter().skip(current_page * page_size).take(page_size).copied().collect();

    let match_type_label = match filters.match_type {
        1 => "SOLO AI", 2 => "LOCAL 2 PLAYER", 3 => "NETWORK", _ => "ALL TYPES",
    };
    let difficulty_label = match filters.difficulty {
        1 => "EASY", 2 => "MEDIUM", 3 => "HARD", 4 => "EXPERT", _ => "ALL DIFFICULTIES",
    };
    let period_label = match filters.period {
        1 => "LAST 7 DAYS", 2 => "LAST 30 DAYS", _ => "ALL TIME",
    };
    let filters = [
        (0usize, format!("{}: {}", tr(language, "MATCH TYPE"), tr(language, match_type_label)), -270.0_f32),
        (1usize, format!("{}: {}", tr(language, "DIFFICULTY"), tr(language, difficulty_label)), 0.0_f32),
        (2usize, format!("{}: {}", tr(language, "DATE PERIOD"), tr(language, period_label)), 270.0_f32),
    ];
    for (index, label, x) in filters {
        commands.spawn((
            crate::game::components::ScreenUi, crate::game::components::MenuItem(index),
            Text2d::new(label),
            TextFont { font: font.clone(), font_size: FontSize::Px(7.3), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Center), TextColor(WHITE), Transform::from_xyz(x, 119.0, 20.0),
        ));
    }
    if page_records.is_empty() {
        spawn_rect(&mut commands, Vec2::new(520.0, 128.0), Vec3::new(0.0, 8.0, 10.0), PANEL);
        spawn_text(&mut commands, font.clone(), tr(language, "NO MATCHES FOR THESE FILTERS"), Vec2::new(0.0, 14.0), 9.0, GOLD);
        spawn_text(&mut commands, font.clone(), tr(language, "CHANGE THE FILTERS OR PLAY A MATCH TO RECORD A SCORE"), Vec2::new(0.0, -20.0), 7.0, DIM);
    } else if current_page == 0 {
        
        
        let podium = [
            page_records.get(1).map(|record| (1usize, record, -166.0_f32, 100.0_f32, Color::srgb(0.47, 0.61, 0.76))),
            page_records.first().map(|record| (0usize, record, 0.0_f32, 112.0_f32, GOLD)),
            page_records.get(2).map(|record| (2usize, record, 166.0_f32, 96.0_f32, Color::srgb(0.69, 0.42, 0.28))),
        ];
        for entry in podium.into_iter().flatten() {
            let (rank, record, x, height, accent) = entry;
            spawn_rect(&mut commands, Vec2::new(160.0, height), Vec3::new(x, 44.0 - height * 0.5, 10.0), Color::srgb(0.045, 0.075, 0.13));
            spawn_rect(&mut commands, Vec2::new(160.0, 2.0), Vec3::new(x, 44.0, 11.0), accent);
            spawn_fit_text(&mut commands, font.clone(), format!("{}  #{}", tr(language, "RANK"), rank + 1), Vec2::new(x, 28.0), 7.0, accent, Vec2::new(146.0, 14.0));
            spawn_fit_text(&mut commands, font.clone(), format!("{:02} : {:02}", record.player_score, record.opponent_score), Vec2::new(x, 4.0), 14.5, GOLD, Vec2::new(146.0, 20.0));
            let result = if record.won { tr(language, "VICTORY") } else { tr(language, "DEFEAT") };
            spawn_fit_text(&mut commands, font.clone(), format!("{}  //  {}", tr(language, record.difficulty.label()), result), Vec2::new(x, -15.0), 6.4, WHITE, Vec2::new(146.0, 15.0));
            if record.personal_record {
                spawn_fit_text(&mut commands, font.clone(), format!("★  {}", tr(language, "NEW PERSONAL BEST")), Vec2::new(x, -31.0), 6.1, GOLD, Vec2::new(146.0, 15.0));
            }
            let date_y = 44.0 - height + 8.0;
            spawn_fit_text(&mut commands, font.clone(), record.date.clone(), Vec2::new(x, date_y), 6.0, DIM, Vec2::new(146.0, 14.0));
        }

        if page_records.len() > 3 {
            spawn_text(&mut commands, font.clone(), tr(language, "MORE HIGH SCORES"), Vec2::new(0.0, -78.0), 7.0, CYAN);
            for (offset, record) in page_records.iter().skip(3).take(7).enumerate() {
                let y = -98.0 - offset as f32 * 13.5;
                let accent = if record.personal_record { GOLD } else { DIM };
                spawn_rect(&mut commands, Vec2::new(620.0, 14.0), Vec3::new(0.0, y, 9.0), if offset % 2 == 0 { Color::srgb(0.035, 0.06, 0.11) } else { Color::srgb(0.025, 0.045, 0.09) });
                spawn_text(&mut commands, font.clone(), format!("{:02}", offset + 4), Vec2::new(-292.0, y - 2.0), 6.4, accent);
                spawn_text(&mut commands, font.clone(), format!("{:02} - {:02}", record.player_score, record.opponent_score), Vec2::new(-188.0, y - 2.0), 7.2, GOLD);
                let details = format!("{}  //  {}  //  {}", tr(language, record.match_type.label()), tr(language, record.difficulty.label()), record.date);
                spawn_high_score_hover_text(&mut commands, font.clone(), details, Vec2::new(70.0, y - 2.0), 6.2, WHITE, Vec2::new(350.0, 13.0));
                if record.personal_record {
                    spawn_text(&mut commands, font.clone(), tr(language, "NEW PB"), Vec2::new(275.0, y - 2.0), 5.5, GOLD);
                }
            }
        }
    } else {
        spawn_text(&mut commands, font.clone(), tr(language, "MORE HIGH SCORES"), Vec2::new(0.0, 105.0), 8.0, CYAN);
        for (offset, record) in page_records.iter().enumerate() {
            let y = 78.0 - offset as f32 * 24.0;
            let rank = current_page * page_size + offset + 1;
            let accent = if record.personal_record { GOLD } else { DIM };
            spawn_rect(&mut commands, Vec2::new(650.0, 20.0), Vec3::new(0.0, y, 9.0), if offset % 2 == 0 { Color::srgb(0.035, 0.06, 0.11) } else { Color::srgb(0.025, 0.045, 0.09) });
            spawn_text(&mut commands, font.clone(), format!("#{rank}"), Vec2::new(-300.0, y - 3.0), 6.4, accent);
            spawn_text(&mut commands, font.clone(), format!("{:02} - {:02}", record.player_score, record.opponent_score), Vec2::new(-190.0, y - 3.0), 7.2, GOLD);
            let details = format!("{} // {} // {}", tr(language, record.match_type.label()), tr(language, record.difficulty.label()), record.date);
            spawn_high_score_hover_text(&mut commands, font.clone(), details, Vec2::new(62.0, y - 3.0), 6.2, WHITE, Vec2::new(380.0, 15.0));
            if record.personal_record { spawn_text(&mut commands, font.clone(), tr(language, "NEW PB"), Vec2::new(284.0, y - 3.0), 5.4, GOLD); }
        }
    }
    for (index, key, x) in [
        (3usize, "PREVIOUS PAGE", -180.0_f32),
        (4usize, "NEXT PAGE", 0.0_f32),
        (5usize, "BACK", 180.0_f32),
    ] {
        let label = match index {
            3 if current_page == 0 => format!("< {}", tr(language, key)),
            4 if current_page + 1 >= page_count => format!("{} >", tr(language, key)),
            _ => tr(language, key),
        };
        commands.spawn((
            crate::game::components::ScreenUi, crate::game::components::MenuItem(index),
            Text2d::new(label),
            TextFont { font: font.clone(), font_size: FontSize::Px(7.5), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::justify(Justify::Center), TextColor(if index == 5 { WHITE } else { CYAN }), Transform::from_xyz(x, -218.0, 20.0),
        ));
    }
    spawn_text(&mut commands, font.clone(), format!("PAGE {} / {}  //  {} MATCHES", current_page + 1, page_count, records.len()), Vec2::new(0.0, -194.0), 5.5, DIM);
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -238.0), 6.0, DIM);
}

pub fn spawn_progress(
    mut commands: Commands,
    data: Res<PersistentData>,
    settings: Res<Settings>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    screen_header(&mut commands, font.clone(), "PLAYER PROGRESS", "ACHIEVEMENTS AND MATCH STATISTICS", language);
    let progress = &data.0.progress;
    let matches = progress.total_matches;
    let win_rate = if matches == 0 { 0 } else { progress.wins.saturating_mul(100) / matches };
    let personal_best = data.0.high_scores.iter().map(|record| record.player_score).max().unwrap_or(0);
    let stats = [
        ("W", "WINS", progress.wins.to_string(), CYAN),
        ("L", "LOSSES", progress.losses.to_string(), PINK),
        ("#", "TOTAL POINTS", progress.points_scored.to_string(), GOLD),
        ("M", "MATCHES PLAYED", matches.to_string(), WHITE),
        ("%", "WIN RATE", format!("{win_rate}%"), CYAN),
        ("★", "PERSONAL BEST", personal_best.to_string(), GOLD),
        ("↗", "BEST WIN STREAK", progress.best_win_streak.to_string(), GOLD),
    ];
    for (index, (icon, label, value, accent)) in stats.iter().enumerate() {
        let column = index % 4;
        let row = index / 4;
        let x = [-330.0_f32, -110.0, 110.0, 330.0][column];
        let y = 105.0 - row as f32 * 59.0;
        spawn_rect(&mut commands, Vec2::new(204.0, 50.0), Vec3::new(x, y, 10.0), Color::srgb(0.045, 0.075, 0.13));
        spawn_rect(&mut commands, Vec2::new(3.0, 50.0), Vec3::new(x - 100.0, y, 11.0), *accent);
        spawn_text(&mut commands, font.clone(), *icon, Vec2::new(x - 72.0, y + 6.0), 12.0, *accent);
        spawn_fit_text(&mut commands, font.clone(), tr(language, label), Vec2::new(x + 22.0, y + 8.0), 7.1, DIM, Vec2::new(142.0, 15.0));
        spawn_fit_text(&mut commands, font.clone(), value.clone(), Vec2::new(x + 22.0, y - 11.0), 13.6, WHITE, Vec2::new(142.0, 19.0));
    }

    spawn_text(&mut commands, font.clone(), tr(language, "ARCADE ACHIEVEMENTS"), Vec2::new(0.0, -13.0), 8.0, CYAN);
    for (index, id) in progression::ACHIEVEMENT_IDS.iter().enumerate() {
        let column = index % 4;
        let row = index / 4;
        let x = [-330.0_f32, -110.0, 110.0, 330.0][column];
        let y = -78.0 - row as f32 * 64.0;
        let unlocked = progress.achievements.iter().any(|earned| earned == id);
        let color = achievement_color(index, unlocked);
        spawn_rect(&mut commands, Vec2::new(204.0, 56.0), Vec3::new(x, y, 10.0), Color::srgb(0.035, 0.06, 0.11));
        spawn_achievement_badge(&mut commands, Vec2::new(x - 78.0, y), 2.0, index, color);
        let title = tr(language, progression::achievement_name(id));
        let subtitle = if unlocked { tr(language, "UNLOCKED") } else { tr(language, progression::achievement_requirement(id)) };
        spawn_fit_text(&mut commands, font.clone(), title, Vec2::new(x + 20.0, y + 8.0), 6.8, if unlocked { GOLD } else { DIM }, Vec2::new(136.0, 16.0));
        spawn_fit_text(&mut commands, font.clone(), subtitle, Vec2::new(x + 20.0, y - 12.0), 6.2, if unlocked { CYAN } else { DIM }, Vec2::new(136.0, 16.0));
        if !unlocked {
            
            spawn_text(&mut commands, font.clone(), "LOCKED", Vec2::new(x + 76.0, y - 23.0), 5.4, DIM);
        }
    }
    commands.spawn((
        crate::game::components::ScreenUi, crate::game::components::MenuItem(3),
        Text2d::new(tr(language, "BACK")),
        TextFont { font: font.clone(), font_size: FontSize::Px(9.0), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(WHITE), Transform::from_xyz(0.0, -218.0, 20.0),
    ));
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -238.0), 6.0, DIM);
}

fn achievement_color(index: usize, unlocked: bool) -> Color {
    if !unlocked { return Color::srgb(0.28, 0.31, 0.37); }
    match index % 4 {
        0 => GOLD,
        1 => CYAN,
        2 => PINK,
        _ => Color::srgb(0.66, 0.58, 1.0),
    }
}

fn spawn_achievement_badge(commands: &mut Commands, center: Vec2, cell: f32, index: usize, color: Color) {
    const BADGES: [[&str; 9]; 8] = [
        ["..#####..", ".##...##.", "##.#.#.##", "##.....##", "##..#..##", "##.....##", ".##...##.", "..#####..", "....#...."],
        ["...###...", "..#####..", ".##.#.##.", "##..#..##", "##..#..##", ".##.#.##.", "..#####..", "...###...", "....#...."],
        ["..#####..", ".#######.", "###...###", "##..#..##", "##..#..##", "###...###", ".#######.", "..#####..", "....#...."],
        ["....#....", "...###...", "..#####..", ".#######.", "#########", "...###...", "...###...", "...###...", "....#...."],
        ["..#####..", ".##...##.", "##.....##", "##.###.##", "##.###.##", "##.....##", ".##...##.", "..#####..", "....#...."],
        ["...###...", "..#####..", ".#######.", "###.#.###", "##..#..##", "###...###", ".#######.", "..#####..", "...#.#..."],
        ["..#####..", ".##...##.", "##.#.#.##", "##..#..##", "##.#.#.##", "##..#..##", ".##...##.", "..#####..", "....#...."],
        ["....#....", "...###...", "..#####..", ".#######.", "###...###", "##.#.#.##", "#########", "...###...", "...###..."],
    ];
    let pattern = &BADGES[index % BADGES.len()];
    for (row, line) in pattern.iter().enumerate() {
        for (column, pixel) in line.chars().enumerate() {
            if pixel != '#' { continue; }
            let x = center.x + (column as f32 - 4.0) * cell;
            let y = center.y + (4.0 - row as f32) * cell;
            spawn_rect(commands, Vec2::splat(cell), Vec3::new(x, y, 13.0), color);
        }
    }
}


fn spawn_high_score_hover_text(
    commands: &mut Commands,
    font: FontSource,
    full_text: String,
    position: Vec2,
    size: f32,
    color: Color,
    bounds: Vec2,
) {
    let max_chars = ((bounds.x / (size * 0.58)).floor() as usize).max(12);
    let visible_text = if full_text.chars().count() > max_chars {
        let mut clipped = full_text.chars().take(max_chars.saturating_sub(1)).collect::<String>();
        clipped.push('…');
        clipped
    } else {
        full_text.clone()
    };
    let entity = commands.spawn((
        crate::game::components::ScreenUi,
        Text2d::new(visible_text),
        TextFont { font, font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(bounds),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    )).id();
    commands.entity(entity).insert(HighScoreTooltipSource { full_text, bounds });
}

pub fn update_high_score_tooltip(
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    sources: Query<(&HighScoreTooltipSource, &Transform), Without<HighScoreTooltip>>,
    mut tooltip: Query<(Entity, &mut Text2d, &mut Transform), With<HighScoreTooltip>>,
    ui_font: Res<UiFont>,
    settings: Res<crate::persistence::Settings>,
) {
    let ui_scale = settings.ui_scale.clamp(0.80, 1.10);
    let hovered = windows.single().ok()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor| {
            let (camera, camera_transform) = cameras.single().ok()?;
            camera.viewport_to_world_2d(camera_transform, cursor).ok()
        })
        .and_then(|cursor| sources.iter().find(|(source, transform)| {
            let half = source.bounds * (0.5 * ui_scale);
            (cursor.x - transform.translation.x).abs() <= half.x
                && (cursor.y - transform.translation.y).abs() <= half.y
                && source.full_text.chars().count() > ((source.bounds.x / (6.2 * 0.58)).floor() as usize).max(12)
        }).map(|(source, _)| (source.full_text.clone(), cursor)));

    if let Some((full_text, cursor)) = hovered {
        let position = Vec3::new((cursor.x + 12.0).clamp(-250.0, 250.0), (cursor.y + 18.0).clamp(-230.0, 230.0), 40.0);
        if let Ok((_, mut text, mut transform)) = tooltip.single_mut() {
            text.0 = full_text;
            transform.translation = position;
        } else {
            commands.spawn((
                crate::game::components::ScreenUi,
                HighScoreTooltip,
                Text2d::new(full_text),
                TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(7.5), font_smoothing: FontSmoothing::None, ..default() },
                TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
                TextBounds::from(Vec2::new(420.0, 36.0)),
                TextColor(GOLD),
                Transform::from_translation(position),
            ));
        }
    } else {
        for (entity, _, _) in &mut tooltip {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_fit_text(
    commands: &mut Commands,
    font: FontSource,
    text: impl Into<String>,
    position: Vec2,
    size: f32,
    color: Color,
    bounds: Vec2,
) {
    commands.spawn((
        crate::game::components::ScreenUi,
        Text2d::new(text),
        TextFont { font, font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(bounds),
        TextColor(color),
        Transform::from_xyz(position.x, position.y, 20.0),
    ));
}
