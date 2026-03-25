use bevy::prelude::*;

/// Marker component for enemy entities.
#[derive(Component)]
pub struct Enemy;

/// Armor type determines damage multiplier from the damage matrix.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
    Fortified,
    Hero,
    Unarmored,
}

/// Enemy movement speed in units per second.
#[derive(Component, Debug)]
pub struct MoveSpeed {
    pub base: f32,
    pub multiplier: f32, // Modified by frost, etc.
}

impl MoveSpeed {
    pub fn effective(&self) -> f32 {
        self.base * self.multiplier
    }
}

impl Default for MoveSpeed {
    fn default() -> Self {
        Self {
            base: 100.0,
            multiplier: 1.0,
        }
    }
}

/// Enemy armor value. Uses same WC3 formula as tower armor.
/// Scales with elapsed time: +1 armor per 30 seconds.
#[derive(Component, Debug, Default)]
pub struct EnemyArmor {
    pub value: f32,
}

impl EnemyArmor {
    /// WC3 armor formula: reduction = (armor * 0.06) / (1 + armor * 0.06)
    pub fn damage_reduction(&self) -> f32 {
        if self.value <= 0.0 { return 0.0; }
        (self.value * 0.06) / (1.0 + self.value * 0.06)
    }

    pub fn apply(&self, raw_damage: f32) -> f32 {
        raw_damage * (1.0 - self.damage_reduction())
    }
}

/// Enemy health pool.
#[derive(Component, Debug)]
pub struct EnemyHealth {
    pub current: f32,
    pub max: f32,
}

/// Gold bounty awarded on kill.
#[derive(Component, Debug)]
pub struct GoldBounty {
    pub base: u32,
}

impl Default for GoldBounty {
    fn default() -> Self {
        Self { base: 10 }
    }
}

/// Enemy attack stats — damage dealt to tower on cooldown.
#[derive(Component, Debug)]
pub struct EnemyAttack {
    pub damage: f32,
    pub cooldown: f32,
    pub timer: f32,
    pub range: f32,
}

/// Frost stacks on this enemy (from Frost weapons).
#[derive(Component, Debug, Default)]
pub struct FrostStacks {
    pub stacks: u32,
    pub decay_timer: f32,
    pub frozen: bool,
    pub freeze_timer: f32,
}

/// Whether this enemy is burning (from Fire weapons).
#[derive(Component, Debug, Default)]
pub struct Burning {
    pub active: bool,
    pub fire_damage: f32, // Damage on death explosion
}

/// Marker for the boss entity.
#[derive(Component)]
pub struct Boss;

/// Marker for enemy health bar background.
#[derive(Component)]
pub struct EnemyHealthBarBg;

/// Marker for enemy health bar fill. Stores the full width for proportional scaling.
#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub full_width: f32,
}

/// Marker for enemy HP text display below the sprite.
#[derive(Component)]
pub struct EnemyHpText;

/// Marker for the armor type legend panel.
#[derive(Component)]
pub struct ArmorLegendPanel;

/// Individual legend entry with its armor type.
#[derive(Component)]
pub struct ArmorLegendEntry(pub ArmorType);

/// Text label within a legend entry (distinguishes from color swatch).
#[derive(Component)]
pub struct ArmorLegendText;

/// Floating text that rises and fades (gold popups, damage numbers).
#[derive(Component)]
pub struct FloatingText {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub rise_speed: f32,
}
