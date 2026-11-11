

use bevy::prelude::*;
use crate::{
    audio::{self, Sfx},
    game::{components::*, Screen},
    persistence::{save, PersistentData, Settings},
    progression,
};
use super::world::paddle_skin_sprite;

pub fn pause_on_focus_loss(
    windows: Query<&Window>,
    session: Res<MatchSession>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if !session.active { return; }
    if let Ok(window) = windows.single() {
        if !window.focused { next_state.set(Screen::Paused); }
    }
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<Settings>,
    gamepads: Query<&Gamepad>,
    session: Res<MatchSession>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    let pause_button = settings.gamepad_controls.button_code(16);
    let gamepad_pause = pause_button.is_some_and(|button| gamepads.iter().any(|gamepad| gamepad.just_pressed(button)));
    if session.active
        && (settings.controls.key_code(16).is_some_and(|key| keyboard.just_pressed(key)) || gamepad_pause)
    {
        next_state.set(Screen::Paused);
    }
}

pub fn finish_match(mut commands: Commands, asset_server: Res<AssetServer>, mut session: ResMut<MatchSession>, settings: Res<Settings>, mut persistent: ResMut<PersistentData>) {
    if session.result_saved { return; }
    session.result_saved = true;
    session.active = false;
    let won = if session.mode == MatchMode::OpenPlay { session.player_score > session.ai_score } else { session.player_score >= session.winning_score };
    let result_sfx = if session.mode == MatchMode::LocalVersus || won { Sfx::Win } else { Sfx::Lose };
    audio::play_sfx(&mut commands, &asset_server, &settings, result_sfx);
    let match_type = match session.mode {
        MatchMode::SinglePlayer => crate::persistence::SavedMatchType::SoloAi,
        MatchMode::LocalVersus => crate::persistence::SavedMatchType::LocalVersus,
        MatchMode::Networked => crate::persistence::SavedMatchType::Network,
        MatchMode::OpenPlay => crate::persistence::SavedMatchType::OpenPlay,
    };
    progression::record_match(&mut persistent.0, session.player_score, session.ai_score, settings.difficulty, won, match_type);
    save(&persistent.0);
}

pub fn cleanup_game(mut commands: Commands, entities: Query<Entity, With<GameEntity>>) {
    for entity in &entities { commands.entity(entity).despawn(); }
}

pub fn refresh_paddle_skin(
    paddles: &mut Query<(&mut Sprite, &Paddle), (With<Paddle>, Without<Ball>)>,
    settings: &Settings,
    asset_server: &AssetServer,
) {
    for (mut sprite, paddle) in paddles.iter_mut() {
        if matches!(paddle.side, Side::Player | Side::PlayerTwo) {
            *sprite = paddle_skin_sprite(settings, asset_server);
        }
    }
}
