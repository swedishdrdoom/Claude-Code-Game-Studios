use serde::Deserialize;

/// Raw enemy definition from enemies.json.
#[derive(Debug, Clone, Deserialize)]
pub struct EnemyDefinition {
    pub name: String,
    pub armor_type: String,
    pub max_hp: f32,
    pub damage: f32,
    pub attack_cooldown: f32,
    pub move_speed: f32,
    pub gold_bounty: u32,
    pub frost_resistance: f32,
    pub stun_resistance: f32,
}
