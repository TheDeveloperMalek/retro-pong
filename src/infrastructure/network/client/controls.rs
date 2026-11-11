

use super::NetworkRuntime;
use crate::network::protocol::ClientMessage;
use crate::{game::{Screen, components::{MatchMode, MatchSession}}, persistence::Settings};
use bevy::prelude::*;

const INPUT_UPDATES_PER_SECOND: f32 = 30.0;
pub fn network_controls(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    screen: Res<State<Screen>>,
    session: Res<MatchSession>,
    settings: Res<Settings>,
    mut runtime: ResMut<NetworkRuntime>,
) {
    if !runtime.connected {
        return;
    }

    let current_screen = *screen.get();
    if session.mode == MatchMode::Networked && runtime.match_started {
        if current_screen == Screen::MainMenu {
            
            
            runtime.send(ClientMessage::LeaveRoom);
            runtime.room_code = None;
            runtime.latest_snapshot = None;
            runtime.match_started = false;
            runtime.local_pause_active = false;
            return;
        }

        if current_screen == Screen::Playing {
            runtime.local_pause_active = false;
        } else if matches!(
            current_screen,
            Screen::Paused
                | Screen::QuickSettings
                | Screen::RestartConfirm
                | Screen::Inventory
                | Screen::Settings
                | Screen::Skins
                | Screen::PowerUpSettings
                | Screen::PowerUpEditor
                | Screen::Controls
                | Screen::GamepadControls
        ) {
            
            
            
            if !runtime.local_pause_active {
                runtime.send(ClientMessage::Input { vertical: 0 });
                runtime.local_pause_active = true;
            }
            return;
        }
    }

    if !is_network_match_active(current_screen, session.mode) {
        handle_lobby_shortcuts(&keyboard, &mut runtime);
        return;
    }

    send_match_input(time.delta_secs(), &keyboard, &gamepads, &mut runtime, &settings);
    send_power_up_activation(&keyboard, &gamepads, &runtime, &settings);
    send_ball_release(&keyboard, &gamepads, &runtime, &settings);
    send_match_shortcuts(&keyboard, &mut runtime);
}

fn is_network_match_active(screen: Screen, mode: MatchMode) -> bool {
    screen == Screen::Playing && mode == MatchMode::Networked
}

fn handle_lobby_shortcuts(keyboard: &ButtonInput<KeyCode>, runtime: &mut NetworkRuntime) {
    if keyboard.just_pressed(KeyCode::F3) {
        if let Some(invite_code) = runtime.pending_invite.take() {
            runtime.send(ClientMessage::JoinInvite { invite_code });
        }
    }

    if keyboard.just_pressed(KeyCode::F4) {
        runtime.send(ClientMessage::CancelQueue);
    }
    if keyboard.just_pressed(KeyCode::F5) {
        runtime.send(ClientMessage::ListRooms);
    }
    if keyboard.just_pressed(KeyCode::F6) {
        runtime.send(ClientMessage::ListPlayers);
    }
    if keyboard.just_pressed(KeyCode::F7) {
        send_friend_invitation(runtime);
    }
    if keyboard.just_pressed(KeyCode::F8) {
        runtime.send(ClientMessage::StartMatch);
    }
    if keyboard.just_pressed(KeyCode::F9) {
        runtime.send(ClientMessage::StartTournament);
    }
    if keyboard.just_pressed(KeyCode::F10) {
        leave_current_room(runtime);
    }
}

fn send_match_input(
    delta_seconds: f32,
    keyboard: &ButtonInput<KeyCode>,
    gamepads: &Query<(Entity, &Gamepad)>,
    runtime: &mut NetworkRuntime,
    settings: &Settings,
) {
    runtime.input_accumulator += delta_seconds;
    if runtime.input_accumulator < 1.0 / INPUT_UPDATES_PER_SECOND {
        return;
    }

    runtime.input_accumulator = 0.0;
    let (up_action, down_action, _, _) = network_player_actions(runtime.team);
    let up = settings.controls.key_code(up_action).is_some_and(|key| keyboard.pressed(key));
    let down = settings.controls.key_code(down_action).is_some_and(|key| keyboard.pressed(key));
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());
    let (pad_up, pad_down) = controllers.first().map(|(_, gamepad)| {
        let stick_y = gamepad.left_stick().y;
        (stick_y > 0.18 || settings.gamepad_controls.button_code(up_action).is_some_and(|button| gamepad.pressed(button)),
         stick_y < -0.18 || settings.gamepad_controls.button_code(down_action).is_some_and(|button| gamepad.pressed(button)))
    }).unwrap_or((false, false));
    let up = up || pad_up;
    let down = down || pad_down;
    let vertical = match (up, down) {
        (true, _) => 1,
        (false, true) => -1,
        (false, false) => 0,
    };

    runtime.send(ClientMessage::Input { vertical });
}

fn send_power_up_activation(
    keyboard: &ButtonInput<KeyCode>,
    gamepads: &Query<(Entity, &Gamepad)>,
    runtime: &NetworkRuntime,
    settings: &Settings,
) {
    let (_, _, first_slot_action, _) = network_player_actions(runtime.team);
    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());
    if let Some(slot) = (0..capacity).find(|slot| {
        settings.controls.key_code(first_slot_action + *slot).is_some_and(|key| keyboard.just_pressed(key))
            || controllers.first().is_some_and(|(_, gamepad)| settings.gamepad_controls.button_code(first_slot_action + *slot).is_some_and(|button| gamepad.just_pressed(button)))
    }) {
        runtime.send(ClientMessage::ActivatePowerUp { slot: slot as u8 });
    }
}

fn send_ball_release(keyboard: &ButtonInput<KeyCode>, gamepads: &Query<(Entity, &Gamepad)>, runtime: &NetworkRuntime, settings: &Settings) {
    let (_, _, _, release_action) = network_player_actions(runtime.team);
    let mut controllers: Vec<_> = gamepads.iter().collect();
    controllers.sort_by_key(|(entity, _)| entity.to_bits());
    let gamepad_release = controllers.first().is_some_and(|(_, gamepad)| settings.gamepad_controls.button_code(release_action).is_some_and(|button| gamepad.just_pressed(button)));
    if settings.controls.key_code(release_action).is_some_and(|key| keyboard.just_pressed(key)) || gamepad_release {
        runtime.send(ClientMessage::ReleaseBall);
    }
}



fn network_player_actions(team: u8) -> (usize, usize, usize, usize) {
    if team == 0 { (0, 1, 2, 14) } else { (7, 8, 9, 15) }
}

fn send_match_shortcuts(keyboard: &ButtonInput<KeyCode>, runtime: &mut NetworkRuntime) {
    if keyboard.just_pressed(KeyCode::F7) {
        send_friend_invitation(runtime);
    }
    if keyboard.just_pressed(KeyCode::F10) {
        leave_current_room(runtime);
    }
}

fn send_friend_invitation(runtime: &NetworkRuntime) {
    runtime.send(ClientMessage::InviteFriend {
        target_player_id: runtime.target_player_id.clone(),
    });
}

fn leave_current_room(runtime: &mut NetworkRuntime) {
    runtime.send(ClientMessage::LeaveRoom);
    runtime.room_code = None;
}


