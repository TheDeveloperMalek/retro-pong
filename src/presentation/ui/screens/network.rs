
use bevy::prelude::*;
use bevy::text::{LineBreak, TextBounds};
#[cfg(not(target_arch = "wasm32"))]
use crate::game::components::MenuItem;
use crate::{
    game::components::{MenuSelection, ScreenUi},
    localization::tr,
    persistence::Settings,
    ui::UiFont,
};
use super::common::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_network_lobby(
    mut commands: Commands,
    settings: Res<Settings>,
    mut selection: ResMut<MenuSelection>,
    ui_font: Res<UiFont>,
    asset_server: Res<AssetServer>,
) {
    selection.selected = 0;
    let font = ui_font.source.clone();
    spawn_background(&mut commands);
    
    spawn_text(&mut commands, font.clone(), tr(settings.language, "NETWORK MULTIPLAYER"), Vec2::new(0.0, 216.0), 23.0, CYAN);
    spawn_text(&mut commands, font.clone(), tr(settings.language, "SET UP ONCE, THEN PLAY FROM THIS SCREEN"), Vec2::new(0.0, 187.0), 9.5, DIM);

    spawn_rect(&mut commands, Vec2::new(386.0, 410.0), Vec3::new(-238.0, -12.0, 10.0), Color::srgb(0.025, 0.075, 0.115));
    spawn_rect(&mut commands, Vec2::new(1.5, 410.0), Vec3::new(-432.0, -12.0, 11.0), CYAN);
    spawn_rect(&mut commands, Vec2::new(418.0, 425.0), Vec3::new(210.0, -12.0, 10.0), Color::srgb(0.025, 0.055, 0.105));
    spawn_rect(&mut commands, Vec2::new(1.5, 425.0), Vec3::new(0.0, -12.0, 11.0), GOLD);

    spawn_text(&mut commands, font.clone(), tr(settings.language, "CONNECTION DETAILS  //  SELECT A FIELD AND TYPE"), Vec2::new(-238.0, 151.0), 7.2, GOLD);
    let fields = [
        format!("{}  [PLAYER]", tr(settings.language, "PLAYER NAME")),
        format!("{}  [127.0.0.1:41123]", tr(settings.language, "SERVER ADDRESS")),
        format!("{}  [EMPTY]", tr(settings.language, "ROOM / INVITE CODE")),
        format!("{}  [EMPTY]", tr(settings.language, "PLAYER ID TO INVITE")),
    ];
    for (index, label) in fields.iter().enumerate() {
        let y = 94.0 - index as f32 * 49.0;
        commands.spawn((
            ScreenUi,
            MenuItem(index),
            Text2d::new(label.clone()),
            TextFont { font: font.clone(), font_size: FontSize::Px(7.4), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
            TextBounds::from(Vec2::new(342.0, 22.0)),
            TextColor(WHITE),
            Transform::from_xyz(-238.0, y, 22.0),
        ));
    }
    spawn_text(&mut commands, font.clone(), tr(settings.language, "STATUS"), Vec2::new(-238.0, -91.0), 7.8, GOLD);
    spawn_rect(&mut commands, Vec2::new(350.0, 42.0), Vec3::new(-238.0, -123.0, 17.0), Color::srgb(0.035, 0.065, 0.105));
    spawn_rect(&mut commands, Vec2::new(350.0, 1.5), Vec3::new(-238.0, -102.0, 18.0), CYAN);
    commands.spawn((
        ScreenUi, NetworkLobbyStatusIndicator,
        Sprite::from_color(Color::srgb(0.9, 0.22, 0.24), Vec2::splat(8.0)),
        Transform::from_xyz(-397.0, -123.0, 23.0),
    ));
    commands.spawn((
        ScreenUi, NetworkLobbyStatusText, Text2d::new(tr(settings.language, "OFFLINE // READY")),
        TextFont { font: font.clone(), font_size: FontSize::Px(7.4), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter), TextBounds::from(Vec2::new(250.0, 20.0)), TextColor(CYAN), Transform::from_xyz(-250.0, -125.0, 22.0),
    ));
    commands.spawn((
        ScreenUi, NetworkLobbyDetailsText,
        Text2d::new(format!("{}: —   {}: —   {}: —", tr(settings.language, "ROOM"), tr(settings.language, "INVITE"), tr(settings.language, "YOUR ID"))),
        TextFont { font: font.clone(), font_size: FontSize::Px(6.8), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter), TextBounds::from(Vec2::new(342.0, 24.0)), TextColor(DIM), Transform::from_xyz(-238.0, -151.0, 22.0),
    ));
    commands.spawn((
        ScreenUi,
        Text2d::new(tr(settings.language, "TIP: HOST ON THIS PC FOR LAN; FRIENDS JOIN USING YOUR ADDRESS.")),
        TextFont { font: font.clone(), font_size: FontSize::Px(6.8), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(356.0, 24.0)),
        TextColor(DIM),
        Transform::from_xyz(-238.0, -202.0, 22.0),
    ));

    spawn_text(&mut commands, font.clone(), tr(settings.language, "PLAY / HOST / CREATE"), Vec2::new(108.0, 157.0), 7.8, GOLD);
    spawn_text(&mut commands, font.clone(), tr(settings.language, "JOIN / MANAGE / SHARE"), Vec2::new(312.0, 157.0), 7.8, GOLD);
    let actions = [
        "QUICK MATCH",
        "QUICK TEAM MATCH  (2 VS 2)",
        "QUICK TOURNAMENT",
        "FIND A GAME ON LAN",
        "HOST A GAME ON THIS PC",
        "CREATE PUBLIC ROOM  (CLASSIC)",
        "CREATE PRIVATE ROOM  (CLASSIC)",
        "CREATE PUBLIC TEAM ROOM  (2 VS 2)",
        "CREATE PRIVATE TEAM ROOM  (2 VS 2)",
        "CREATE PUBLIC TOURNAMENT ROOM",
        "CREATE PRIVATE TOURNAMENT ROOM",
        "JOIN ROOM CODE",
        "JOIN FRIEND INVITE CODE",
        "ACCEPT PENDING INVITE",
        "CANCEL MATCHMAKING",
        "REFRESH PUBLIC ROOMS",
        "REFRESH ONLINE PLAYERS",
        "INVITE PLAYER ID",
        "START ROOM MATCH  (HOST)",
        "START TOURNAMENT  (HOST)",
        "LEAVE ROOM",
        "COPY HOST ADDRESS",
        "COPY ROOM CODE",
        "COPY FRIEND INVITE CODE",
        "COPY YOUR PLAYER ID",
        "BACK TO MAIN MENU",
    ];
    let action_icons = [
        "quick_match", "team", "trophy", "lan", "server", "public_room", "private_room",
        "team", "private_room", "trophy", "private_room", "join_code", "invite",
        "inbox", "stop", "refresh", "group", "invite", "play", "trophy", "exit",
        "link", "key", "invite", "player_id", "home",
    ];
    for (index, label) in actions.iter().enumerate() {
        let column = index / 13;
        let row = index % 13;
        let x = if column == 0 { 108.0 } else { 312.0 };
        let y = 127.0 - row as f32 * 19.2;
        
        
        spawn_rect(&mut commands, Vec2::new(194.0, 18.0), Vec3::new(x, y, 15.0),
            if index % 2 == 0 { Color::srgb(0.035, 0.078, 0.12) } else { Color::srgb(0.027, 0.062, 0.10) });
        spawn_rect(&mut commands, Vec2::new(2.0, 12.0), Vec3::new(x - 92.0, y, 16.0),
            if index < 11 { CYAN } else if index < 15 { GOLD } else { Color::srgb(0.38, 0.58, 0.78) });
        let icon_path = format!("ui/multiplayer/{}.png", action_icons[index]);
        let mut icon = Sprite::from_image(asset_server.load(icon_path));
        icon.custom_size = Some(Vec2::splat(11.0));
        commands.spawn((ScreenUi, icon, Transform::from_xyz(x - 81.0, y, 21.0)));
        commands.spawn((
            ScreenUi,
            MenuItem(index + 4),
            Text2d::new(if index == 25 { tr(settings.language, "EXIT") } else { tr(settings.language, label) }),
            TextFont { font: font.clone(), font_size: FontSize::Px(7.4), font_smoothing: FontSmoothing::None, ..default() },
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
            TextBounds::from(Vec2::new(174.0, 16.0)),
            TextColor(WHITE),
            Transform::from_xyz(if index == 25 { x + 26.0 } else { x + 7.0 }, y, 22.0),
        ));
        if index == 25 {
            spawn_left_pixel_arrow(&mut commands, Vec2::new(x - 18.0, y));
        }
    }
    
    spawn_rect(&mut commands, Vec2::new(410.0, 48.0), Vec3::new(210.0, -194.0, 17.0), Color::srgb(0.035, 0.07, 0.115));
    spawn_rect(&mut commands, Vec2::new(410.0, 1.5), Vec3::new(210.0, -170.5, 18.0), CYAN);
    commands.spawn((
        ScreenUi, crate::game::components::ContextHelpTitle,
        Text2d::new(crate::localization::network_context_help(settings.language, 0).0),
        TextFont { font: font.clone(), font_size: FontSize::Px(7.5), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD), Transform::from_xyz(210.0, -180.0, 22.0),
    ));
    commands.spawn((
        ScreenUi, crate::game::components::ContextHelpBody,
        Text2d::new(crate::localization::network_context_help(settings.language, 0).1),
        TextFont { font: font.clone(), font_size: FontSize::Px(6.2), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter), TextBounds::from(Vec2::new(390.0, 22.0)), TextColor(WHITE), Transform::from_xyz(210.0, -200.0, 22.0),
    ));

    spawn_rect(&mut commands, Vec2::new(220.0, 27.0), Vec3::new(210.0, -146.0, 17.0), Color::srgb(0.055, 0.10, 0.16));
    spawn_rect(&mut commands, Vec2::new(220.0, 1.5), Vec3::new(210.0, -132.5, 18.0), CYAN);
    commands.spawn((
        ScreenUi, MenuItem(30), Text2d::new(tr(settings.language, "MULTIPLAYER GUIDE")),
        TextFont { font: font.clone(), font_size: FontSize::Px(7.2), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD), Transform::from_xyz(210.0, -147.0, 22.0),
    ));
    spawn_navigation_hint(&mut commands, font, settings.language, Vec2::new(0.0, -240.0), 5.8, DIM);
}



#[cfg(not(target_arch = "wasm32"))]
fn spawn_left_pixel_arrow(commands: &mut Commands, center: Vec2) {
    
    for (dx, dy) in [(-4.0_f32, 0.0_f32), (-2.0, 2.0), (-2.0, -2.0), (0.0, 0.0), (2.0, 0.0), (4.0, 0.0)] {
        spawn_rect(commands, Vec2::splat(2.4), Vec3::new(center.x + dx, center.y + dy, 23.0), CYAN);
    }
}


pub fn spawn_network_guide(
    mut commands: Commands,
    settings: Res<Settings>,
    mut selection: ResMut<crate::game::components::MenuSelection>,
    ui_font: Res<UiFont>,
) {
    selection.selected = 0;
    let font = ui_font.source.clone();
    let language = settings.language;
    spawn_background(&mut commands);
    spawn_text(&mut commands, font.clone(), tr(language, "MULTIPLAYER GUIDE"), Vec2::new(0.0, 216.0), 23.0, CYAN);
    spawn_text(&mut commands, font.clone(), tr(language, "FOLLOW THE STEPS FOR YOUR CONNECTION TYPE"), Vec2::new(0.0, 181.0), 9.5, DIM);
    let labels = (0..8).map(|step| crate::localization::multiplayer_guide_title(language, step)).chain([tr(language, "BACK")]).collect::<Vec<_>>();
    spawn_menu_items_at_x(&mut commands, font.clone(), &labels, -360.0, 110.0, 28.0, 7.4);
    spawn_rect(&mut commands, Vec2::new(390.0, 320.0), Vec3::new(225.0, -5.0, 10.0), Color::srgb(0.025, 0.065, 0.105));
    spawn_rect(&mut commands, Vec2::new(390.0, 2.0), Vec3::new(225.0, 155.0, 11.0), CYAN);
    commands.spawn((
        ScreenUi, crate::game::components::ContextHelpTitle,
        Text2d::new(crate::localization::multiplayer_guide_title(language, 0)),
        TextFont { font: font.clone(), font_size: FontSize::Px(10.2), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::justify(Justify::Center), TextColor(GOLD), Transform::from_xyz(225.0, 112.0, 20.0),
    ));
    commands.spawn((
        ScreenUi, crate::game::components::ContextHelpBody,
        Text2d::new(crate::localization::multiplayer_guide_body(language, 0)),
        TextFont { font: font.clone(), font_size: FontSize::Px(7.4), font_smoothing: FontSmoothing::None, ..default() },
        TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        TextBounds::from(Vec2::new(356.0, 112.0)),
        TextColor(WHITE), Transform::from_xyz(225.0, -8.0, 20.0),
    ));
    
    
    
    for step in 0..8 {
        spawn_network_guide_diagram(&mut commands, step);
    }
    spawn_navigation_hint(&mut commands, font, language, Vec2::new(0.0, -205.0), 6.5, DIM);
}

fn spawn_network_guide_diagram(commands: &mut Commands, step: usize) {
    use crate::game::components::NetworkGuideIllustration;
    let accent = if step == 6 { CYAN } else if step == 7 { GOLD } else { Color::srgb(0.48, 0.75, 1.0) };
    let bg = Color::srgb(0.035, 0.09, 0.14);
    let node = |commands: &mut Commands, x: f32, y: f32, width: f32, height: f32, color: Color| {
        commands.spawn((
            ScreenUi, NetworkGuideIllustration(step),
            Sprite::from_color(bg, Vec2::new(width, height)),
            Transform::from_xyz(x, y, 20.0), Visibility::Hidden,
        ));
        commands.spawn((
            ScreenUi, NetworkGuideIllustration(step),
            Sprite::from_color(color, Vec2::new(width - 8.0, 3.0)),
            Transform::from_xyz(x, y + height * 0.5 - 5.0, 21.0), Visibility::Hidden,
        ));
        commands.spawn((
            ScreenUi, NetworkGuideIllustration(step),
            Sprite::from_color(Color::srgb(0.08, 0.17, 0.24), Vec2::new(width - 16.0, height - 18.0)),
            Transform::from_xyz(x, y - 3.0, 21.0), Visibility::Hidden,
        ));
    };
    let link = |commands: &mut Commands, from: Vec2, to: Vec2| {
        let delta = to - from;
        let midpoint = (from + to) * 0.5;
        commands.spawn((
            ScreenUi, NetworkGuideIllustration(step),
            Sprite::from_color(accent, Vec2::new(delta.length(), 2.0)),
            Transform::from_xyz(midpoint.x, midpoint.y, 19.0)
                .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
            Visibility::Hidden,
        ));
    };

    match step {
        0 => { 
            node(commands, 145.0, -112.0, 54.0, 42.0, CYAN);
            node(commands, 305.0, -112.0, 62.0, 42.0, GOLD);
            link(commands, Vec2::new(176.0, -112.0), Vec2::new(272.0, -112.0));
            node(commands, 342.0, -144.0, 38.0, 14.0, GOLD);
            link(commands, Vec2::new(305.0, -134.0), Vec2::new(326.0, -141.0));
        }
        1 => { 
            node(commands, 225.0, -108.0, 60.0, 44.0, GOLD);
            node(commands, 130.0, -134.0, 42.0, 30.0, CYAN);
            node(commands, 320.0, -134.0, 42.0, 30.0, CYAN);
            link(commands, Vec2::new(164.0, -125.0), Vec2::new(195.0, -113.0));
            link(commands, Vec2::new(255.0, -113.0), Vec2::new(286.0, -125.0));
        }
        2 => { 
            node(commands, 145.0, -112.0, 58.0, 42.0, CYAN);
            node(commands, 225.0, -112.0, 58.0, 42.0, GOLD);
            node(commands, 305.0, -112.0, 58.0, 42.0, PINK);
        }
        3 => { 
            node(commands, 150.0, -112.0, 58.0, 42.0, CYAN);
            node(commands, 300.0, -92.0, 72.0, 25.0, GOLD);
            node(commands, 300.0, -136.0, 72.0, 25.0, PINK);
            link(commands, Vec2::new(181.0, -112.0), Vec2::new(260.0, -92.0));
            link(commands, Vec2::new(181.0, -112.0), Vec2::new(260.0, -136.0));
        }
        4 | 5 => { 
            node(commands, 140.0, -112.0, 54.0, 40.0, CYAN);
            node(commands, 225.0, -112.0, 54.0, 32.0, if step == 4 { GOLD } else { PINK });
            node(commands, 310.0, -112.0, 54.0, 40.0, GOLD);
            link(commands, Vec2::new(169.0, -112.0), Vec2::new(196.0, -112.0));
            link(commands, Vec2::new(254.0, -112.0), Vec2::new(281.0, -112.0));
        }
        6 => { 
            node(commands, 225.0, -112.0, 62.0, 28.0, CYAN);
            node(commands, 130.0, -137.0, 42.0, 26.0, GOLD);
            node(commands, 320.0, -137.0, 42.0, 26.0, GOLD);
            link(commands, Vec2::new(192.0, -118.0), Vec2::new(151.0, -130.0));
            link(commands, Vec2::new(258.0, -118.0), Vec2::new(299.0, -130.0));
        }
        _ => { 
            node(commands, 130.0, -112.0, 48.0, 36.0, CYAN);
            node(commands, 225.0, -112.0, 58.0, 42.0, GOLD);
            node(commands, 320.0, -112.0, 48.0, 36.0, PINK);
            link(commands, Vec2::new(157.0, -112.0), Vec2::new(194.0, -112.0));
            link(commands, Vec2::new(256.0, -112.0), Vec2::new(293.0, -112.0));
        }
    }
}

pub fn update_network_guide(
    settings: Res<Settings>,
    selection: Res<MenuSelection>,
    mut titles: Query<&mut Text2d, (With<crate::game::components::ContextHelpTitle>, Without<crate::game::components::ContextHelpBody>)>,
    mut bodies: Query<&mut Text2d, (With<crate::game::components::ContextHelpBody>, Without<crate::game::components::ContextHelpTitle>)>,
    mut diagrams: Query<(&crate::game::components::NetworkGuideIllustration, &mut Visibility)>,
) {
    let step = selection.selected.min(7);
    for mut title in &mut titles { title.0 = crate::localization::multiplayer_guide_title(settings.language, step); }
    for mut body in &mut bodies { body.0 = crate::localization::multiplayer_guide_body(settings.language, step); }
    for (diagram, mut visibility) in &mut diagrams {
        *visibility = if diagram.0 == step { Visibility::Visible } else { Visibility::Hidden };
    }
}
