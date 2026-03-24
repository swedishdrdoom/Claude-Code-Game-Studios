use bevy::prelude::*;

use crate::data::damage_matrix::DamageMatrix;
use crate::data::weapons::WeaponDefinition;
use crate::data::upgrades::UpgradeDefinition;

/// Database of all weapon definitions, loaded from JSON.
#[derive(Resource, Debug)]
pub struct WeaponDatabase {
    pub weapons: Vec<WeaponDefinition>,
}

/// Database of all upgrade definitions, loaded from JSON.
#[derive(Resource, Debug)]
pub struct UpgradeDatabase {
    pub upgrades: Vec<UpgradeDefinition>,
}

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DamageMatrix::default())
            .add_systems(PreStartup, load_content);
    }
}

fn load_content(mut commands: Commands) {
    // Load weapons
    let weapons_json = std::fs::read_to_string("assets/content/weapons.json")
        .expect("Failed to load assets/content/weapons.json");
    let weapons: Vec<WeaponDefinition> = serde_json::from_str(&weapons_json)
        .expect("Failed to parse weapons.json");
    info!("Loaded {} weapons", weapons.len());
    commands.insert_resource(WeaponDatabase { weapons });

    // Load upgrades
    let upgrades_json = std::fs::read_to_string("assets/content/upgrades.json")
        .expect("Failed to load assets/content/upgrades.json");
    let upgrades: Vec<UpgradeDefinition> = serde_json::from_str(&upgrades_json)
        .expect("Failed to parse upgrades.json");
    info!("Loaded {} upgrades", upgrades.len());
    commands.insert_resource(UpgradeDatabase { upgrades });
}
