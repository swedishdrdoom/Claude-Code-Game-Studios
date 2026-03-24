use bevy::prelude::*;

/// Marker component for the tower entity.
#[derive(Component)]
pub struct Tower;

/// Tower health pool.
#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: STARTING_HP,
            max: STARTING_HP,
        }
    }
}

/// Tower armor value. Uses WC3 damage reduction formula.
#[derive(Component, Debug, Default)]
pub struct Armor {
    pub value: f32,
}

impl Armor {
    /// WC3 armor formula: reduction = (armor * 0.06) / (1 + armor * 0.06)
    pub fn damage_reduction(&self) -> f32 {
        if self.value >= 0.0 {
            (self.value * ARMOR_CONSTANT) / (1.0 + self.value * ARMOR_CONSTANT)
        } else {
            // Negative armor: multiplier = 2 - 0.94^(-armor), capped at 2.0
            let mult = 2.0 - 0.94_f32.powf(-self.value);
            // Return as negative reduction (amplification)
            -(mult - 1.0)
        }
    }

    /// Apply armor to incoming damage. Returns damage after armor.
    pub fn apply(&self, raw_damage: f32) -> f32 {
        if self.value >= 0.0 {
            raw_damage * (1.0 - self.damage_reduction())
        } else {
            let multiplier = 2.0_f32.min(2.0 - 0.94_f32.powf(-self.value));
            raw_damage * multiplier
        }
    }
}

/// Mana shield — absorbs raw damage before armor.
#[derive(Component, Debug, Default)]
pub struct ManaShield {
    pub current: f32,
    pub max: f32,
}

/// HP regeneration per second.
#[derive(Component, Debug, Default)]
pub struct HpRegen {
    pub per_second: f32,
}

/// Flat damage reduction (from Deflection upgrade).
/// Cannot reduce a hit below 25% of its post-armor value.
#[derive(Component, Debug, Default)]
pub struct FlatDamageReduction {
    pub value: f32,
}

/// HP restored per enemy kill (from Mask of Death, etc.)
#[derive(Component, Debug, Default)]
pub struct HealOnKill {
    pub amount: f32,
}

/// HP restored when tower is attacked (from Dreadlord Fang, etc.)
#[derive(Component, Debug, Default)]
pub struct HealOnAttacked {
    pub amount: f32,
}

/// Mana shield regeneration per second (from Moonwell, etc.)
#[derive(Component, Debug, Default)]
pub struct ManaShieldRegen {
    pub per_second: f32,
}

/// Mana shield restored per enemy kill (from Maw of Death, etc.)
#[derive(Component, Debug, Default)]
pub struct ManaShieldOnKill {
    pub amount: f32,
}

/// Mana shield restored per weapon hit (from Soulstealer, etc.)
#[derive(Component, Debug, Default)]
pub struct ManaShieldOnHit {
    pub amount: f32,
}

/// HP restored per weapon hit (from Healing Sprayer, Holy Bolt, etc.)
#[derive(Component, Debug, Default)]
pub struct HealPerHit {
    pub amount: f32,
}

/// Max HP gained per weapon hit (from Blood Bomb)
#[derive(Component, Debug, Default)]
pub struct MaxHpPerHit {
    pub amount: f32,
}

/// Spikes damage dealt back to melee attackers.
#[derive(Component, Debug, Default)]
pub struct SpikesDamage {
    pub flat: f32,
    pub bonus_percent: f32,
}

impl SpikesDamage {
    pub fn total(&self) -> f32 {
        self.flat * (1.0 + self.bonus_percent)
    }
}

/// Tracks all purchased upgrades for display.
#[derive(Resource, Debug, Default)]
pub struct PurchasedUpgrades {
    pub items: Vec<String>,
}

// Constants
pub const STARTING_HP: f32 = 1500.0;
pub const STARTING_ARMOR: f32 = 0.0;
pub const ARMOR_CONSTANT: f32 = 0.06;
