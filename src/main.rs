mod audio;
mod game;
mod localization;
mod persistence;
mod network;
mod platform_links;
mod progression;
mod random;
mod ui;

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use game::{GamePlugin, Screen};
use persistence::{PersistentData, load};

fn main() {
    let saved = load();
    let settings = saved.settings.clone();
    #[cfg(not(target_arch = "wasm32"))]
    let network_menu = network::client::NetworkMenuState::from_preferences(&saved.network_preferences);
    let initial_window_mode = if settings.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
    } else {
        WindowMode::Windowed
    };

    #[allow(unused_mut)]
    let mut default_plugins = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "RETRO PONG // ARCADE EDITION".into(),
                resolution: (960, 540).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: 640.0,
                    min_height: 360.0,
                    ..default()
                },
                resizable: true,
                mode: initial_window_mode,
                fit_canvas_to_parent: true,
                canvas: Some("#game-canvas".into()),
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        .set(ImagePlugin::default_nearest());

    
    
    #[cfg(target_arch = "wasm32")]
    {
        default_plugins = default_plugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        });
    }

    let mut app = App::new();
    app.insert_resource(PersistentData(saved))
        .insert_resource(settings)
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.075)))
        .add_plugins(default_plugins)
        .init_state::<Screen>()
        .add_plugins((audio::AudioPlugin, GamePlugin, ui::UiFontPlugin));

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.init_resource::<network::client::NetworkRuntime>()
            .insert_resource(network_menu)
            .add_systems(Update, network::client::poll_events)
            .add_systems(Update, network::client::network_lobby_controls.run_if(in_state(Screen::NetworkLobby)))
            .add_systems(Update, network::client::network_controls)
            .add_systems(Update, network::client::apply_snapshot.run_if(in_state(Screen::Playing)));
    }

    app.run();
}
