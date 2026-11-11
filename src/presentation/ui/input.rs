use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
use bevy::ecs::system::SystemParam;
use bevy::window::PrimaryWindow;
use crate::{
    audio::{self, Sfx},
    game::{components::{ActiveInputDevice, InputContext, NavigationState, MainMenuGithubBar, MainMenuLabel, MenuItem, MenuSelection, MatchSession, Paddle, PendingPowerUp, PowerUpEditorSelection, PowerUpInventory, Side, ControlCapture, SkinSelection, HighScoreFilters, LanguageDropdownState, LanguageDropdownItem, LanguageDropdownPanel, DeveloperDocumentState}, Screen},
    persistence::{gamepad_button_name, key_name, key_code_from_name, Difficulty, PersistentData, Settings, ControlBindings, GamepadControlBindings, MenuNavigationBindings},
    localization::tr,
    platform_links,
    ui::UiFont,
};
use super::screens::{control_action_for_row, menu_navigation_action_for_row, inventory_management_items, is_powerup_editor_row_selectable, powerup_editor_items, powerup_settings_items, settings_items, refresh_language_flag};
use super::screens::show_notice;

mod settings;
use settings::{
    adjust_selected, apply_live_display_settings, menu_count, persist_settings, reset_powerup_item_settings,
};

const GITHUB_REPOSITORY_URL: &str = "https://github.com/TheDeveloperMalek/retro-pong";

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum SkinImportError {
    Cancelled,
    Decode,
    Dimensions(u32, u32, u32, u32),
    UnsupportedFormat,
    Storage,
}

#[cfg(not(target_arch = "wasm32"))]
fn import_custom_skin(is_paddle: bool) -> Result<String, SkinImportError> {
    use std::fs;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::io::Cursor;

    let Some(source) = rfd::FileDialog::new()
        .set_title(if is_paddle { "Choose a paddle skin" } else { "Choose a ball skin" })
        .add_filter("PNG / JPEG / WebP", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
    else {
        return Err(SkinImportError::Cancelled);
    };
    let extension = source.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(SkinImportError::UnsupportedFormat)?;
    if !["png", "jpg", "jpeg", "webp"].contains(&extension.as_str()) {
        return Err(SkinImportError::UnsupportedFormat);
    }

    
    
    let decoded = image::open(&source).map_err(|_| SkinImportError::Decode)?;
    let (width, height) = (decoded.width(), decoded.height());
    let (expected_width, expected_height) = if is_paddle { (15, 88) } else { (13, 13) };
    if (width, height) != (expected_width, expected_height) {
        return Err(SkinImportError::Dimensions(width, height, expected_width, expected_height));
    }

    let mut png = Cursor::new(Vec::new());
    decoded.write_to(&mut png, image::ImageFormat::Png).map_err(|_| SkinImportError::Decode)?;
    let bytes = png.into_inner();
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let digest = hasher.finish();
    let directory = PathBuf::from("assets").join("custom_skins");
    fs::create_dir_all(&directory).map_err(|_| SkinImportError::Storage)?;
    let object = if is_paddle { "paddle" } else { "ball" };
    let destination = directory.join(format!("{object}_{digest:016x}.png"));
    fs::write(&destination, bytes).map_err(|_| SkinImportError::Storage)?;
    Ok(destination.strip_prefix(Path::new("assets"))
        .unwrap_or(&destination)
        .to_string_lossy()
        .replace('\\', "/"))
}

#[cfg(not(target_arch = "wasm32"))]
fn skin_import_error_message(error: SkinImportError, language: crate::persistence::Language) -> String {
    match error {
        SkinImportError::Cancelled => tr(language, "IMAGE IMPORT CANCELLED"),
        SkinImportError::Decode => tr(language, "IMAGE DECODE FAILED"),
        SkinImportError::Dimensions(width, height, expected_width, expected_height) => format!(
            "{}: {}×{} / {}×{}",
            tr(language, "IMAGE DIMENSIONS INVALID"), width, height, expected_width, expected_height
        ),
        SkinImportError::UnsupportedFormat => tr(language, "IMAGE FORMAT UNSUPPORTED"),
        SkinImportError::Storage => tr(language, "IMAGE SAVE FAILED"),
    }
}

fn adjust_skin_selection(settings: &mut Settings, selection: &mut SkinSelection, row: usize, direction: i32) {
    match row {
        0 => selection.tab = ((selection.tab as i32 + direction).rem_euclid(2)) as u8,
        1 if selection.tab == 0 => {
            settings.paddle_skin = (settings.paddle_skin as i32 + direction).rem_euclid(13) as u8;
            settings.paddle_custom_image = None;
        }
        1 => {
            settings.ball_skin = (settings.ball_skin as i32 + direction).rem_euclid(13) as u8;
            settings.ball_custom_image = None;
        }
        2 if selection.tab == 0 && settings.paddle_custom_image.is_none() && settings.paddle_skin < 10 =>
            settings.paddle_shade = (settings.paddle_shade + direction as i8).clamp(-4, 4),
        2 if selection.tab == 1 && settings.ball_custom_image.is_none() && settings.ball_skin < 10 =>
            settings.ball_shade = (settings.ball_shade + direction as i8).clamp(-4, 4),
        _ => {},
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct NavigationActions {
    pub(crate) up: bool,
    pub(crate) down: bool,
    pub(crate) left: bool,
    pub(crate) right: bool,
    pub(crate) confirm: bool,
    pub(crate) cancel: bool,
    pub(crate) device: Option<ActiveInputDevice>,
}

fn is_text_entry_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::KeyA | KeyCode::KeyB | KeyCode::KeyC | KeyCode::KeyD | KeyCode::KeyE
            | KeyCode::KeyF | KeyCode::KeyG | KeyCode::KeyH | KeyCode::KeyI | KeyCode::KeyJ
            | KeyCode::KeyK | KeyCode::KeyL | KeyCode::KeyM | KeyCode::KeyN | KeyCode::KeyO
            | KeyCode::KeyP | KeyCode::KeyQ | KeyCode::KeyR | KeyCode::KeyS | KeyCode::KeyT
            | KeyCode::KeyU | KeyCode::KeyV | KeyCode::KeyW | KeyCode::KeyX | KeyCode::KeyY
            | KeyCode::KeyZ | KeyCode::Digit0 | KeyCode::Digit1 | KeyCode::Digit2
            | KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 | KeyCode::Digit6
            | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9 | KeyCode::Space
            | KeyCode::Period | KeyCode::Minus | KeyCode::Semicolon | KeyCode::BracketLeft
            | KeyCode::BracketRight | KeyCode::Backspace
    )
}

fn key_binding_pressed(
    input: &ButtonInput<KeyCode>,
    bindings: &MenuNavigationBindings,
    action: usize,
    allow_text_keys: bool,
) -> bool {
    bindings.binding(action)
        .and_then(key_code_from_name)
        .filter(|key| allow_text_keys || !is_text_entry_key(*key))
        .is_some_and(|key| input.just_pressed(key))
}




fn built_in_key_allowed(bindings: &MenuNavigationBindings, key_name: &str, action: usize) -> bool {
    !(0..6).any(|other| other != action && bindings.binding(other) == Some(key_name))
}

pub(crate) fn read_navigation_actions(
    input: &ButtonInput<KeyCode>,
    gamepads: &Query<&Gamepad>,
    bindings: &MenuNavigationBindings,
    state: &mut NavigationState,
    allow_text_keys: bool,
) -> NavigationActions {
    
    
    
    let mut actions = NavigationActions {
        up: key_binding_pressed(input, bindings, 0, allow_text_keys)
            || (built_in_key_allowed(bindings, "ArrowUp", 0) && input.just_pressed(KeyCode::ArrowUp))
            || (allow_text_keys && built_in_key_allowed(bindings, "KeyW", 0) && input.just_pressed(KeyCode::KeyW)),
        down: key_binding_pressed(input, bindings, 1, allow_text_keys)
            || (built_in_key_allowed(bindings, "ArrowDown", 1) && input.just_pressed(KeyCode::ArrowDown))
            || (allow_text_keys && built_in_key_allowed(bindings, "KeyS", 1) && input.just_pressed(KeyCode::KeyS)),
        left: key_binding_pressed(input, bindings, 2, allow_text_keys)
            || (built_in_key_allowed(bindings, "ArrowLeft", 2) && input.just_pressed(KeyCode::ArrowLeft))
            || (allow_text_keys && built_in_key_allowed(bindings, "KeyA", 2) && input.just_pressed(KeyCode::KeyA)),
        right: key_binding_pressed(input, bindings, 3, allow_text_keys)
            || (built_in_key_allowed(bindings, "ArrowRight", 3) && input.just_pressed(KeyCode::ArrowRight))
            || (allow_text_keys && built_in_key_allowed(bindings, "KeyD", 3) && input.just_pressed(KeyCode::KeyD)),
        confirm: key_binding_pressed(input, bindings, 4, allow_text_keys)
            || (built_in_key_allowed(bindings, "Enter", 4) && input.just_pressed(KeyCode::Enter))
            || (built_in_key_allowed(bindings, "Space", 4) && input.just_pressed(KeyCode::Space)),
        cancel: key_binding_pressed(input, bindings, 5, allow_text_keys)
            || (built_in_key_allowed(bindings, "Escape", 5) && input.just_pressed(KeyCode::Escape))
            || (allow_text_keys && built_in_key_allowed(bindings, "Backspace", 5) && input.just_pressed(KeyCode::Backspace)),
        device: None,
    };
    let keyboard_activity = input.get_just_pressed().next().is_some();
    if keyboard_activity {
        actions.device = Some(ActiveInputDevice::Keyboard);
    }

    let mut any_gamepad = false;
    let mut stick_up = false;
    let mut stick_down = false;
    let mut stick_left = false;
    let mut stick_right = false;
    let mut pad_up = false;
    let mut pad_down = false;
    let mut pad_left = false;
    let mut pad_right = false;
    let mut pad_confirm = false;
    let mut pad_cancel = false;
    for gamepad in gamepads.iter() {
        any_gamepad = true;
        let stick = gamepad.left_stick();
        stick_up |= stick.y > 0.65;
        stick_down |= stick.y < -0.65;
        stick_left |= stick.x < -0.65;
        stick_right |= stick.x > 0.65;
        pad_up |= gamepad.just_pressed(GamepadButton::DPadUp);
        pad_down |= gamepad.just_pressed(GamepadButton::DPadDown);
        pad_left |= gamepad.just_pressed(GamepadButton::DPadLeft);
        pad_right |= gamepad.just_pressed(GamepadButton::DPadRight);
        pad_confirm |= gamepad.just_pressed(GamepadButton::South) || gamepad.just_pressed(GamepadButton::Start);
        pad_cancel |= gamepad.just_pressed(GamepadButton::East) || gamepad.just_pressed(GamepadButton::Select);
    }
    
    
    pad_up |= stick_up && !state.stick_up;
    pad_down |= stick_down && !state.stick_down;
    pad_left |= stick_left && !state.stick_left;
    pad_right |= stick_right && !state.stick_right;
    state.stick_up = stick_up;
    state.stick_down = stick_down;
    state.stick_left = stick_left;
    state.stick_right = stick_right;
    if !any_gamepad {
        state.stick_up = false;
        state.stick_down = false;
        state.stick_left = false;
        state.stick_right = false;
    }
    if pad_up || pad_down || pad_left || pad_right || pad_confirm || pad_cancel {
        actions.up |= pad_up;
        actions.down |= pad_down;
        actions.left |= pad_left;
        actions.right |= pad_right;
        actions.confirm |= pad_confirm;
        actions.cancel |= pad_cancel;
        actions.device = Some(ActiveInputDevice::Gamepad);
    }
    actions
}


pub fn detect_gameplay_input_device(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut navigation: ResMut<NavigationState>,
) {
    if keyboard.get_just_pressed().next().is_some() {
        navigation.active_device = ActiveInputDevice::Keyboard;
    }
    if mouse_buttons.get_just_pressed().next().is_some() {
        navigation.active_device = ActiveInputDevice::Mouse;
    }
    for gamepad in gamepads.iter() {
        let stick = gamepad.left_stick();
        let active = stick.length_squared() > 0.08
            || gamepad.get_just_pressed().next().is_some();
        if active {
            navigation.active_device = ActiveInputDevice::Gamepad;
            break;
        }
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
}

pub fn restore_navigation_focus(
    screen: Res<State<Screen>>,
    capture: Res<ControlCapture>,
    mut selection: ResMut<MenuSelection>,
    mut navigation: ResMut<NavigationState>,
) {
    let current = *screen.get();
    if navigation.last_screen != Some(current) {
        if let Some(previous) = navigation.last_screen {
            let last_selected = navigation.last_selected;
            navigation.focus_by_screen.insert(previous, last_selected);
        }
        if let Some(focus) = navigation.focus_by_screen.get(&current).copied() {
            selection.selected = focus;
        }
        navigation.last_screen = Some(current);
    }
    navigation.context = if capture.action_index.is_some() || capture.menu_action_index.is_some() || capture.gamepad_action_index.is_some() {
        InputContext::Rebinding
    } else {
        match current {
            Screen::Playing => InputContext::Gameplay,
            Screen::Paused | Screen::QuickSettings | Screen::RestartConfirm => InputContext::PauseMenu,
            Screen::NetworkLobby => InputContext::TextEntry,
            _ => InputContext::Menu,
        }
    };
}

pub fn update_input_context(
    screen: Res<State<Screen>>,
    capture: Res<ControlCapture>,
    mut navigation: ResMut<NavigationState>,
) {
    navigation.context = if capture.action_index.is_some() || capture.menu_action_index.is_some() || capture.gamepad_action_index.is_some() {
        InputContext::Rebinding
    } else {
        match screen.get() {
            Screen::Playing => InputContext::Gameplay,
            Screen::Paused | Screen::QuickSettings | Screen::RestartConfirm => InputContext::PauseMenu,
            Screen::NetworkLobby => InputContext::TextEntry,
            _ => InputContext::Menu,
        }
    };
}

pub fn capture_navigation_focus(
    screen: Res<State<Screen>>,
    selection: Res<MenuSelection>,
    mut navigation: ResMut<NavigationState>,
) {
    let current = *screen.get();
    navigation.last_screen = Some(current);
    navigation.last_selected = selection.selected;
    navigation.focus_by_screen.insert(current, selection.selected);
}

#[derive(SystemParam)]
pub(crate) struct MenuInputParams<'w, 's> {
    input: Res<'w, ButtonInput<KeyCode>>,
    time: Res<'w, Time>,
    screen: Res<'w, State<Screen>>,
    next_state: ResMut<'w, NextState<Screen>>,
    selection: ResMut<'w, MenuSelection>,
    settings: ResMut<'w, Settings>,
    persistent: ResMut<'w, PersistentData>,
    session: ResMut<'w, MatchSession>,
    inventory: ResMut<'w, PowerUpInventory>,
    pending_pickup: ResMut<'w, PendingPowerUp>,
    open_play_duration: ResMut<'w, crate::game::components::OpenPlayDuration>,
    arena: ResMut<'w, crate::game::components::ArenaState>,
    inventory_discard: ResMut<'w, crate::game::components::InventoryDiscardState>,
    powerup_editor: ResMut<'w, PowerUpEditorSelection>,
    control_capture: ResMut<'w, ControlCapture>,
    navigation: ResMut<'w, NavigationState>,
    developer_document_state: ResMut<'w, DeveloperDocumentState>,
    skin_selection: ResMut<'w, SkinSelection>,
    high_score_filters: ResMut<'w, HighScoreFilters>,
    language_dropdown: ResMut<'w, LanguageDropdownState>,
    asset_server: Res<'w, AssetServer>,
    ui_font: Res<'w, UiFont>,
    app_exit: MessageWriter<'w, AppExit>,
    windows: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
    
    
    menu_text: Query<'w, 's, (&'static MenuItem, &'static mut Text2d, &'static Transform), With<MenuItem>>,
    menu_targets: Query<'w, 's, (&'static MenuItem, &'static Transform), (With<crate::game::components::SocialIcon>, Without<Text2d>)>,
    main_menu_labels: Query<'w, 's, (&'static MainMenuLabel, &'static mut Text2d), Without<MenuItem>>,
    github_bar_sprites: Query<'w, 's, (&'static mut Sprite, &'static MainMenuGithubBar), (Without<Paddle>, Without<crate::game::components::Ball>)>,
    flag_pixels: Query<'w, 's, Entity, With<crate::game::components::LanguageFlagPixel>>,
    language_dropdown_items: Query<'w, 's, (&'static LanguageDropdownItem, &'static mut Visibility), Without<LanguageDropdownPanel>>,
    language_dropdown_panel: Query<'w, 's, &'static mut Visibility, With<LanguageDropdownPanel>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    gamepads: Query<'w, 's, &'static Gamepad>,
    gamepad_controllers: Query<'w, 's, (Entity, &'static Gamepad)>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera2d>>,
    player_paddles: Query<'w, 's, (&'static mut Sprite, &'static Paddle), (With<Paddle>, Without<crate::game::components::Ball>)>,
    ball_sprites: Query<'w, 's, &'static mut Sprite, (With<crate::game::components::Ball>, Without<Paddle>)>,
}

pub fn update_highlight(selection: Res<MenuSelection>, mut items: Query<(&MenuItem, &mut TextColor)>) {
    for (item, mut color) in &mut items {
        color.0 = if item.0 == selection.selected { Color::srgb(1.0, 0.77, 0.34) }
            else { Color::srgb(0.72, 0.82, 0.94) };
    }
}

pub fn menu_input(mut commands: Commands, params: MenuInputParams) {
    let MenuInputParams {
        input,
        time,
        screen,
        mut next_state,
        mut selection,
        mut settings,
        mut persistent,
        mut session,
        mut inventory,
        mut pending_pickup,
        mut open_play_duration,
        mut arena,
        mut inventory_discard,
        mut powerup_editor,
        mut control_capture,
        mut navigation,
        mut developer_document_state,
        mut skin_selection,
        mut high_score_filters,
        mut language_dropdown,
        asset_server,
        ui_font,
        mut app_exit,
        mut windows,
        mut menu_text,
        menu_targets,
        mut main_menu_labels,
        mut github_bar_sprites,
        mut flag_pixels,
        mut language_dropdown_items,
        mut language_dropdown_panel,
        mouse_buttons,
        gamepads,
        gamepad_controllers,
        cameras,
        mut player_paddles,
        mut ball_sprites,
    } = params;

    #[cfg(target_arch = "wasm32")]
    let _ = ui_font;

    let current = *screen.get();
    if current == Screen::Inventory {
        if input.pressed(KeyCode::Delete) {
            inventory_discard.held_seconds += time.delta_secs();
            if inventory_discard.held_seconds >= 0.5 {
                if let Some((side, slot)) = inventory_discard.selected.take() {
                    if side != Side::PlayerTwo || session.mode == crate::game::components::MatchMode::LocalVersus {
                        let slots = inventory.for_side_mut(side);
                        if slot < slots.len() { slots.remove(slot); }
                    }
                }
                inventory_discard.held_seconds = 0.0;
            }
        } else {
            inventory_discard.held_seconds = 0.0;
        }
    } else {
        inventory_discard.selected = None;
        inventory_discard.held_seconds = 0.0;
    }
    let navigation_actions = if current == Screen::NetworkLobby {
        NavigationActions::default()
    } else {
        read_navigation_actions(&input, &gamepads, &settings.menu_navigation, &mut navigation, true)
    };
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
    navigation.context = if control_capture.action_index.is_some() || control_capture.menu_action_index.is_some() || control_capture.gamepad_action_index.is_some() {
        InputContext::Rebinding
    } else {
        match current {
            Screen::Playing => InputContext::Gameplay,
            Screen::Paused | Screen::QuickSettings | Screen::RestartConfirm => InputContext::PauseMenu,
            Screen::NetworkLobby => InputContext::TextEntry,
            _ => InputContext::Menu,
        }
    };

    
    
    
    if current == Screen::MainMenu && language_dropdown.open {
        if navigation_actions.cancel {
            language_dropdown.open = false;
            selection.selected = 14;
            for (_, mut visibility) in &mut language_dropdown_items { *visibility = Visibility::Hidden; }
            for mut visibility in &mut language_dropdown_panel { *visibility = Visibility::Hidden; }
            return;
        }
        if navigation_actions.up {
            language_dropdown.selected = if language_dropdown.selected == 0 { crate::persistence::Language::ALL.len() - 1 } else { language_dropdown.selected - 1 };
        }
        if navigation_actions.down {
            language_dropdown.selected = (language_dropdown.selected + 1) % crate::persistence::Language::ALL.len();
        }
        let mut chosen = None;
        if mouse_buttons.just_pressed(MouseButton::Left) {
            if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
                if let Some(cursor) = window.cursor_position() {
                    if let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) {
                        let scale = settings.ui_scale.clamp(0.80, 1.10);
                        for (item, _) in &language_dropdown_items {
                            let y = (96.0 - item.0 as f32 * 17.0) * scale;
                            if (world.x + 326.0 * scale).abs() <= 77.0 * scale
                                && (world.y - y).abs() <= 9.0 * scale
                            {
                                chosen = Some(item.0);
                                break;
                            }
                        }
                    }
                }
            }
        }
        if navigation_actions.confirm {
            chosen = Some(language_dropdown.selected);
        }
        if mouse_buttons.just_pressed(MouseButton::Left) && chosen.is_none() {
            language_dropdown.open = false;
            selection.selected = 14;
            for (_, mut visibility) in &mut language_dropdown_items { *visibility = Visibility::Hidden; }
            for mut visibility in &mut language_dropdown_panel { *visibility = Visibility::Hidden; }
            return;
        }
        if let Some(index) = chosen {
            settings.language = crate::persistence::Language::from_index(index);
            refresh_language_flag(&mut commands, &mut flag_pixels, settings.language);
            persist_settings(&settings, &mut persistent);
            language_dropdown.open = false;
            selection.selected = 14;
            for (_, mut visibility) in &mut language_dropdown_items { *visibility = Visibility::Hidden; }
            for mut visibility in &mut language_dropdown_panel { *visibility = Visibility::Hidden; }
            return;
        }
        selection.selected = 20 + language_dropdown.selected;
        return;
    }

    
    
    if current == Screen::NetworkLobby {
        if navigation_actions.cancel {
            next_state.set(Screen::MainMenu);
            audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Menu);
        }
        return;
    }

    
    
    if current == Screen::GamepadControls {
        if let Some(action) = control_capture.gamepad_action_index {
            let cancel_capture = input.just_pressed(KeyCode::Escape)
                || gamepads.iter().any(|gamepad| gamepad.just_pressed(GamepadButton::Select));
            if cancel_capture {
                control_capture.gamepad_action_index = None;
                show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "GAMEPAD BINDING CANCELLED"), false);
                return;
            }
            let mut controllers: Vec<_> = gamepad_controllers.iter().collect();
            controllers.sort_by_key(|(entity, _)| entity.to_bits());
            let pressed = if action == 16 {
                
                controllers.iter().find_map(|(_, gamepad)| {
                    gamepad.get_just_pressed().find(|button| **button != GamepadButton::Select).copied()
                })
            } else {
                
                
                let controller_index = if action <= 6 || action == 14 { 0 } else { 1 };
                controllers.get(controller_index).and_then(|(_, gamepad)| {
                    gamepad.get_just_pressed().find(|button| **button != GamepadButton::Select).copied()
                })
            };
            if let Some(button) = pressed {
                let name = gamepad_button_name(button);
                if name != "Unsupported" {
                    if settings.gamepad_controls.conflict(action, name).is_none() {
                        settings.gamepad_controls.set_binding(action, name.to_owned());
                        persist_settings(&settings, &mut persistent);
                        control_capture.gamepad_action_index = None;
                        show_notice(&mut commands, ui_font.source.clone(), format!("{}: {name}", tr(settings.language, "GAMEPAD BINDING SAVED")), true);
                    } else {
                        show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "GAMEPAD BUTTON ALREADY USED"), false);
                    }
                }
            }
            return;
        }
    }

    
    
    if current == Screen::Controls {
        if let Some(action) = control_capture.menu_action_index {
            if navigation_actions.cancel {
                control_capture.menu_action_index = None;
                show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "KEY BINDING CANCELLED"), false);
                return;
            }
            if let Some(code) = input.get_just_pressed().next().copied() {
                let key = key_name(code);
                if key != "Unsupported" {
                    let duplicate = (0..6).any(|other| other != action && settings.menu_navigation.binding(other) == Some(key));
                    if !duplicate {
                        settings.menu_navigation.set_binding(action, key.to_owned());
                        persist_settings(&settings, &mut persistent);
                        control_capture.menu_action_index = None;
                        show_notice(&mut commands, ui_font.source.clone(), format!("{}: {key}", tr(settings.language, "KEY BINDING SAVED")), true);
                    } else {
                        show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "KEY ALREADY USED BY"), false);
                    }
                }
            }
            return;
        }
        if let Some(action) = control_capture.action_index {
            if navigation_actions.cancel {
                control_capture.action_index = None;
                show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "KEY BINDING CANCELLED"), false);
                return;
            }
            if let Some(code) = input.get_just_pressed().next().copied() {
                let key = key_name(code);
                if key != "Unsupported" {
                    let mut updated_bindings = settings.controls.clone();
                    let previous_key = updated_bindings.binding(action).map(str::to_owned);
                    let conflict = updated_bindings.conflict(action, key);
                    let can_swap = if let Some(conflict_action) = conflict {
                        previous_key.as_ref().is_some_and(|old_key| {
                            updated_bindings.set_binding(conflict_action, old_key.clone())
                        })
                    } else {
                        true
                    };
                    if can_swap && updated_bindings.set_binding(action, key.to_owned()) {
                        
                        
                        
                        let has_duplicate = (0..17).any(|left| {
                            ((left + 1)..17).any(|right| {
                                updated_bindings.binding(left).is_some()
                                    && updated_bindings.binding(left) == updated_bindings.binding(right)
                            })
                        });
                        if !has_duplicate {
                            settings.controls = updated_bindings;
                            persist_settings(&settings, &mut persistent);
                            control_capture.action_index = None;
                            let message = if conflict.is_some() {
                                format!("{}: {} ↔ {}", tr(settings.language, "KEY BINDINGS SWAPPED"), key, previous_key.unwrap_or_default())
                            } else {
                                format!("{}: {key}", tr(settings.language, "KEY BINDING SAVED"))
                            };
                            show_notice(&mut commands, ui_font.source.clone(), message, true);
                        } else {
                            show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "KEY ALREADY USED BY"), false);
                        }
                    }
                }
            }
            return;
        }
    }

    if navigation_actions.cancel {
        match current {
            Screen::Paused if input.just_pressed(KeyCode::Escape) => {
                next_state.set(Screen::Playing);
                audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Menu);
                return;
            }
            Screen::Paused => { session.active = false; next_state.set(Screen::MainMenu); },
            Screen::QuickSettings | Screen::RestartConfirm => next_state.set(Screen::Paused),
            Screen::DeveloperDocument => {
                developer_document_state.closing = true;
                developer_document_state.close_elapsed = 0.0;
            },
            Screen::NetworkGuide => next_state.set(Screen::NetworkLobby),
            Screen::GamepadControls => next_state.set(Screen::Controls),
            Screen::Settings => {
                let return_screen = selection.pause_return_to.take()
                    .or_else(|| selection.return_to.take())
                    .unwrap_or(Screen::MainMenu);
                next_state.set(return_screen);
            },
            Screen::Skins => next_state.set(Screen::Settings),
            Screen::PowerUpSettings => next_state.set(selection.return_to.take().unwrap_or(Screen::Settings)),
            Screen::PowerUpEditor => next_state.set(Screen::PowerUpSettings),
            Screen::Inventory => next_state.set(Screen::Paused),
            Screen::Results => next_state.set(Screen::MainMenu),
            Screen::MainMenu => return,
            _ => next_state.set(Screen::MainMenu),
        }
        audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Menu);
        return;
    }

    let mut mouse_activate = false;
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), cameras.single()) {
            if let Some(cursor) = window.cursor_position() {
                if let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) {
                    
                    
                    if current == Screen::Settings {
                        let scale = settings.ui_scale.clamp(0.80, 1.10);
                        let slider_y = (126.0 - 9.0 * 22.0) * scale;
                        let slider_min_x = -172.0 * scale;
                        let slider_max_x = -40.0 * scale;
                        if (slider_min_x..=slider_max_x).contains(&world.x)
                            && (world.y - slider_y).abs() <= 10.0 * scale
                        {
                            selection.selected = 9;
                            let fraction = ((world.x / scale + 172.0) / 132.0).clamp(0.0, 1.0);
                            let steps = (fraction * 30.0).round();
                            settings.ui_scale = (0.80 + steps * 0.01).clamp(0.80, 1.10);
                            persist_settings(&settings, &mut persistent);
                        }
                    }
                    let ui_scale = settings.ui_scale.clamp(0.80, 1.10);
                    if current == Screen::MainMenu
                        && (-405.0 * ui_scale..=-247.0 * ui_scale).contains(&world.x)
                        && (110.0 * ui_scale..=142.0 * ui_scale).contains(&world.y)
                    {
                        selection.selected = 14;
                        mouse_activate = true;
                    }
                    for (item, _, transform) in &mut menu_text {
                        if mouse_activate { break; }
                        let controls_restore_row = 13 + settings.inventory_slots.clamp(1, 5) as usize * 2;
                        let width = if current == Screen::Credits && item.0 == 2 { 170.0 }
                            else if current == Screen::Credits { 54.0 }
                            else if current == Screen::Controls && item.0 >= controls_restore_row { 118.0 }
                            else if current == Screen::GamepadControls { 112.0 }
                            else if current == Screen::PowerUpEditor && item.0 >= 12 { 42.0 }
                            else { 170.0 };
                        let height = if current == Screen::Credits { 25.0 }
                            else if current == Screen::GamepadControls { 15.0 }
                            else if current == Screen::PowerUpEditor && item.0 >= 12 { 11.0 }
                            else { 13.0 };
                        if (world.x - transform.translation.x).abs() <= width * ui_scale
                            && (world.y - transform.translation.y).abs() <= height * ui_scale
                        {
                            selection.selected = item.0;
                            mouse_activate = true;
                            break;
                        }
                    }
                    if !mouse_activate && current == Screen::Credits {
                        for (item, transform) in &menu_targets {
                            if (world.x - transform.translation.x).abs() <= 34.0 * ui_scale
                                && (world.y - transform.translation.y).abs() <= 20.0 * ui_scale
                            {
                                selection.selected = item.0;
                                mouse_activate = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let count = menu_count(current, settings.inventory_slots, session.mode);
    
    
    
    if selection.selected >= count {
        selection.selected = count.saturating_sub(1);
    }
    if current == Screen::PowerUpEditor {
        
        
        if navigation_actions.up {
            let mut candidate = selection.selected;
            for _ in 0..count {
                candidate = if candidate == 0 { count.saturating_sub(1) } else { candidate - 1 };
                if is_powerup_editor_row_selectable(powerup_editor.0, candidate) {
                    selection.selected = candidate;
                    break;
                }
            }
        }
        if navigation_actions.down {
            let mut candidate = selection.selected;
            for _ in 0..count {
                candidate = (candidate + 1) % count.max(1);
                if is_powerup_editor_row_selectable(powerup_editor.0, candidate) {
                    selection.selected = candidate;
                    break;
                }
            }
        }
    } else {
        if navigation_actions.up {
            selection.selected = if selection.selected == 0 { count.saturating_sub(1) } else { selection.selected - 1 };
        }
        if navigation_actions.down {
            selection.selected = (selection.selected + 1) % count.max(1);
        }
    }
    if current == Screen::Controls {
        let capacity = settings.inventory_slots.clamp(1, 5) as usize;
        let player_two_start = capacity + 2;
        let release_start = capacity * 2 + 4;
        let pause_row = release_start + 2;
        let menu_start = pause_row + 1;
        let restore_row = menu_start + 6;
        if navigation_actions.right {
            if selection.selected < player_two_start {
                selection.selected += player_two_start;
            } else if (player_two_start..player_two_start * 2).contains(&selection.selected) {
                selection.selected -= player_two_start;
            } else if selection.selected == release_start {
                selection.selected += 1;
            } else if selection.selected == release_start + 1 {
                selection.selected -= 1;
            } else if (menu_start..menu_start + 6).contains(&selection.selected) {
                let row_start = menu_start + ((selection.selected - menu_start) / 3) * 3;
                selection.selected = row_start + ((selection.selected - row_start + 1) % 3);
            } else if selection.selected == restore_row {
                selection.selected += 1;
            } else if selection.selected == restore_row + 1 {
                selection.selected = restore_row;
            }
        } else if navigation_actions.left {
            if selection.selected < player_two_start {
                selection.selected += player_two_start;
            } else if (player_two_start..player_two_start * 2).contains(&selection.selected) {
                selection.selected -= player_two_start;
            } else if selection.selected == release_start {
                selection.selected += 1;
            } else if selection.selected == release_start + 1 {
                selection.selected -= 1;
            } else if (menu_start..menu_start + 6).contains(&selection.selected) {
                let row_start = menu_start + ((selection.selected - menu_start) / 3) * 3;
                selection.selected = row_start + ((selection.selected - row_start + 2) % 3);
            } else if selection.selected == restore_row {
                selection.selected += 1;
            } else if selection.selected == restore_row + 1 {
                selection.selected = restore_row;
            }
        }
    }

    let direction = if navigation_actions.left { -1 }
        else if navigation_actions.right { 1 }
        else { 0 };
    let can_adjust = direction != 0 && match current {
        Screen::Difficulty => true,
        Screen::ScoreLimit => selection.selected == 4,
        Screen::OpenPlaySetup => selection.selected <= 1,
        Screen::ArenaSelect => selection.selected == 0,
        Screen::Settings => selection.selected <= 9,
        Screen::QuickSettings => selection.selected <= 4,
        Screen::Skins => selection.selected <= 2,
        Screen::HighScores => selection.selected <= 2,
        Screen::PowerUpSettings => selection.selected < 7,
        Screen::PowerUpEditor => selection.selected == 0 || selection.selected == 1
            || is_powerup_editor_row_selectable(powerup_editor.0, selection.selected)
                && (2..=9).contains(&selection.selected),
        Screen::MainMenu => selection.selected == 14,
        _ => false,
    };
    if can_adjust {
        let previous_language = settings.language;
        if current == Screen::Skins {
            adjust_skin_selection(&mut settings, &mut skin_selection, selection.selected, direction);
        } else if current == Screen::QuickSettings {
            match selection.selected {
                0 => settings.music_enabled = !settings.music_enabled,
                1 => settings.music_volume = (settings.music_volume + direction as f32 * 0.05).clamp(0.0, 1.0),
                2 => settings.sfx_enabled = !settings.sfx_enabled,
                3 => settings.sfx_volume = (settings.sfx_volume + direction as f32 * 0.05).clamp(0.0, 1.0),
                4 => settings.reduce_motion = !settings.reduce_motion,
                _ => {},
            }
        } else if current == Screen::HighScores {
            match selection.selected {
                0 => high_score_filters.match_type = (high_score_filters.match_type as i32 + direction).rem_euclid(4) as u8,
                1 => high_score_filters.difficulty = (high_score_filters.difficulty as i32 + direction).rem_euclid(5) as u8,
                2 => high_score_filters.period = (high_score_filters.period as i32 + direction).rem_euclid(3) as u8,
                _ => {},
            }
            high_score_filters.page = 0;
            next_state.set(Screen::HighScoresRefresh);
        } else {
            adjust_selected(current, selection.selected, direction, &mut settings, &persistent, &mut powerup_editor, &mut open_play_duration, &mut arena);
        }
        let language_changed = settings.language != previous_language;
        if current == Screen::Difficulty { selection.selected = settings.difficulty.index(); }
        if language_changed { refresh_language_flag(&mut commands, &mut flag_pixels, settings.language); }
        apply_live_display_settings(current, selection.selected, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
        persist_settings(&settings, &mut persistent);
    }

    if navigation_actions.confirm || mouse_activate {
        audio::play_sfx(&mut commands, &asset_server, &settings, Sfx::Menu);
        match current {
            Screen::MainMenu => match selection.selected {
                0 => {
                    session.mode = crate::game::components::MatchMode::SinglePlayer;
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                }
                1 => {
                    session.mode = crate::game::components::MatchMode::LocalVersus;
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                }
                2 => next_state.set(Screen::Difficulty),
                3 => next_state.set(Screen::ScoreLimit),
                4 => next_state.set(Screen::OpenPlaySetup),
                5 => next_state.set(Screen::ArenaSelect),
                6 => next_state.set(Screen::Controls),
                7 => { selection.return_to = Some(Screen::MainMenu); next_state.set(Screen::Settings); }
                8 => { selection.return_to = Some(Screen::MainMenu); next_state.set(Screen::PowerUpSettings); }
                9 => next_state.set(Screen::HighScores),
                10 => next_state.set(Screen::Progress),
                11 => next_state.set(Screen::Credits),
                12 => {
                    #[cfg(not(target_arch = "wasm32"))]
                    next_state.set(Screen::NetworkLobby);
                    #[cfg(target_arch = "wasm32")]
                    show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "NETWORK PLAY IS AVAILABLE IN THE DESKTOP BUILD"), false);
                }
                13 => { app_exit.write(AppExit::Success); },
                14 => {
                    language_dropdown.open = true;
                    language_dropdown.selected = settings.language.index();
                    selection.selected = 20 + language_dropdown.selected;
                    for (item, mut visibility) in &mut language_dropdown_items {
                        *visibility = if item.0 < crate::persistence::Language::ALL.len() { Visibility::Visible } else { Visibility::Hidden };
                    }
                    for mut visibility in &mut language_dropdown_panel { *visibility = Visibility::Visible; }
                }
                15 => {
                    #[cfg(target_arch = "wasm32")]
                    platform_links::open_and_copy(
                        GITHUB_REPOSITORY_URL,
                        &tr(settings.language, "COPY LINK SUCCESS"),
                        &tr(settings.language, "COPY LINK FAILED"),
                        &tr(settings.language, "COPY LINK COPIED ONLY"),
                        &tr(settings.language, "GITHUB ACTION FAILED"),
                    );
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let opened = platform_links::open_url(GITHUB_REPOSITORY_URL);
                        let copied = platform_links::copy_to_clipboard(GITHUB_REPOSITORY_URL);
                        let message = match (opened, copied) {
                            (true, true) => "COPY LINK SUCCESS",
                            (true, false) => "COPY LINK FAILED",
                            (false, true) => "COPY LINK COPIED ONLY",
                            (false, false) => "GITHUB ACTION FAILED",
                        };
                        show_notice(
                            &mut commands,
                            ui_font.source.clone(),
                            tr(settings.language, message),
                            opened && copied,
                        );
                    }
                }
                _ => {},
            },
            Screen::OpenPlaySetup => match selection.selected {
                2 => {
                    if open_play_duration.total_seconds() < 1.0 { open_play_duration.seconds = 1; }
                    session.mode = crate::game::components::MatchMode::OpenPlay;
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                }
                _ => next_state.set(Screen::MainMenu),
            },
            Screen::ArenaSelect => next_state.set(Screen::MainMenu),
            Screen::Difficulty => {
                if selection.selected < 4 {
                    settings.difficulty = Difficulty::from_index(selection.selected);
                    persist_settings(&settings, &mut persistent);
                }
                next_state.set(Screen::MainMenu);
            }
            Screen::ScoreLimit => match selection.selected {
                0 => { settings.winning_score = 5; persist_settings(&settings, &mut persistent); next_state.set(Screen::MainMenu); }
                1 => { settings.winning_score = 7; persist_settings(&settings, &mut persistent); next_state.set(Screen::MainMenu); }
                2 => { settings.winning_score = 10; persist_settings(&settings, &mut persistent); next_state.set(Screen::MainMenu); }
                3 => { settings.winning_score = 15; persist_settings(&settings, &mut persistent); next_state.set(Screen::MainMenu); }
                4 => { persist_settings(&settings, &mut persistent); next_state.set(Screen::MainMenu); }
                _ => next_state.set(Screen::MainMenu),
            },
            Screen::Settings => {
                if selection.selected == 10 || selection.selected == 11 {
                    skin_selection.tab = if selection.selected == 10 { 0 } else { 1 };
                    next_state.set(Screen::Skins);
                } else if selection.selected == 12 {
                    selection.return_to = Some(Screen::Settings);
                    next_state.set(Screen::PowerUpSettings);
                } else if selection.selected == 13 {
                    let return_screen = selection.pause_return_to.take()
                        .or_else(|| selection.return_to.take())
                        .unwrap_or(Screen::MainMenu);
                    next_state.set(return_screen);
                } else {
                    adjust_selected(current, selection.selected, 1, &mut settings, &persistent, &mut powerup_editor, &mut open_play_duration, &mut arena);
                    apply_live_display_settings(current, selection.selected, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
                    persist_settings(&settings, &mut persistent);
                }
            }
            Screen::Skins => match selection.selected {
                3 => {
                    #[cfg(not(target_arch = "wasm32"))]
                    match import_custom_skin(skin_selection.tab == 0) {
                        Ok(relative_path) => {
                            if skin_selection.tab == 0 { settings.paddle_custom_image = Some(relative_path); }
                            else { settings.ball_custom_image = Some(relative_path); }
                            apply_live_display_settings(current, 1, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
                            persist_settings(&settings, &mut persistent);
                            show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "CUSTOM IMAGE IMPORTED"), true);
                        }
                        Err(error) => show_notice(&mut commands, ui_font.source.clone(), skin_import_error_message(error, settings.language), false),
                    }
                    #[cfg(target_arch = "wasm32")]
                    show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "CUSTOM IMAGE PICKER DESKTOP ONLY"), false);
                }
                4 => {
                    if skin_selection.tab == 0 { settings.paddle_custom_image = None; }
                    else { settings.ball_custom_image = None; }
                    apply_live_display_settings(current, 1, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
                    persist_settings(&settings, &mut persistent);
                }
                5 => {
                    if skin_selection.tab == 0 { settings.paddle_shade = 0; }
                    else { settings.ball_shade = 0; }
                    apply_live_display_settings(current, 2, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
                    persist_settings(&settings, &mut persistent);
                }
                6 => next_state.set(Screen::Settings),
                _ => {
                    adjust_skin_selection(&mut settings, &mut skin_selection, selection.selected, 1);
                    apply_live_display_settings(current, selection.selected, &settings, &mut windows, &mut player_paddles, &mut ball_sprites, &asset_server);
                    persist_settings(&settings, &mut persistent);
                }
            },
            Screen::PowerUpSettings => {
                if selection.selected == 7 {
                    next_state.set(Screen::PowerUpEditor);
                } else if selection.selected == 8 {
                    next_state.set(selection.return_to.take().unwrap_or(Screen::Settings));
                } else {
                    adjust_selected(current, selection.selected, 1, &mut settings, &persistent, &mut powerup_editor, &mut open_play_duration, &mut arena);
                    persist_settings(&settings, &mut persistent);
                }
            }
            Screen::PowerUpEditor => {
                if (12..27).contains(&selection.selected) {
                    powerup_editor.0 = selection.selected - 12;
                    selection.selected = 0;
                } else if selection.selected == 10 {
                    reset_powerup_item_settings(&mut settings, powerup_editor.0);
                    persist_settings(&settings, &mut persistent);
                } else if selection.selected == 11 {
                    next_state.set(Screen::PowerUpSettings);
                } else {
                    adjust_selected(current, selection.selected, 1, &mut settings, &persistent, &mut powerup_editor, &mut open_play_duration, &mut arena);
                    persist_settings(&settings, &mut persistent);
                }
            }
            Screen::Inventory => {
                if selection.selected == 10 {
                    pending_pickup.kind = None;
                    pending_pickup.owner = Side::Player;
                    inventory_discard.selected = None;
                    inventory_discard.held_seconds = 0.0;
                } else if selection.selected >= 11 {
                    inventory_discard.selected = None;
                    inventory_discard.held_seconds = 0.0;
                    next_state.set(Screen::Paused);
                } else {
                    let (side, slot) = if selection.selected < 5 {
                        (Side::Player, selection.selected)
                    } else {
                        (Side::PlayerTwo, selection.selected - 5)
                    };
                    if side != Side::PlayerTwo || session.mode == crate::game::components::MatchMode::LocalVersus {
                        let slots = inventory.for_side_mut(side);
                        let capacity = settings.inventory_slots.clamp(1, 5) as usize;
                        if let Some(kind) = pending_pickup.kind {
                            if pending_pickup.owner == side {
                                if slot < slots.len() {
                                    slots[slot] = crate::game::components::PowerUpSlot::new(
                                        kind,
                                        crate::game::mechanics::powerup_max_charges(&settings, kind),
                                    );
                                    pending_pickup.kind = None;
                                    inventory_discard.selected = None;
                                } else if slot == slots.len() && slots.len() < capacity {
                                    slots.push(crate::game::components::PowerUpSlot::new(
                                        kind,
                                        crate::game::mechanics::powerup_max_charges(&settings, kind),
                                    ));
                                    pending_pickup.kind = None;
                                    inventory_discard.selected = None;
                                }
                            } else if slot < slots.len() {
                                inventory_discard.selected = Some((side, slot));
                                inventory_discard.held_seconds = 0.0;
                            }
                        } else if slot < slots.len() {
                            inventory_discard.selected = Some((side, slot));
                            inventory_discard.held_seconds = 0.0;
                        }
                    }
                }
            }
            Screen::Paused if session.mode == crate::game::components::MatchMode::Networked => match selection.selected {
                0 => next_state.set(Screen::Playing),
                1 => next_state.set(Screen::QuickSettings),
                2 => { selection.pause_return_to = Some(Screen::Paused); selection.return_to = Some(Screen::Paused); next_state.set(Screen::Settings); }
                _ => { session.active = false; next_state.set(Screen::MainMenu); },
            },
            Screen::Paused => match selection.selected {
                0 => next_state.set(Screen::Playing),
                1 => next_state.set(Screen::RestartConfirm),
                2 => next_state.set(Screen::QuickSettings),
                3 => next_state.set(Screen::Inventory),
                4 => { selection.pause_return_to = Some(Screen::Paused); selection.return_to = Some(Screen::Paused); next_state.set(Screen::Settings); }
                _ => { session.active = false; next_state.set(Screen::MainMenu); },
            },
            Screen::QuickSettings => {
                match selection.selected {
                    0 => settings.music_enabled = !settings.music_enabled,
                    2 => settings.sfx_enabled = !settings.sfx_enabled,
                    4 => settings.reduce_motion = !settings.reduce_motion,
                    5 => next_state.set(Screen::Paused),
                    _ => {},
                }
                persist_settings(&settings, &mut persistent);
            },
            Screen::RestartConfirm => {
                if selection.selected == 0 {
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                } else {
                    next_state.set(Screen::Paused);
                }
            },
            Screen::Results => {
                if selection.selected == 0 {
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                } else {
                    next_state.set(Screen::MainMenu);
                }
            }
            Screen::Credits => match selection.selected {
                0 | 1 => {
                    if let Some(url) = platform_links::social_profile_url(selection.selected) {
                        platform_links::open_url(url);
                    }
                }
                2 => next_state.set(Screen::DeveloperDocument),
                _ => next_state.set(Screen::MainMenu),
            },
            Screen::DeveloperDocument => {
                developer_document_state.closing = true;
                developer_document_state.close_elapsed = 0.0;
            },
            Screen::NetworkGuide => { if selection.selected >= 8 { next_state.set(Screen::NetworkLobby); } },
            Screen::Controls => {
                let capacity = settings.inventory_slots.clamp(1, 5) as usize;
                let gamepad_row = 13 + capacity * 2;
                let restore_row = gamepad_row + 1;
                let back_row = restore_row + 1;
                if selection.selected == gamepad_row {
                    next_state.set(Screen::GamepadControls);
                } else if selection.selected == restore_row {
                    settings.controls = ControlBindings::default();
                    settings.menu_navigation = MenuNavigationBindings::default();
                    settings.gamepad_controls = GamepadControlBindings::default();
                    control_capture.action_index = None;
                    control_capture.menu_action_index = None;
                    control_capture.gamepad_action_index = None;
                    persist_settings(&settings, &mut persistent);
                    show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "DEFAULT CONTROLS RESTORED"), true);
                } else if selection.selected == back_row {
                    next_state.set(Screen::MainMenu);
                } else if let Some(action) = control_action_for_row(selection.selected, capacity) {
                    control_capture.menu_action_index = None;
                    control_capture.action_index = Some(action);
                    show_notice(&mut commands, ui_font.source.clone(), format!("{}: {}", tr(settings.language, "PRESS A KEY FOR"), tr(settings.language, ControlBindings::action_label(action))), true);
                } else if let Some(action) = menu_navigation_action_for_row(selection.selected, capacity) {
                    control_capture.action_index = None;
                    control_capture.menu_action_index = Some(action);
                    show_notice(&mut commands, ui_font.source.clone(), format!("{}: {}", tr(settings.language, "PRESS A KEY FOR"), tr(settings.language, MenuNavigationBindings::action_label(action))), true);
                }
            }
            Screen::GamepadControls => {
                if selection.selected == 17 {
                    settings.gamepad_controls = GamepadControlBindings::default();
                    control_capture.gamepad_action_index = None;
                    persist_settings(&settings, &mut persistent);
                    show_notice(&mut commands, ui_font.source.clone(), tr(settings.language, "GAMEPAD DEFAULTS RESTORED"), true);
                } else if selection.selected >= 18 {
                    next_state.set(Screen::Controls);
                } else if selection.selected < 17 {
                    control_capture.action_index = None;
                    control_capture.menu_action_index = None;
                    control_capture.gamepad_action_index = Some(selection.selected);
                    show_notice(&mut commands, ui_font.source.clone(), format!("{}: {}", tr(settings.language, "PRESS A CONTROLLER BUTTON FOR"), tr(settings.language, GamepadControlBindings::action_label(selection.selected))), true);
                }
            }
            Screen::HighScores => {
                match selection.selected {
                    0 => { high_score_filters.match_type = (high_score_filters.match_type + 1) % 4; high_score_filters.page = 0; next_state.set(Screen::HighScoresRefresh); }
                    1 => { high_score_filters.difficulty = (high_score_filters.difficulty + 1) % 5; high_score_filters.page = 0; next_state.set(Screen::HighScoresRefresh); }
                    2 => { high_score_filters.period = (high_score_filters.period + 1) % 3; high_score_filters.page = 0; next_state.set(Screen::HighScoresRefresh); }
                    3 => { high_score_filters.page = high_score_filters.page.saturating_sub(1); next_state.set(Screen::HighScoresRefresh); }
                    4 => { high_score_filters.page = high_score_filters.page.saturating_add(1); next_state.set(Screen::HighScoresRefresh); }
                    _ => next_state.set(Screen::MainMenu),
                }
            }
            Screen::Progress => next_state.set(Screen::MainMenu),
            Screen::NetworkLobby | Screen::Playing | Screen::HighScoresRefresh => {}
        }
    }

    if current == Screen::MainMenu {
        let hint_key = match navigation.active_device {
            ActiveInputDevice::Gamepad => "NAVIGATION HINT GAMEPAD",
            ActiveInputDevice::Mouse => "NAVIGATION HINT MOUSE",
            ActiveInputDevice::Keyboard => "NAVIGATION HINT KEYBOARD",
        };
        let labels = [
            "RETRO PONG",
            "ARCADE EDITION  //  32-BIT CLASSICS",
            "SOLO AI  //  MULTIPLAYER NETWORK",
            hint_key,
        ];
        for (label, mut text) in &mut main_menu_labels {
            if let Some(key) = labels.get(label.0) { text.0 = tr(settings.language, key); }
        }
        let keys = [
            "PLAY VS AI", "LOCAL 2 PLAYER", "DIFFICULTY", "SCORE LIMIT", "OPEN PLAY MODE", "ARENA SELECT",
            "CONTROLS", "SETTINGS", "POWER-UP RULES", "HIGH SCORES", "PLAYER PROGRESS", "CREDITS / ABOUT", "NETWORK MULTIPLAYER", "EXIT",
        ];
        for (item, mut text, _) in &mut menu_text {
            if let Some(key) = keys.get(item.0) { text.0 = tr(settings.language, key); }
            else if item.0 == 14 { text.0 = settings.language.native_name().to_string(); }
            else if item.0 == 15 { text.0 = tr(settings.language, "STAR ON GITHUB"); }
            else if (20..26).contains(&item.0) {
                text.0 = crate::persistence::Language::from_index(item.0 - 20).native_name().to_string();
            }
            else if item.0 == 12 {
                #[cfg(target_arch = "wasm32")]
                { text.0 = tr(settings.language, "NETWORK MULTIPLAYER (DESKTOP ONLY)"); }
                #[cfg(not(target_arch = "wasm32"))]
                { text.0 = tr(settings.language, "NETWORK MULTIPLAYER"); }
            }
        }
        let github_width = ((tr(settings.language, "STAR ON GITHUB").chars().count() as f32 * 5.2) + 90.0).clamp(184.0, 360.0);
        for (mut sprite, bar) in &mut github_bar_sprites {
            sprite.custom_size = Some(Vec2::new(github_width, bar.height));
        }
    }

    if current == Screen::Skins {
        let labels = super::screens::skin_items(&settings, &skin_selection);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::QuickSettings {
        let labels = super::screens::quick_settings_items(&settings);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::Settings {
        let labels = settings_items(&settings);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::PowerUpSettings {
        let labels = powerup_settings_items(&settings);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::PowerUpEditor {
        let labels = powerup_editor_items(&settings, powerup_editor.0);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::Inventory {
        let labels = inventory_management_items(&inventory, &pending_pickup, &inventory_discard, &session, settings.inventory_slots, settings.language);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::ScoreLimit {
        for (item, mut text, _) in &mut menu_text {
            if item.0 == 4 { text.0 = format!("{}{:02}", tr(settings.language, "CUSTOM SCORE: "), settings.winning_score); }
        }
    }
    if current == Screen::OpenPlaySetup {
        for (item, mut text, _) in &mut menu_text {
            match item.0 {
                0 => text.0 = format!("MINUTES  //  {:02}", open_play_duration.minutes),
                1 => text.0 = format!("SECONDS  //  {:02}", open_play_duration.seconds),
                _ => {}
            }
        }
    }
    if current == Screen::Controls && control_capture.action_index.is_none() && control_capture.menu_action_index.is_none() {
        let labels = super::screens::controls_items(&settings);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
    if current == Screen::GamepadControls && control_capture.gamepad_action_index.is_none() {
        let labels = super::screens::gamepad_controls_items(&settings);
        for (item, mut text, _) in &mut menu_text {
            if let Some(label) = labels.get(item.0) { text.0 = label.clone(); }
        }
    }
}

