use bevy::prelude::*;

/// Tracks accumulated damage bonus percentages per type for HUD display.
#[derive(Resource, Debug, Default)]
pub struct DamageBonuses {
    pub normal: f32,
    pub piercing: f32,
    pub siege: f32,
    pub magic: f32,
    pub chaos: f32,
    pub global: f32,
    pub attack_speed: f32,
}
