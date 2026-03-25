mod components;
mod data;
mod plugins;

use bevy::prelude::*;
use bevy::window::WindowMode;
use components::run::GameState;
use plugins::TowerOfDoomPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower of Doom".to_string(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TowerOfDoomPlugin)
        .run();
}
