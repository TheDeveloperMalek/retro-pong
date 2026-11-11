mod about;
mod arena;
mod common;
mod main_menu;
mod network;
mod profile;
mod settings;

pub use arena::{animate_arena_preview, spawn_arena_select, update_arena_preview};
pub use about::{animate_social_icons, quick_settings_items, spawn_credits, spawn_developer_document, spawn_pause_menu, spawn_quick_settings, spawn_restart_confirmation, spawn_results, update_developer_document};
pub use common::{
    animate_background, background_click_effects, cleanup_screen, cleanup_pause_overlay, refresh_language_flag,
    capture_ui_scale_baselines, apply_ui_scale, update_ui_scale_slider,
    show_notice, update_navigation_hint_text,
};
pub use main_menu::spawn_main_menu;
#[cfg(not(target_arch = "wasm32"))]
pub use network::spawn_network_lobby;
pub use network::{spawn_network_guide, update_network_guide};
pub use profile::{spawn_high_scores, spawn_progress, update_high_score_tooltip};
pub use settings::{
    control_action_for_row,
    controls_items,
    gamepad_controls_items,
    inventory_management_items,
    menu_navigation_action_for_row,
    powerup_editor_items,
    powerup_settings_items,
    settings_items,
    skin_items,
    spawn_controls,
    spawn_difficulty,
    spawn_open_play_setup,
    spawn_gamepad_controls,
    spawn_inventory_management,
    spawn_powerup_editor,
    spawn_powerup_settings,
    spawn_score_limit,
    spawn_settings,
    spawn_skins,
    update_contextual_help,
    update_menu_hover,
    update_powerup_editor_preview,
    update_skin_preview,
};
pub(crate) use settings::{is_powerup_editor_row_selectable, powerup_editor_field, PowerUpEditorField};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use common::{NetworkLobbyDetailsText, NetworkLobbyStatusText, NetworkLobbyStatusIndicator};
