

mod hud;
mod lifecycle;
mod movement;
mod scoring;
mod world;

pub use hud::update_hud;
pub use lifecycle::{
    cleanup_game,
    finish_match,
    pause_on_focus_loss,
    refresh_paddle_skin,
    toggle_pause,
};
pub use movement::{move_ball, player_movement};
pub use scoring::check_scoring;
pub use world::{
    animate_celebration,
    ball_color_with_shade,
    paddle_color_with_shade,
    paddle_skin_asset_path,
    ball_skin_asset_path,
    ball_skin_sprite,
    enter_playing,
    setup_world,
};
pub(crate) use world::spawn_arena_preview_backdrop;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use world::spawn_storm_burst;
