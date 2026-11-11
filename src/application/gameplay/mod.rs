mod ai;
#[path = "../../domain/game/components/mod.rs"]
pub(crate) mod components;
pub(crate) mod mechanics;
pub mod systems;

use bevy::prelude::*;
use components::{
    AiRandom, ClickEffectCooldown, MatchSession, MechanicsClock, MenuSelection,
    PendingPowerUp, PowerUpEditorSelection, PowerUpInventory, ControlCapture, SkinSelection,
    DeveloperDocumentState, NavigationState, ArenaState, OpenPlayDuration,
    InventoryDiscardState,
};

#[derive(States, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    MainMenu,
    NetworkLobby,
    NetworkGuide,
    HighScoresRefresh,
    DeveloperDocument,
    Difficulty,
    ScoreLimit,
    OpenPlaySetup,
    ArenaSelect,
    Settings,
    Skins,
    PowerUpSettings,
    PowerUpEditor,
    Inventory,
    Controls,
    GamepadControls,
    HighScores,
    Progress,
    Credits,
    Playing,
    Paused,
    QuickSettings,
    RestartConfirm,
    Results,
}

fn is_menu_screen(screen: Res<State<Screen>>) -> bool {
    matches!(
        screen.get(),
        Screen::MainMenu
            | Screen::NetworkLobby
            | Screen::NetworkGuide
            | Screen::DeveloperDocument
            | Screen::Difficulty
            | Screen::ScoreLimit
            | Screen::OpenPlaySetup
            | Screen::ArenaSelect
            | Screen::Settings
            | Screen::Skins
            | Screen::PowerUpSettings
            | Screen::PowerUpEditor
            | Screen::Inventory
            | Screen::Controls
            | Screen::GamepadControls
            | Screen::HighScores
            | Screen::Progress
            | Screen::Credits
            | Screen::Paused
            | Screen::QuickSettings
            | Screen::RestartConfirm
            | Screen::Results
    )
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        initialize_game_resources(app);
        register_startup_systems(app);
        register_screen_transitions(app);
        register_menu_systems(app);
        register_match_systems(app);
    }
}

fn initialize_game_resources(app: &mut App) {
    app.init_resource::<MenuSelection>()
        .init_resource::<MatchSession>()
        .init_resource::<AiRandom>()
        .init_resource::<MechanicsClock>()
        .init_resource::<PowerUpInventory>()
        .init_resource::<ArenaState>()
        .init_resource::<OpenPlayDuration>()
        .init_resource::<InventoryDiscardState>()
        .init_resource::<PendingPowerUp>()
        .init_resource::<PowerUpEditorSelection>()
        .init_resource::<ControlCapture>()
        .init_resource::<SkinSelection>()
        .init_resource::<components::HighScoreFilters>()
        .init_resource::<components::LanguageDropdownState>()
        .init_resource::<DeveloperDocumentState>()
        .init_resource::<NavigationState>()
        .init_resource::<ClickEffectCooldown>();
}

fn register_startup_systems(app: &mut App) {
    app.add_systems(Startup, systems::setup_world);
}

fn register_screen_transitions(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::MainMenu),
        (crate::ui::cleanup_pause_overlay, systems::cleanup_game, crate::ui::spawn_main_menu).chain(),
    )
    .add_systems(OnEnter(Screen::Difficulty), crate::ui::spawn_difficulty)
    .add_systems(OnEnter(Screen::ScoreLimit), crate::ui::spawn_score_limit)
    .add_systems(OnEnter(Screen::OpenPlaySetup), crate::ui::spawn_open_play_setup)
    .add_systems(OnEnter(Screen::ArenaSelect), crate::ui::spawn_arena_select)
    .add_systems(OnEnter(Screen::Settings), crate::ui::spawn_settings)
    .add_systems(OnEnter(Screen::Skins), crate::ui::spawn_skins)
    .add_systems(OnEnter(Screen::PowerUpSettings), crate::ui::spawn_powerup_settings)
    .add_systems(OnEnter(Screen::PowerUpEditor), crate::ui::spawn_powerup_editor)
    .add_systems(OnEnter(Screen::Inventory), crate::ui::spawn_inventory_management)
    .add_systems(OnEnter(Screen::Controls), crate::ui::spawn_controls)
    .add_systems(OnEnter(Screen::GamepadControls), crate::ui::spawn_gamepad_controls)
    .add_systems(OnEnter(Screen::HighScores), crate::ui::spawn_high_scores)
    .add_systems(OnEnter(Screen::Progress), crate::ui::spawn_progress)
    .add_systems(OnEnter(Screen::Credits), crate::ui::spawn_credits)
    .add_systems(OnEnter(Screen::NetworkGuide), crate::ui::spawn_network_guide)
    .add_systems(OnEnter(Screen::DeveloperDocument), crate::ui::spawn_developer_document)
    .add_systems(OnEnter(Screen::HighScoresRefresh), refresh_high_scores_screen)
    .add_systems(OnEnter(Screen::Playing), (crate::ui::cleanup_pause_overlay, systems::enter_playing).chain())
    .add_systems(OnEnter(Screen::Paused), crate::ui::spawn_pause_menu)
    .add_systems(OnEnter(Screen::QuickSettings), crate::ui::spawn_quick_settings)
    .add_systems(OnEnter(Screen::RestartConfirm), crate::ui::spawn_restart_confirmation)
    .add_systems(
        OnEnter(Screen::Results),
        (
            crate::ui::cleanup_pause_overlay,
            systems::finish_match,
            systems::cleanup_game,
            crate::ui::spawn_results,
        )
            .chain(),
    )
    .add_systems(OnExit(Screen::MainMenu), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::NetworkLobby), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::NetworkGuide), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Difficulty), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::ScoreLimit), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::OpenPlaySetup), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::ArenaSelect), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Settings), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Skins), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::PowerUpSettings), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::PowerUpEditor), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Inventory), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Controls), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::GamepadControls), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::HighScores), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Progress), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Credits), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::DeveloperDocument), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Paused), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::QuickSettings), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::RestartConfirm), crate::ui::cleanup_screen)
    .add_systems(OnExit(Screen::Results), crate::ui::cleanup_screen);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(OnEnter(Screen::NetworkLobby), crate::ui::spawn_network_lobby);
}

fn register_menu_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            
            crate::ui::detect_gameplay_input_device,
            crate::ui::capture_ui_scale_baselines,
            crate::ui::apply_ui_scale,
            crate::ui::update_ui_scale_slider,
            crate::ui::restore_navigation_focus,
            crate::ui::menu_input,
            crate::ui::update_menu_hover,
            crate::ui::capture_navigation_focus,
            crate::ui::update_contextual_help,
            crate::ui::update_navigation_hint_text,
            crate::ui::update_highlight,
        )
            .chain()
            .run_if(is_menu_screen),
    )
    .add_systems(Update, crate::ui::update_input_context)
    .add_systems(
        Update,
        crate::ui::update_powerup_editor_preview.run_if(in_state(Screen::PowerUpEditor)),
    )
    .add_systems(
        Update,
        (crate::ui::update_arena_preview, crate::ui::animate_arena_preview)
            .chain()
            .run_if(in_state(Screen::ArenaSelect))
            .after(crate::ui::menu_input),
    )
    .add_systems(Update, crate::ui::animate_background.run_if(is_menu_screen))
    .add_systems(Update, crate::ui::background_click_effects.run_if(is_menu_screen))
    .add_systems(Update, crate::ui::update_skin_preview.run_if(in_state(Screen::Skins)))
    .add_systems(Update, crate::ui::update_high_score_tooltip.run_if(in_state(Screen::HighScores)))
    .add_systems(Update, crate::ui::update_network_guide.run_if(in_state(Screen::NetworkGuide)))
    .add_systems(Update, crate::ui::animate_social_icons.run_if(in_state(Screen::Credits)))
    .add_systems(Update, crate::ui::update_developer_document.run_if(in_state(Screen::DeveloperDocument)));
}

fn register_match_systems(app: &mut App) {
    app.add_systems(
        Update,
        crate::ui::detect_gameplay_input_device.run_if(in_state(Screen::Playing)),
    )
        .add_systems(Update, systems::pause_on_focus_loss.run_if(in_state(Screen::Playing)))
        .add_systems(Update, systems::toggle_pause.run_if(in_state(Screen::Playing)))
        .add_systems(
            Update,
            (
                crate::ui::capture_ui_scale_baselines,
                crate::ui::apply_ui_scale,
                systems::player_movement,
                ai::move_ai,
                mechanics::activate_inventory_items,
                mechanics::tick_mechanics,
                mechanics::activate_ai_inventory_items,
                systems::move_ball,
                systems::check_scoring,
                systems::animate_celebration,
                systems::update_hud,
            )
                .chain()
                .run_if(in_state(Screen::Playing)),
        );
}


fn refresh_high_scores_screen(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::HighScores);
}
