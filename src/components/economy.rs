use bevy::prelude::*;

/// Gold resource — the player's currency.
#[derive(Resource, Debug)]
pub struct Gold {
    pub current: u32,
    pub per_second: f32,
    pub per_second_bonus_percent: f32,
    pub bounty_bonus_percent: f32,
    pub total_earned: u32,
    pub total_spent: u32,
    /// Fractional gold accumulator for per-second income.
    pub income_accumulator: f32,
}

impl Default for Gold {
    fn default() -> Self {
        Self {
            current: STARTING_GOLD,
            per_second: BASE_GOLD_PER_SECOND,
            per_second_bonus_percent: 0.0,
            bounty_bonus_percent: 0.0,
            total_earned: 0,
            total_spent: 0,
            income_accumulator: 0.0,
        }
    }
}

/// Shop state resource.
#[derive(Resource, Debug)]
pub struct ShopState {
    pub weapon_slots: [Option<ShopItem>; 4],
    pub upgrade_slots: [Option<ShopItem>; 4],
    pub refresh_timer: f32,
    pub rerolls_this_cycle: u32,
    pub total_rerolls_this_run: u32,
}

impl Default for ShopState {
    fn default() -> Self {
        Self {
            weapon_slots: [None, None, None, None],
            upgrade_slots: [None, None, None, None],
            refresh_timer: SHOP_REFRESH_INTERVAL,
            rerolls_this_cycle: 0,
            total_rerolls_this_run: 0,
        }
    }
}

impl ShopState {
    pub fn reroll_cost(&self) -> u32 {
        REROLL_BASE_COST + (self.total_rerolls_this_run * REROLL_INCREMENT)
    }
}

/// An item in a shop slot.
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub name: String,
    pub rarity: Rarity,
    pub item_type: ShopItemType,
    pub definition_index: usize,
    /// Override rarity-based price (e.g., 0 for Philosopher's Stone).
    pub price_override: Option<u32>,
}

impl ShopItem {
    pub fn price(&self) -> u32 {
        self.price_override.unwrap_or_else(|| self.rarity.price())
    }
}

/// Item type in the shop.
#[derive(Debug, Clone, PartialEq)]
pub enum ShopItemType {
    Weapon,
    Upgrade,
}

/// Rarity tiers with fixed prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    pub fn price(&self) -> u32 {
        match self {
            Rarity::Common => 500,
            Rarity::Uncommon => 2_000,
            Rarity::Rare => 5_000,
            Rarity::Epic => 10_000,
        }
    }
}

/// Event: weapon was purchased from shop.
#[derive(Message)]
pub struct WeaponPurchasedEvent {
    pub weapon_name: String,
    pub definition_index: usize,
}

/// Event: upgrade was purchased from shop.
#[derive(Message)]
pub struct UpgradePurchasedEvent {
    pub upgrade_name: String,
    pub definition_index: usize,
}

// Constants
pub const STARTING_GOLD: u32 = 5_000;
pub const SHOP_REFRESH_INTERVAL: f32 = 30.0;
pub const REROLL_BASE_COST: u32 = 100;
pub const REROLL_INCREMENT: u32 = 150;
pub const MAX_REROLLS_PER_CYCLE: u32 = 5;
pub const BASE_KILL_BOUNTY: u32 = 10;
pub const BASE_GOLD_PER_SECOND: f32 = 50.0;
