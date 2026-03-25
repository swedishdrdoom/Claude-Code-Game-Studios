use bevy::prelude::*;

/// Damage types — determines multiplier against armor types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Normal,
    Piercing,
    Siege,
    Magic,
    Chaos,
}

/// Attack patterns — how a weapon's projectiles behave.
#[derive(Debug, Clone)]
pub enum AttackPattern {
    SingleTarget,
    Splash { radius: f32 },
    Bounce { max_targets: u32 },
    Barrage { target_count: u32 },
    Area { radius: f32 },
    Wave { bonus_range: f32 },
}

/// A weapon instance equipped on the tower.
/// Each instance fires independently with its own cooldown.
#[derive(Component, Debug)]
pub struct WeaponInstance {
    pub definition_index: usize,
    pub name: String,
    pub damage: f32,
    pub attack_cooldown: f32,
    pub cooldown_timer: f32,
    pub range: f32,
    pub damage_type: DamageType,
    pub attack_pattern: AttackPattern,
    /// Max HP gained per attack (Blood Bomb). Only this weapon's attacks trigger the gain.
    pub max_hp_per_attack: f32,
    /// Whether this weapon applies Frost on hit.
    pub applies_frost: bool,
}

/// Marker component for projectile entities.
#[derive(Component)]
pub struct Projectile;

/// Visual effect entity — expanding circle outline drawn with gizmos, then despawns.
#[derive(Component, Debug)]
pub struct AttackVfx {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub start_radius: f32,
    pub end_radius: f32,
    pub center: Vec2,
    pub damage_type: DamageType,
}

/// Cone-shaped VFX drawn with gizmos. Expands outward from origin
/// in a specific direction over its lifetime, then despawns.
#[derive(Component, Debug)]
pub struct ConeVfx {
    pub origin: Vec2,
    pub direction: Vec2,
    pub half_angle: f32,
    pub max_radius: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub damage_type: DamageType,
}

/// Marker for barrage projectiles that deal splash damage on landing.
/// These projectiles target a ground position, not a live entity.
#[derive(Component, Debug)]
pub struct BarrageSplash {
    pub splash_radius: f32,
}

/// Projectile movement data.
#[derive(Component, Debug)]
pub struct ProjectileData {
    pub target: Option<Entity>,
    pub target_position: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub damage_type: DamageType,
    pub attack_pattern: AttackPattern,
    pub source_weapon: String,
    /// Whether the source weapon applies Frost.
    pub applies_frost: bool,
    /// Entities already hit by this projectile (for bounce chains).
    pub hits: Vec<Entity>,
}

/// Event: enemy was hit by a projectile or AoE attack.
/// `is_primary_hit` is true for the first hit of each attack (projectile arrival,
/// first target in AoE). Splash/area secondary hits are false. Used to distinguish
/// "per attack" effects (Max HP on hit) from "per enemy hit" effects (heal on hit).
#[derive(Message)]
pub struct DamageEvent {
    pub target: Entity,
    pub damage: f32,
    pub damage_type: DamageType,
    pub position: Vec2,
    pub attack_pattern: AttackPattern,
    pub is_primary_hit: bool,
    pub applies_frost: bool,
}

/// Event: enemy was killed.
#[derive(Message)]
pub struct EnemyKilledEvent {
    pub position: Vec2,
    pub gold_bounty: u32,
    pub had_burning: bool,
    pub fire_damage: f32,
}

/// Event: tower took damage.
#[derive(Message)]
pub struct TowerDamageEvent {
    pub raw_damage: f32,
}

/// Event: tower was destroyed.
#[derive(Message)]
pub struct TowerDestroyedEvent;

/// Event: boss was killed.
#[derive(Message)]
pub struct BossKilledEvent;
