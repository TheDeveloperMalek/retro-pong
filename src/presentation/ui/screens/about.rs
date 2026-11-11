
use bevy::prelude::*;
use bevy::text::TextBounds;
use crate::{
    game::components::{DeveloperDocumentState, DeveloperPongBall, DeveloperPongPaddle, MatchSession, MenuItem, MenuSelection, PauseOverlay, ScreenUi, SocialIcon, SocialIconLabel},
    localization::tr,
    persistence::Settings,
    ui::UiFont,
};
use super::common::*;

pub fn spawn_credits(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_developer_background(&mut commands, font.clone(), language);
    screen_header(&mut commands, font.clone(), "CREDITS / ABOUT", "A SMALL ARCADE GAME WITH BIG NOSTALGIA", language);

    
    
    spawn_rect(&mut commands, Vec2::new(380.0, 210.0), Vec3::new(0.0, 14.0, 10.0), Color::srgb(0.025, 0.042, 0.078));
    spawn_rect(&mut commands, Vec2::new(380.0, 22.0), Vec3::new(0.0, 108.0, 11.0), Color::srgb(0.055, 0.085, 0.14));
    for (index, color) in [PINK, GOLD, CYAN].iter().enumerate() {
        spawn_rect(&mut commands, Vec2::splat(6.0), Vec3::new(132.0 + index as f32 * 14.0, 108.0, 12.0), *color);
    }
    commands.spawn((
        ScreenUi,
        Text2d::new("src/credits.rs"),
        TextFont { font: font.clone(), font_size: FontSize::Px(9.2), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center),
        TextColor(CYAN),
        Transform::from_xyz(0.0, 105.0, 22.0),
    ));
    
    
    let code_lines = [
        ("01", "use std::fmt;", DIM),
        ("02", "struct Developer {", CYAN),
        ("03", "    name: &'static str,", Color::srgb(0.62, 0.87, 0.67)),
        ("04", "}", Color::srgb(0.58, 0.77, 1.0)),
        ("05", "impl Developer {", Color::srgb(0.58, 0.77, 1.0)),
        ("06", "    fn new(name: &'static str) -> Self {", WHITE),
        ("07", "        Self { name }", Color::srgb(0.62, 0.87, 0.67)),
        ("08", "    }", Color::srgb(0.58, 0.77, 1.0)),
        ("09", "}", Color::srgb(0.58, 0.77, 1.0)),
        ("10", "impl fmt::Display for Developer {", Color::srgb(0.58, 0.77, 1.0)),
        ("11", "    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {", WHITE),
        ("12", "        write!(f, \"{}\", self.name)", Color::srgb(0.62, 0.87, 0.67)),
        ("13", "    }", Color::srgb(0.58, 0.77, 1.0)),
        ("14", "}", Color::srgb(0.58, 0.77, 1.0)),
        ("15", "fn main() {", Color::srgb(0.58, 0.77, 1.0)),
        ("16", "    let developer = Developer::new(\"Muhammad Malek Alset\");", GOLD),
        ("17", "    println!(\"{developer}\");", Color::srgb(0.62, 0.87, 0.67)),
        ("18", "}", Color::srgb(0.58, 0.77, 1.0)),
    ];
    for (index, (number, code, color)) in code_lines.iter().enumerate() {
        let y = 77.0 - index as f32 * 9.0;
        spawn_code_line(&mut commands, font.clone(), number, code, y, *color);
    }

    
    
    for (index, x) in [-72.0_f32, 72.0_f32].iter().enumerate() {
        spawn_rect(&mut commands, Vec2::new(68.0, 40.0), Vec3::new(*x, -145.0, 18.0), Color::srgb(0.045, 0.085, 0.13));
        spawn_rect(&mut commands, Vec2::new(68.0, 1.5), Vec3::new(*x, -125.0, 19.0), CYAN);
        commands.spawn((
            ScreenUi,
            MenuItem(index),
            SocialIcon { selection_index: index },
            Transform::from_xyz(*x, -145.0, 21.0),
        )).with_children(|parent| {
            parent.spawn((
                ScreenUi,
                SocialIconLabel { selection_index: index },
                Text2d::new(if index == 0 { "GITHUB" } else { "LINKEDIN" }),
                TextFont { font: font.clone(), font_size: FontSize::Px(9.0), font_smoothing: FontSmoothing::None, ..default() },
                TextLayout::justify(Justify::Center),
                TextColor(WHITE),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });
    }
    commands.spawn((
        ScreenUi, MenuItem(3), Text2d::new(tr(language, "BACK")),
        TextFont { font: font.clone(), font_size: FontSize::Px(9.0), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(WHITE), Transform::from_xyz(0.0, -218.0, 20.0),
    ));
}

pub fn animate_social_icons(
    time: Res<Time>,
    selection: Res<MenuSelection>,
    mut icons: Query<(&SocialIcon, &mut Transform)>,
    mut labels: Query<(&SocialIconLabel, &mut TextColor)>,
) {
    let interpolation = (time.delta_secs() * 12.0).clamp(0.0, 1.0);
    for (icon, mut transform) in &mut icons {
        let target = if selection.selected == icon.selection_index { 1.08 } else { 1.0 };
        let scale = transform.scale.x + (target - transform.scale.x) * interpolation;
        transform.scale = Vec3::splat(scale);
    }

    for (label, mut color) in &mut labels {
        color.0 = if selection.selected == label.selection_index {
            GOLD
        } else {
            WHITE
        };
    }
}

pub fn spawn_developer_document(
    mut commands: Commands,
    settings: Res<Settings>,
    mut state: ResMut<DeveloperDocumentState>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
) {
    state.closing = false;
    state.close_elapsed = 0.0;
    selection.selected = 0;
    let font = ui_font.source.clone();
    let language = settings.language;
    
    
    spawn_rect(&mut commands, Vec2::new(960.0, 540.0), Vec3::new(0.0, 0.0, 30.0), Color::srgba(0.005, 0.008, 0.02, 0.93));
    spawn_rect(&mut commands, Vec2::new(470.0, 470.0), Vec3::new(0.0, 0.0, 31.0), Color::srgb(0.035, 0.055, 0.10));
    spawn_rect(&mut commands, Vec2::new(462.0, 2.0), Vec3::new(0.0, 233.0, 32.0), GOLD);
    spawn_rect(&mut commands, Vec2::new(462.0, 2.0), Vec3::new(0.0, -233.0, 32.0), CYAN);
    screen_header(&mut commands, font.clone(), "DEVELOPER DOCUMENT", "A FRIENDLY MATCH BETWEEN TWO HALVES OF ONE NAME", language);
    spawn_text(&mut commands, font.clone(), "P1 // Muhammad Malek", Vec2::new(-115.0, 112.0), 6.1, CYAN);
    spawn_text(&mut commands, font.clone(), "P2 // Alset", Vec2::new(115.0, 112.0), 6.1, GOLD);
    spawn_rect(&mut commands, Vec2::new(368.0, 150.0), Vec3::new(0.0, 24.0, 33.0), Color::srgb(0.008, 0.018, 0.035));
    spawn_rect(&mut commands, Vec2::new(2.0, 150.0), Vec3::new(-184.0, 24.0, 34.0), CYAN);
    spawn_rect(&mut commands, Vec2::new(2.0, 150.0), Vec3::new(184.0, 24.0, 34.0), GOLD);
    for row in 0..7 {
        spawn_rect(&mut commands, Vec2::new(366.0, 1.0), Vec3::new(0.0, -36.0 + row as f32 * 20.0, 34.0), Color::srgba(0.3, 0.5, 0.7, 0.12));
    }
    commands.spawn((ScreenUi, DeveloperPongPaddle { side: -1.0, follow_speed: 270.0 }, Sprite::from_color(CYAN, Vec2::new(8.0, 38.0)), Transform::from_xyz(-158.0, 24.0, 36.0)));
    commands.spawn((ScreenUi, DeveloperPongPaddle { side: 1.0, follow_speed: 82.0 }, Sprite::from_color(GOLD, Vec2::new(8.0, 38.0)), Transform::from_xyz(158.0, 24.0, 36.0)));
    commands.spawn((ScreenUi, DeveloperPongBall { velocity: Vec2::new(235.0, 128.0) }, Sprite::from_color(WHITE, Vec2::splat(8.0)), Transform::from_xyz(0.0, 24.0, 37.0)));
    spawn_text(&mut commands, font.clone(), tr(language, "PADDLES KEEP PLAYING  //  CLOSE DOCUMENT WHEN READY"), Vec2::new(0.0, -91.0), 6.2, DIM);
    commands.spawn((
        ScreenUi, MenuItem(0), Text2d::new(tr(language, "CLOSE DOCUMENT")),
        TextFont { font: font.clone(), font_size: FontSize::Px(8.5), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD), Transform::from_xyz(0.0, -188.0, 40.0),
    ));
}

pub fn update_developer_document(
    time: Res<Time>,
    mut state: ResMut<DeveloperDocumentState>,
    mut next_state: ResMut<NextState<crate::game::Screen>>,
    
    
    mut ball_query: Query<
        (&mut Transform, &mut DeveloperPongBall),
        (With<DeveloperPongBall>, Without<DeveloperPongPaddle>),
    >,
    mut paddles: Query<
        (&mut Transform, &DeveloperPongPaddle),
        (With<DeveloperPongPaddle>, Without<DeveloperPongBall>),
    >,
    mut document_transforms: Query<
        &mut Transform,
        (With<ScreenUi>, Without<DeveloperPongBall>, Without<DeveloperPongPaddle>),
    >,
) {
    let dt = time.delta_secs().min(0.05);
    if state.closing {
        state.close_elapsed += dt;
        let progress = (state.close_elapsed / 0.24).clamp(0.0, 1.0);
        let scale = 1.0 - progress * 0.90;
        for mut transform in &mut document_transforms {
            transform.scale = Vec3::splat(scale);
        }
        if progress >= 1.0 {
            next_state.set(crate::game::Screen::Credits);
        }
        return;
    }

    if let Ok((mut ball_transform, mut ball)) = ball_query.single_mut() {
        
        
        const ARENA_HALF_W: f32 = 184.0;
        const ARENA_HALF_H: f32 = 75.0;
        const BALL_RADIUS: f32 = 4.0;
        const PADDLE_HALF_HEIGHT: f32 = 19.0;
        const PADDLE_HALF_WIDTH: f32 = 4.0;
        const PADDLE_X: f32 = 158.0;
        let half_w = ARENA_HALF_W - BALL_RADIUS;
        let half_h = ARENA_HALF_H - BALL_RADIUS;
        let mut y = ball_transform.translation.y + ball.velocity.y * dt;
        if y > 24.0 + half_h { y = 24.0 + half_h; ball.velocity.y = -ball.velocity.y.abs(); }
        if y < 24.0 - half_h { y = 24.0 - half_h; ball.velocity.y = ball.velocity.y.abs(); }
        let previous_x = ball_transform.translation.x;
        let mut x = previous_x + ball.velocity.x * dt;
        for (mut paddle_transform, paddle) in &mut paddles {
            let paddle_vertical_limit = half_h - PADDLE_HALF_HEIGHT;
            let target = y.clamp(24.0 - paddle_vertical_limit, 24.0 + paddle_vertical_limit);
            let delta = (target - paddle_transform.translation.y).clamp(-paddle.follow_speed * dt, paddle.follow_speed * dt);
            paddle_transform.translation.y += delta;
            let paddle_x = paddle.side * PADDLE_X;
            let contact_x = PADDLE_HALF_WIDTH + BALL_RADIUS;
            let contact_plane = paddle_x - paddle.side * contact_x;
            
            
            let crossed_paddle_plane = crossed_paddle_contact(
                previous_x,
                x,
                ball.velocity.x,
                contact_plane,
                paddle.side,
            );
            if crossed_paddle_plane && (y - paddle_transform.translation.y).abs() <= PADDLE_HALF_HEIGHT + BALL_RADIUS {
                ball.velocity.x = -ball.velocity.x;
                ball.velocity.y += (y - paddle_transform.translation.y) * 1.8;
                x = paddle_x - paddle.side * (contact_x + 2.0);
            }
        }
        if x > half_w {
            x = half_w;
            ball.velocity.x = -ball.velocity.x.abs();
        } else if x < -half_w {
            x = -half_w;
            ball.velocity.x = ball.velocity.x.abs();
        }
        ball_transform.translation.x = x;
        ball_transform.translation.y = y;
    }
}



fn crossed_paddle_contact(previous_x: f32, next_x: f32, velocity_x: f32, plane_x: f32, side: f32) -> bool {
    if side < 0.0 {
        velocity_x < 0.0 && previous_x >= plane_x && next_x <= plane_x
    } else {
        velocity_x > 0.0 && previous_x <= plane_x && next_x >= plane_x
    }
}


fn spawn_developer_background(commands: &mut Commands, _font: FontSource, _language: crate::persistence::Language) {
    
    
    spawn_background(commands);
}

const CODE_LINE_NUMBER_X: f32 = -174.0;
const CODE_LINE_NUMBER_WIDTH: f32 = 15.0;
const CODE_LINE_NUMBER_GAP: f32 = 12.0;
const CODE_TEXT_X: f32 = CODE_LINE_NUMBER_X + CODE_LINE_NUMBER_WIDTH + CODE_LINE_NUMBER_GAP + 145.0;
const CODE_TEXT_WIDTH: f32 = 320.0;
const CODE_FONT_SIZE: f32 = 7.2;
const CODE_LINE_NUMBER_FONT_SIZE: f32 = 6.5;

fn spawn_code_line(commands: &mut Commands, font: FontSource, number: &str, code: &str, y: f32, color: Color) {
    commands.spawn((
        ScreenUi,
        Text2d::new(number.to_owned()),
        TextFont { font: font.clone(), font_size: FontSize::Px(CODE_LINE_NUMBER_FONT_SIZE), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Left),
        TextBounds::from(Vec2::new(CODE_LINE_NUMBER_WIDTH, 15.0)),
        TextColor(Color::srgb(0.48, 0.64, 0.78)),
        Transform::from_xyz(CODE_LINE_NUMBER_X, y, 22.0),
    ));
    commands.spawn((
        ScreenUi,
        Text2d::new(code.to_owned()),
        TextFont { font, font_size: FontSize::Px(CODE_FONT_SIZE), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(CODE_TEXT_WIDTH, 15.0)),
        TextColor(color),
        Transform::from_xyz(CODE_TEXT_X, y, 22.0),
    ));
}

pub fn spawn_pause_menu(mut commands: Commands, session: Res<MatchSession>, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>, overlays: Query<Entity, With<PauseOverlay>>) {
    selection.selected = 0;
    selection.pause_return_to = None;
    let language = settings.language;
    let networked = session.mode == crate::game::components::MatchMode::Networked;
    spawn_background(&mut commands);

    
    
    
    if overlays.is_empty() {
        commands.spawn((
            PauseOverlay,
            Sprite::from_color(Color::srgba(0.008, 0.015, 0.035, 0.78), Vec2::new(2000.0, 1200.0)),
            Transform::from_xyz(0.0, 0.0, 10.0),
        ));
    }

    if networked {
        screen_header(&mut commands, ui_font.source.clone(), "LOCAL PAUSE", "THE NETWORK MATCH KEEPS RUNNING", language);
    } else {
        screen_header(&mut commands, ui_font.source.clone(), "GAME PAUSED", "TAKE A BREATHER. THE BALL CAN WAIT.", language);
    }

    let local_multiplayer = matches!(
        session.mode,
        crate::game::components::MatchMode::LocalVersus | crate::game::components::MatchMode::Networked
    );
    let score_label = if local_multiplayer {
        format!("P1 {:02}        P2 {:02}", session.player_score, session.ai_score)
    } else {
        format!("{} {:02}        {} {:02}", tr(language, "YOU"), session.player_score, tr(language, "CPU"), session.ai_score)
    };
    spawn_text(&mut commands, ui_font.source.clone(), score_label, Vec2::new(0.0, 105.0), 12.0, GOLD);

    let items = if networked {
        ["RESUME", "QUICK SETTINGS", "SETTINGS", "LEAVE MATCH"].map(|key| tr(language, key)).to_vec()
    } else {
        ["RESUME", "RESTART MATCH", "QUICK SETTINGS", "POWER-UP INVENTORY", "SETTINGS", "MAIN MENU"]
            .map(|key| tr(language, key)).to_vec()
    };
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, 50.0, 34.0, 10.5);
    let hint = if networked { "LOCAL PAUSE HINT" } else { "ESC ALSO RESUMES" };
    spawn_text(&mut commands, ui_font.source.clone(), tr(language, hint), Vec2::new(0.0, -176.0), 7.5, DIM);
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), language, Vec2::new(0.0, -205.0), 6.6, DIM);
}

pub fn quick_settings_items(settings: &Settings) -> Vec<String> {
    let language = settings.language;
    let on_off = |value: bool| tr(language, if value { "ON" } else { "OFF" });
    vec![
        format!("{}  //  {}", tr(language, "MUSIC"), on_off(settings.music_enabled)),
        format!("{}  //  {:>3}%", tr(language, "MUSIC VOLUME"), (settings.music_volume * 100.0).round() as u32),
        format!("{}  //  {}", tr(language, "SOUND FX"), on_off(settings.sfx_enabled)),
        format!("{}  //  {:>3}%", tr(language, "FX VOLUME"), (settings.sfx_volume * 100.0).round() as u32),
        format!("{}  //  {}", tr(language, "REDUCED MOTION"), on_off(settings.reduce_motion)),
        tr(language, "BACK TO PAUSE"),
    ]
}

pub fn spawn_quick_settings(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_rect(&mut commands, Vec2::new(650.0, 380.0), Vec3::new(0.0, 0.0, 15.0), Color::srgba(0.025, 0.055, 0.095, 0.98));
    spawn_rect(&mut commands, Vec2::new(650.0, 2.0), Vec3::new(0.0, 189.0, 16.0), CYAN);
    spawn_rect(&mut commands, Vec2::new(650.0, 2.0), Vec3::new(0.0, -189.0, 16.0), GOLD);
    spawn_text(&mut commands, font.clone(), tr(language, "QUICK SETTINGS"), Vec2::new(0.0, 143.0), 20.0, CYAN);
    spawn_text(&mut commands, font.clone(), tr(language, "LEFT / RIGHT OR ENTER TO ADJUST"), Vec2::new(0.0, 110.0), 8.5, DIM);
    spawn_menu_items(&mut commands, font.clone(), &quick_settings_items(&settings), 65.0, 38.0, 10.0);
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -164.0), 7.0, DIM);
}

pub fn spawn_restart_confirmation(mut commands: Commands, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    let font = ui_font.source.clone();
    spawn_rect(&mut commands, Vec2::new(650.0, 300.0), Vec3::new(0.0, 0.0, 15.0), Color::srgba(0.025, 0.055, 0.095, 0.98));
    spawn_rect(&mut commands, Vec2::new(650.0, 2.0), Vec3::new(0.0, 149.0, 16.0), PINK);
    spawn_text(&mut commands, font.clone(), tr(language, "RESTART THIS MATCH?"), Vec2::new(0.0, 87.0), 20.0, PINK);
    spawn_text(&mut commands, font.clone(), tr(language, "YOUR CURRENT MATCH STATE WILL BE RESET."), Vec2::new(0.0, 43.0), 8.5, DIM);
    let items = ["CONFIRM RESTART", "BACK TO PAUSE"].map(|key| tr(language, key));
    spawn_menu_items(&mut commands, font.clone(), &items, -22.0, 52.0, 11.0);
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -118.0), 7.0, DIM);
}

pub fn spawn_results(mut commands: Commands, session: Res<MatchSession>, settings: Res<Settings>, mut selection: ResMut<MenuSelection>, ui_font: Res<UiFont>) {
    selection.selected = 0;
    let language = settings.language;
    spawn_background(&mut commands);
    let won = session.player_score >= session.winning_score;
    let local = matches!(session.mode, crate::game::components::MatchMode::LocalVersus | crate::game::components::MatchMode::Networked);
    let headline = match (local, won) {
        (true, true) => tr(language, "PLAYER 1 WINS!"),
        (true, false) => tr(language, "PLAYER 2 WINS!"),
        (false, true) => tr(language, "YOU WIN!"),
        (false, false) => tr(language, "GAME OVER"),
    };
    spawn_text(&mut commands, ui_font.source.clone(), headline, Vec2::new(0.0, 150.0), 27.0, if won { CYAN } else { PINK });
    spawn_text(&mut commands, ui_font.source.clone(), tr(language, "FINAL SCORE"), Vec2::new(0.0, 93.0), 9.0, DIM);
    let score = if local {
        format!("P1  {:02}        {:02}  P2", session.player_score, session.ai_score)
    } else {
        format!("{}  {:02}        {:02}  {}", tr(language, "YOU"), session.player_score, session.ai_score, tr(language, "CPU"))
    };
    spawn_text(&mut commands, ui_font.source.clone(), score, Vec2::new(0.0, 42.0), 16.0, GOLD);
    let result_message = if won {
        tr(language, "NICE RALLY. KEEP THE STREAK ALIVE.")
    } else {
        tr(language, "THE ARCADE IS WAITING FOR A REMATCH.")
    };
    spawn_text(
        &mut commands,
        ui_font.source.clone(),
        result_message,
        Vec2::new(0.0, -12.0),
        8.0,
        WHITE,
    );
    let items = ["PLAY AGAIN", "MAIN MENU"].map(|key| tr(language, key));
    spawn_menu_items(&mut commands, ui_font.source.clone(), &items, -94.0, 48.0, 12.0);
    spawn_navigation_hint(&mut commands, ui_font.source.clone(), language, Vec2::new(0.0, -204.0), 6.8, DIM);
}
