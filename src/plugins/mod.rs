pub mod combat;
pub mod content;
pub mod core;
pub mod debug;
pub mod economy;
pub mod enemy;
pub mod run;
pub mod ui;

use bevy::prelude::*;

/// Master plugin that registers all game plugins.
pub struct TowerOfDoomPlugin;

impl Plugin for TowerOfDoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            content::ContentPlugin,
            run::RunPlugin,
            core::CorePlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            economy::EconomyPlugin,
            ui::UiPlugin,
            // debug::DebugPlugin, // Disabled — enable with backtick when needed
        ));
    }
}
