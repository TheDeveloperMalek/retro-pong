use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::state::prelude::StateTransition;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
struct UiFontStartup;



#[derive(Resource, Clone)]
pub(crate) struct UiFont {
    pub(crate) source: FontSource,
}

pub struct UiFontPlugin;

impl Plugin for UiFontPlugin {
    fn build(&self, app: &mut App) {
        
        
        app.add_schedule(Schedule::new(UiFontStartup));
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_startup_before(StateTransition, UiFontStartup);
        app.add_systems(UiFontStartup, load_ui_font);
    }
}

#[cfg(target_arch = "wasm32")]
fn load_ui_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: FontSource = asset_server.load("fonts/NotoSansMono-VF.ttf").into();
    commands.insert_resource(UiFont { source: font });
}

#[cfg(not(target_arch = "wasm32"))]
fn load_ui_font(mut commands: Commands) {
    commands.insert_resource(UiFont {
        source: FontSource::monospace(),
    });
}

mod input;
mod screens;

pub use input::{
    capture_navigation_focus, detect_gameplay_input_device, menu_input, restore_navigation_focus,
    update_highlight, update_input_context,
};
pub(crate) use input::read_navigation_actions;
pub use screens::{
    animate_background,
    capture_ui_scale_baselines,
    apply_ui_scale,
    update_ui_scale_slider,
    animate_social_icons,
    update_developer_document,
    background_click_effects,
    cleanup_screen,
    cleanup_pause_overlay,
    spawn_controls,
    spawn_gamepad_controls,
    spawn_credits,
    spawn_difficulty,
    spawn_developer_document,
    spawn_high_scores,
    spawn_main_menu,
    spawn_pause_menu,
    spawn_quick_settings,
    spawn_restart_confirmation,
    spawn_progress,
    spawn_powerup_editor,
    spawn_powerup_settings,
    spawn_inventory_management,
    spawn_results,
    spawn_score_limit,
    spawn_settings,
    spawn_skins,
    update_skin_preview,
    update_menu_hover,
    update_high_score_tooltip,
    update_contextual_help,
    update_powerup_editor_preview,
    spawn_network_guide,
    update_network_guide,
    update_navigation_hint_text,
    spawn_open_play_setup,
    spawn_arena_select,
    update_arena_preview,
    animate_arena_preview,
};
pub(crate) use screens::{powerup_editor_field, PowerUpEditorField};

#[cfg(not(target_arch = "wasm32"))]
pub use screens::spawn_network_lobby;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use screens::{NetworkLobbyDetailsText, NetworkLobbyStatusText, NetworkLobbyStatusIndicator};
