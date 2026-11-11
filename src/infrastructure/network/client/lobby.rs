

use super::{localized_runtime_status, NetworkMenuState, NetworkRuntime};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{
    game::{components::*, Screen},
    localization::tr,
    network::protocol::{ClientMessage, RoomMode, RoomVisibility},
    persistence::{self, PersistentData, Settings},
    platform_links,
    ui::{NetworkLobbyDetailsText, NetworkLobbyStatusText, NetworkLobbyStatusIndicator},
};

pub fn network_lobby_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut selection: ResMut<MenuSelection>,
    mut navigation: ResMut<NavigationState>,
    gamepads: Query<&Gamepad>,
    mut form: ResMut<NetworkMenuState>,
    mut runtime: ResMut<NetworkRuntime>,
    settings: Res<Settings>,
    mut persistent: ResMut<PersistentData>,
    mut next_state: ResMut<NextState<Screen>>,
    
    
    mut menu_items: Query<
        (&MenuItem, &mut Text2d, &Transform),
        (Without<NetworkLobbyStatusText>, Without<NetworkLobbyDetailsText>),
    >,
    mut status_labels: Query<
        &mut Text2d,
        (With<NetworkLobbyStatusText>, Without<MenuItem>, Without<NetworkLobbyDetailsText>),
    >,
    mut detail_labels: Query<
        &mut Text2d,
        (With<NetworkLobbyDetailsText>, Without<NetworkLobbyStatusText>, Without<MenuItem>),
    >,
    mut status_indicator: Query<&mut Sprite, With<NetworkLobbyStatusIndicator>>,
) {
    let navigation_actions = crate::ui::read_navigation_actions(
        &keyboard,
        &gamepads,
        &settings.menu_navigation,
        &mut navigation,
        selection.selected >= 4,
    );
    if let Some(device) = navigation_actions.device {
        navigation.active_device = device;
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        navigation.active_device = ActiveInputDevice::Mouse;
    }
    if let Ok(window) = windows.single() {
        if let Some(position) = window.cursor_position() {
            if navigation.last_mouse_position.is_some_and(|previous| previous.distance_squared(position) > 1.0) {
                navigation.active_device = ActiveInputDevice::Mouse;
            }
            navigation.last_mouse_position = Some(position);
        } else {
            navigation.last_mouse_position = None;
        }
    }

    
    let mut clicked_action = false;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
            if let Some(cursor) = window.cursor_position() {
                if let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) {
                    for (item, _, transform) in &mut menu_items {
                        let index = item.0;
                        let hit = if index < 4 {
                            (-420.0..=-55.0).contains(&world.x)
                                && (world.y - transform.translation.y).abs() <= 13.0
                        } else {
                            (transform.translation.x - 104.0..=transform.translation.x + 104.0).contains(&world.x)
                                && (world.y - transform.translation.y).abs() <= 9.2
                        };
                        if hit {
                            selection.selected = index;
                            clicked_action = index >= 4;
                            break;
                        }
                    }
                }
            }
        }
    }

    
    
    if selection.selected < 4 {
        if navigation_actions.up {
            selection.selected = if selection.selected == 0 { 30 } else { selection.selected - 1 };
        }
        if navigation_actions.down {
            selection.selected = if selection.selected == 3 { 4 } else { selection.selected + 1 };
        }
    } else if selection.selected == 30 {
        if navigation_actions.up { selection.selected = 29; }
        if navigation_actions.down { selection.selected = 4; }
    } else {
        let relative = selection.selected - 4;
        let column = relative / 13;
        let row = relative % 13;
        if navigation_actions.up {
            let next_row = if row == 0 { 12 } else { row - 1 };
            selection.selected = 4 + column * 13 + next_row;
        }
        if navigation_actions.down {
            let next_row = if row == 12 { 0 } else { row + 1 };
            selection.selected = 4 + column * 13 + next_row;
        }
        if navigation_actions.left && column == 1 {
            selection.selected = 4 + row;
        }
        if navigation_actions.right && column == 0 {
            selection.selected = 4 + 13 + row;
        }
    }
    if keyboard.just_pressed(KeyCode::Tab) {
        selection.selected = (selection.selected + 1) % 31;
    }

    
    
    if selection.selected < 4 {
        let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        let clear_field = control && keyboard.just_pressed(KeyCode::KeyA);
        let paste_field = control && keyboard.just_pressed(KeyCode::KeyV);
        if clear_field {
            let field = match selection.selected {
                0 => &mut form.player_name,
                1 => &mut form.server_address,
                2 => &mut form.room_code,
                _ => &mut form.target_player_id,
            };
            field.clear();
        }
        if paste_field {
            if let Some(pasted) = platform_links::read_from_clipboard() {
                let (field, max_len) = match selection.selected {
                    0 => (&mut form.player_name, 16),
                    1 => (&mut form.server_address, 96),
                    2 => (&mut form.room_code, 24),
                    _ => (&mut form.target_player_id, 48),
                };
                for pasted_character in pasted.chars() {
                    if pasted_character.is_control() { continue; }
                    let character = if selection.selected >= 2 { pasted_character.to_ascii_uppercase() } else { pasted_character };
                    if field_char_allowed(selection.selected, character)
                        && field.len() + character.len_utf8() <= max_len
                    {
                        field.push(character);
                    }
                }
            } else {
                runtime.status = tr(settings.language, "SYSTEM CLIPBOARD PASTE IS NOT AVAILABLE").into();
            }
        }
        if keyboard.just_pressed(KeyCode::Backspace) {
            let field = match selection.selected {
                0 => &mut form.player_name,
                1 => &mut form.server_address,
                2 => &mut form.room_code,
                _ => &mut form.target_player_id,
            };
            field.pop();
        }
        let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        if !clear_field && !paste_field && !control {
            if let Some(character) = typed_character(&keyboard, shift) {
                let field = match selection.selected {
                    0 => &mut form.player_name,
                    1 => &mut form.server_address,
                    2 => &mut form.room_code,
                    _ => &mut form.target_player_id,
                };
                let max_len = match selection.selected { 0 => 16, 1 => 96, 2 => 24, _ => 48 };
                if field.len() < max_len && field_char_allowed(selection.selected, character) {
                    let character = if selection.selected >= 2 { character.to_ascii_uppercase() } else { character };
                    field.push(character);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || (navigation_actions.cancel && selection.selected >= 4) {
        next_state.set(Screen::MainMenu);
    }

    if clicked_action || navigation_actions.confirm {
        let endpoint = if runtime.embedded_server_started {
            "127.0.0.1:41123".to_string()
        } else {
            form.server_address.trim().to_string()
        };
        let player_name = if form.player_name.trim().is_empty() { "PLAYER".to_string() } else { form.player_name.trim().to_string() };
        let room_code = form.room_code.trim().to_string();
        let target_id = form.target_player_id.trim().to_string();
        runtime.target_player_id = if target_id.is_empty() { None } else { Some(target_id.clone()) };
        match selection.selected {
            4 => runtime.connect_or_send(endpoint, player_name, ClientMessage::QuickMatch { mode: RoomMode::Classic }),
            5 => runtime.connect_or_send(endpoint, player_name, ClientMessage::QuickMatch { mode: RoomMode::Teams }),
            6 => runtime.connect_or_send(endpoint, player_name, ClientMessage::QuickMatch { mode: RoomMode::Tournament }),
            7 => runtime.discover_and_join(player_name),
            8 => {
                runtime.host_on_this_pc(player_name, settings.clone());
                
                
                selection.selected = 9;
            },
            9 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Public, mode: RoomMode::Classic, winning_score: settings.winning_score,
            }),
            10 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Private, mode: RoomMode::Classic, winning_score: settings.winning_score,
            }),
            11 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Public, mode: RoomMode::Teams, winning_score: settings.winning_score,
            }),
            12 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Private, mode: RoomMode::Teams, winning_score: settings.winning_score,
            }),
            13 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Public, mode: RoomMode::Tournament, winning_score: settings.winning_score,
            }),
            14 => runtime.connect_or_send(endpoint, player_name, ClientMessage::CreateRoom {
                visibility: RoomVisibility::Private, mode: RoomMode::Tournament, winning_score: settings.winning_score,
            }),
            15 if !room_code.is_empty() => runtime.connect_or_send(endpoint, player_name, ClientMessage::JoinRoom {
                room_code, invite_code: None,
            }),
            16 if !room_code.is_empty() => runtime.connect_or_send(endpoint, player_name, ClientMessage::JoinInvite {
                invite_code: room_code,
            }),
            17 if runtime.pending_invite.is_some() => {
                if let Some(code) = runtime.pending_invite.take() { runtime.send(ClientMessage::JoinInvite { invite_code: code }); }
            },
            18 if runtime.connected => runtime.send(ClientMessage::CancelQueue),
            19 => runtime.connect_or_send(endpoint, player_name, ClientMessage::ListRooms),
            20 => runtime.connect_or_send(endpoint, player_name, ClientMessage::ListPlayers),
            21 if !target_id.is_empty() => runtime.connect_or_send(endpoint, player_name, ClientMessage::InviteFriend {
                target_player_id: Some(target_id),
            }),
            22 if runtime.connected => runtime.send(ClientMessage::StartMatch),
            23 if runtime.connected => runtime.send(ClientMessage::StartTournament),
            24 if runtime.connected => { runtime.send(ClientMessage::LeaveRoom); runtime.room_code = None; runtime.invite_code = None; },
            25 => match runtime.host_lan_address.as_deref() {
                Some(address) if platform_links::copy_to_clipboard(address) => runtime.status = tr(settings.language, "HOST ADDRESS COPIED TO CLIPBOARD").into(),
                Some(_) => runtime.status = tr(settings.language, "COULD NOT ACCESS THE SYSTEM CLIPBOARD").into(),
                None => runtime.status = tr(settings.language, "HOST A GAME FIRST TO GET A SHARE ADDRESS").into(),
            },
            26 => match runtime.room_code.as_deref() {
                Some(code) if platform_links::copy_to_clipboard(code) => runtime.status = tr(settings.language, "ROOM CODE COPIED TO CLIPBOARD").into(),
                Some(_) => runtime.status = tr(settings.language, "COULD NOT ACCESS THE SYSTEM CLIPBOARD").into(),
                None => runtime.status = tr(settings.language, "CREATE OR JOIN A ROOM FIRST").into(),
            },
            27 => match runtime.invite_code.as_deref() {
                Some(code) if platform_links::copy_to_clipboard(code) => runtime.status = tr(settings.language, "FRIEND INVITE CODE COPIED").into(),
                Some(_) => runtime.status = tr(settings.language, "COULD NOT ACCESS THE SYSTEM CLIPBOARD").into(),
                None => runtime.status = tr(settings.language, "CREATE A ROOM OR INVITE FIRST").into(),
            },
            28 => match runtime.player_id.as_deref() {
                Some(player_id) if platform_links::copy_to_clipboard(player_id) => runtime.status = tr(settings.language, "YOUR PLAYER ID COPIED").into(),
                Some(_) => runtime.status = tr(settings.language, "COULD NOT ACCESS THE SYSTEM CLIPBOARD").into(),
                None => runtime.status = tr(settings.language, "CONNECT TO A SERVER FIRST").into(),
            },
            29 => next_state.set(Screen::MainMenu),
            30 => next_state.set(Screen::NetworkGuide),
            15 | 16 => runtime.status = tr(settings.language, "ENTER A ROOM OR INVITE CODE IN THE LEFT PANEL").into(),
            17 => runtime.status = tr(settings.language, "NO PENDING INVITATION").into(),
            18 => runtime.status = tr(settings.language, "CONNECT TO A SERVER FIRST").into(),
            21 => runtime.status = tr(settings.language, "ENTER A PLAYER ID IN THE LEFT PANEL, THEN SELECT INVITE PLAYER").into(),
            22 | 23 | 24 => runtime.status = tr(settings.language, "CONNECT AND JOIN A ROOM FIRST").into(),
            _ => {}
        }
    }

    
    
    
    if clicked_action
        || navigation_actions.confirm
        || keyboard.just_pressed(KeyCode::Escape)
    {
        let preferences = &mut persistent.0.network_preferences;
        if preferences.player_name != form.player_name || preferences.server_address != form.server_address {
            preferences.player_name = form.player_name.trim().chars().take(16).collect();
            if preferences.player_name.is_empty() { preferences.player_name = "PLAYER".into(); }
            preferences.server_address = form.server_address.trim().chars().take(96).collect();
            if preferences.server_address.is_empty() { preferences.server_address = "127.0.0.1:41123".into(); }
            persistence::save(&persistent.0);
        }
    }

    
    
    for (item, mut text, _) in &mut menu_items {
        text.0 = match item.0 {
            0 => format!("{}  [ {} ]", tr(settings.language, "PLAYER NAME"), clipped_value(&form.player_name, "PLAYER", 22)),
            1 => format!("{}  [ {} ]", tr(settings.language, "SERVER ADDRESS"), clipped_value(&form.server_address, "127.0.0.1:41123", 42)),
            2 => format!("{}  [ {} ]", tr(settings.language, "ROOM / INVITE CODE"), clipped_value(&form.room_code, "EMPTY", 18)),
            3 => format!("{}  [ {} ]", tr(settings.language, "PLAYER ID TO INVITE"), clipped_value(&form.target_player_id, "EMPTY", 25)),
            _ => text.0.clone(),
        };
    }
    let waiting = !runtime.connected && (runtime.status.starts_with("CONNECTING") || runtime.status.starts_with("SEARCHING"));
    let indicator_color = if runtime.connected {
        Color::srgb(0.20, 0.88, 0.43)
    } else if waiting {
        Color::srgb(1.0, 0.72, 0.20)
    } else {
        Color::srgb(0.92, 0.24, 0.27)
    };
    for mut sprite in &mut status_indicator { sprite.color = indicator_color; }
    for mut label in &mut status_labels {
        let status = localized_runtime_status(&runtime.status, settings.language);
        label.0 = format!("{} // {}", if runtime.connected { tr(settings.language, "ONLINE") } else { tr(settings.language, "NETWORK") }, status.chars().take(84).collect::<String>());
    }
    let room = runtime.room_code.as_deref().unwrap_or("—");
    let invite = runtime.invite_code.as_deref().unwrap_or("—");
    let player = runtime.player_id.as_deref().unwrap_or("—");
    let room_count = runtime.rooms.len();
    let player_count = runtime.players.len();
    let share_address = runtime.host_lan_address.as_deref().map(str::to_string)
        .unwrap_or_else(|| tr(settings.language, "HOST TO SHOW LAN ADDRESS"));
    
    
    
    let room_choices = runtime.rooms.iter().take(3)
        .map(|room| {
            let mode = match room.mode {
                RoomMode::Classic => tr(settings.language, "CLASSIC"),
                RoomMode::Teams => "2V2".to_string(),
                RoomMode::Tournament => tr(settings.language, "TOURNAMENT"),
            };
            let visibility = match room.visibility {
                RoomVisibility::Public => tr(settings.language, "PUBLIC"),
                RoomVisibility::Private => tr(settings.language, "PRIVATE"),
            };
            let activity = if room.in_progress { " *" } else { "" };
            format!("{} {} {} {}/{}{}", room.room_code, mode, visibility, room.players.len(), room.max_players, activity)
        })
        .collect::<Vec<_>>().join("  |  ");
    let player_choices = runtime.players.iter().take(3)
        .map(|player| {
            let name = clipped_value(&player.display_name, "PLAYER", 11);
            let player_id = clipped_value(&player.player_id, "—", 7);
            let state = if player.connected { tr(settings.language, "ONLINE") } else { tr(settings.language, "OFFLINE") };
            format!("{name} #{player_id} T{} {state}", player.team + 1)
        })
        .collect::<Vec<_>>().join("  |  ");
    for mut label in &mut detail_labels {
        label.0 = format!(
            "{}: {room}  {}: {invite}  {}: {player}\n{}: {share_address}\n{}: {} // {room_choices}\n{}: {} // {player_choices}",
            tr(settings.language, "ROOM"), tr(settings.language, "INVITE"), tr(settings.language, "YOUR ID"),
            tr(settings.language, "SHARE ADDRESS"), tr(settings.language, "PUBLIC ROOMS"), room_count,
            tr(settings.language, "PLAYERS"), player_count,
        );
    }
}

fn clipped_value(value: &str, fallback: &str, max_chars: usize) -> String {
    let value = if value.trim().is_empty() { fallback } else { value };
    let mut characters = value.chars();
    let clipped: String = characters.by_ref().take(max_chars).collect();
    if characters.next().is_some() { format!("{clipped}…") } else { clipped }
}

pub(super) fn field_char_allowed(field: usize, character: char) -> bool {
    match field {
        0 => character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-'),
        1 => character.is_ascii_alphanumeric() || matches!(character, '.' | ':' | '-' | '_' | '[' | ']'),
        _ => character.is_ascii_alphanumeric() || matches!(character, '-' | '_'),
    }
}

fn typed_character(keyboard: &ButtonInput<KeyCode>, shift: bool) -> Option<char> {
    let keys = [
        (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'), (KeyCode::KeyC, 'c'), (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'), (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'), (KeyCode::KeyJ, 'j'), (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'), (KeyCode::KeyO, 'o'), (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'), (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'), (KeyCode::KeyV, 'v'), (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'),
    ];
    for (key, character) in keys {
        if keyboard.just_pressed(key) { return Some(if shift { character.to_ascii_uppercase() } else { character }); }
    }
    for (key, character) in [
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'), (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
    ] {
        if keyboard.just_pressed(key) { return Some(character); }
    }
    for (key, character) in [
        (KeyCode::Period, '.'), (KeyCode::Minus, if shift { '_' } else { '-' }),
        (KeyCode::Semicolon, if shift { ':' } else { ';' }), (KeyCode::Space, ' '), (KeyCode::BracketLeft, '['),
        (KeyCode::BracketRight, ']'),
    ] {
        if keyboard.just_pressed(key) { return Some(character); }
    }
    None
}

