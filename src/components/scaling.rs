use bevy::prelude::*;

/// A scaling buff that applies a stat increase on a timer.
#[derive(Debug, Clone)]
pub struct ScalingBuff {
    pub effect: ScalingEffect,
    pub amount: f32,
    pub interval: f32,
    pub timer: f32,
}

#[derive(Debug, Clone)]
pub enum ScalingEffect {
    SpikesDamage,
    GlobalDamagePercent,
    GoldPerSecond,
    Armor,
    MaxHp,
    HpRegen,
    ManaShield,
    InstantGold,
    PiercingDamagePercent,
    ChaosDamagePercent,
    MagicDamagePercent,
    SiegeDamagePercent,
    NormalDamagePercent,
}

/// Resource holding all active scaling buffs.
#[derive(Resource, Debug, Default)]
pub struct ScalingBuffs {
    pub buffs: Vec<ScalingBuff>,
}
