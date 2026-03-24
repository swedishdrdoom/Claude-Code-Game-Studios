use bevy::prelude::*;
use rand::Rng;

use crate::components::combat::*;
use crate::components::economy::*;
use crate::components::run::GameState;
use crate::components::scaling::*;
use crate::components::stats::DamageBonuses;
use crate::components::tower::*;
use crate::plugins::content::{WeaponDatabase, UpgradeDatabase};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Gold>()
            .init_resource::<ShopState>()
            .add_message::<WeaponPurchasedEvent>()
            .add_message::<UpgradePurchasedEvent>()
            .add_systems(Update, (
                tick_passive_income.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Boss)),
                ),
                update_shop_timer.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::GracePeriod))
                        .or(in_state(GameState::Boss)),
                ),
                handle_weapon_purchase,
                handle_upgrade_purchase,
            ))
            .add_systems(OnEnter(GameState::GracePeriod), generate_initial_shop);
    }
}

fn roll_rarity() -> Rarity {
    let mut rng = rand::rng();
    let roll: f32 = rng.random();
    if roll < 0.40 {
        Rarity::Common
    } else if roll < 0.75 {
        Rarity::Uncommon
    } else if roll < 0.95 {
        Rarity::Rare
    } else {
        Rarity::Epic
    }
}

fn rarity_matches(item_rarity: &str, target: &Rarity) -> bool {
    match target {
        Rarity::Common => item_rarity == "Common",
        Rarity::Uncommon => item_rarity == "Uncommon",
        Rarity::Rare => item_rarity == "Rare",
        Rarity::Epic => item_rarity == "Epic",
    }
}

/// Parse weapon damage type from the JSON "Weapon" field.
fn parse_damage_type(weapon_type: &str) -> DamageType {
    let primary = weapon_type.split(" and ").next().unwrap_or(weapon_type);
    match primary {
        "Normal" => DamageType::Normal,
        "Piercing" => DamageType::Piercing,
        "Siege" => DamageType::Siege,
        "Magic" => DamageType::Magic,
        "Chaos" => DamageType::Chaos,
        _ => DamageType::Normal,
    }
}

/// Parse attack pattern from the JSON "AttackType" field.
fn parse_attack_pattern(attack_type: &str) -> AttackPattern {
    if attack_type.starts_with("Splash") {
        let radius = extract_number(attack_type).unwrap_or(150.0);
        AttackPattern::Splash { radius }
    } else if attack_type.starts_with("Bounce") {
        let targets = extract_number(attack_type).unwrap_or(4.0) as u32;
        AttackPattern::Bounce { max_targets: targets }
    } else if attack_type.starts_with("Barrage") {
        let targets = extract_number(attack_type).unwrap_or(4.0) as u32;
        AttackPattern::Barrage { target_count: targets }
    } else if attack_type.starts_with("Area") {
        if attack_type.contains("Enemies in Range") || attack_type.contains("Enemies In Range") {
            // "Area (Enemies in Range)" — uses weapon range as radius, set to 0 as sentinel
            // The actual range will be the weapon's range value
            AttackPattern::Area { radius: 0.0 }
        } else {
            let radius = extract_number(attack_type).unwrap_or(150.0);
            AttackPattern::Area { radius }
        }
    } else if attack_type.starts_with("Wave") {
        let bonus = extract_number(attack_type).unwrap_or(300.0);
        AttackPattern::Wave { bonus_range: bonus }
    } else {
        AttackPattern::SingleTarget
    }
}

/// Extract a number from a string like "Splash (300)" or "Bounce (8 Targets)".
fn extract_number(s: &str) -> Option<f32> {
    let start = s.find('(')?;
    let after = &s[start + 1..];
    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    num_str.parse().ok()
}

pub fn generate_shop_inventory(
    shop: &mut ShopState,
    weapons: &WeaponDatabase,
    upgrades: &UpgradeDatabase,
) {
    let mut rng = rand::rng();

    // Generate 4 weapon slots
    for slot in &mut shop.weapon_slots {
        let rarity = roll_rarity();
        let matching: Vec<(usize, &str)> = weapons.weapons.iter().enumerate()
            .filter(|(_, w)| rarity_matches(&w.rarity, &rarity))
            .map(|(i, w)| (i, w.name.as_str()))
            .collect();

        if matching.is_empty() {
            *slot = None;
            continue;
        }

        let idx = rng.random_range(0..matching.len());
        let (def_idx, name) = matching[idx];
        *slot = Some(ShopItem {
            name: name.to_string(),
            rarity,
            item_type: ShopItemType::Weapon,
            definition_index: def_idx,
        });
    }

    // Generate 4 upgrade slots with weighted type selection
    // Income/Bounty upgrades appear slightly more often
    for slot in &mut shop.upgrade_slots {
        let rarity = roll_rarity();
        let matching: Vec<(usize, &str, &str)> = upgrades.upgrades.iter().enumerate()
            .filter(|(_, u)| rarity_matches(&u.rarity, &rarity))
            .map(|(i, u)| (i, u.name.as_str(), u.upgrade_type.as_str()))
            .collect();

        if matching.is_empty() {
            *slot = None;
            continue;
        }

        // Build weighted selection: Income/Bounty get 3x weight
        let mut weighted: Vec<(usize, &str)> = Vec::new();
        for (def_idx, name, utype) in &matching {
            let weight = match *utype {
                "Income" | "Bounty" => 3,
                _ => 1,
            };
            for _ in 0..weight {
                weighted.push((*def_idx, *name));
            }
        }

        let idx = rng.random_range(0..weighted.len());
        let (def_idx, name) = weighted[idx];
        *slot = Some(ShopItem {
            name: name.to_string(),
            rarity,
            item_type: ShopItemType::Upgrade,
            definition_index: def_idx,
        });
    }
}

fn generate_initial_shop(
    mut shop: ResMut<ShopState>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
) {
    generate_shop_inventory(&mut shop, &weapons, &upgrades);
    info!("Initial shop generated");
}

fn tick_passive_income(
    time: Res<Time>,
    mut gold: ResMut<Gold>,
) {
    if gold.per_second > 0.0 {
        let effective_rate = gold.per_second * (1.0 + gold.per_second_bonus_percent);
        gold.income_accumulator += time.delta_secs();
        // Tick once per second
        if gold.income_accumulator >= 1.0 {
            gold.income_accumulator -= 1.0;
            let earned = effective_rate as u32;
            gold.current += earned;
            gold.total_earned += earned;
        }
    }
}

fn update_shop_timer(
    time: Res<Time>,
    mut shop: ResMut<ShopState>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
) {
    shop.refresh_timer -= time.delta_secs();
    if shop.refresh_timer <= 0.0 {
        shop.refresh_timer = SHOP_REFRESH_INTERVAL;
        shop.rerolls_this_cycle = 0;
        generate_shop_inventory(&mut shop, &weapons, &upgrades);
        info!("Shop refreshed!");
    }
}

/// When a weapon purchase event fires, spawn the weapon instance.
fn handle_weapon_purchase(
    mut events: MessageReader<WeaponPurchasedEvent>,
    weapons: Res<WeaponDatabase>,
    mut commands: Commands,
    mut tower_query: Query<(
        &mut HealPerHit, &mut ManaShieldOnHit, &mut MaxHpPerHit,
    ), With<Tower>>,
) {
    let Ok((mut heal_per_hit, mut shield_on_hit, mut maxhp_on_hit)) = tower_query.single_mut() else {
        return;
    };

    for event in events.read() {
        let def = &weapons.weapons[event.definition_index];
        let damage_type = parse_damage_type(&def.weapon_type);
        let attack_pattern = parse_attack_pattern(&def.attack_type);

        commands.spawn(WeaponInstance {
            definition_index: event.definition_index,
            name: def.name.clone(),
            damage: def.damage as f32,
            attack_cooldown: def.attack_cooldown,
            cooldown_timer: 0.0,
            range: def.range as f32,
            damage_type,
            attack_pattern,
        });

        // Parse weapon on-hit abilities
        let ability = &def.ability;
        if ability.contains("Heal") && ability.contains("per enemy hit") {
            // Healing Sprayer: "Heal 10 health per enemy hit"
            // Holy Bolt: "Heal 40 health per enemy hit"
            // Chain Heal: "Heal 200 health per enemy hit"
            if let Some(val) = parse_first_number(ability) {
                heal_per_hit.amount += val;
                info!("  On-hit: +{} HP per hit (total: {})", val, heal_per_hit.amount);
            }
        }
        if ability.contains("Mana Shield per enemy hit") || ability.contains("Mana Shield Per Enemy Hit") {
            // Soulstealer: "Attacks grant +3 Mana Shield per enemy hit."
            if let Some(val) = parse_first_number(ability) {
                shield_on_hit.amount += val;
                info!("  On-hit: +{} Mana Shield per hit (total: {})", val, shield_on_hit.amount);
            }
        }
        if ability.contains("Max HP per enemy hit") || ability.contains("Max HP Per Enemy Hit") {
            // Blood Bomb: "Attacks grant +3 Max HP per enemy hit."
            if let Some(val) = parse_first_number(ability) {
                maxhp_on_hit.amount += val;
                info!("  On-hit: +{} Max HP per hit (total: {})", val, maxhp_on_hit.amount);
            }
        }

        info!("Weapon equipped: {} ({})", def.name, def.rarity);
    }
}

/// Parse a number from an upgrade description like "+500 Max HP" or "+10% Attack Speed".
fn parse_first_number(desc: &str) -> Option<f32> {
    let mut chars = desc.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == '+' || c == '-' || c.is_ascii_digit() {
            let mut num_str = String::new();
            if c == '+' {
                chars.next();
            } else if c == '-' {
                num_str.push('-');
                chars.next();
            }
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() || d == '.' {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if !num_str.is_empty() && num_str != "-" {
                return num_str.parse().ok();
            }
        }
        chars.next();
    }
    None
}

/// Apply upgrade effects to tower stats based on upgrade type and description.
/// Parse "every N seconds" scaling effects from description and register them.
fn register_scaling_effects(desc: &str, buffs: &mut ScalingBuffs) {
    for part in desc.split('.') {
        let part = part.trim();
        if !part.contains("every") || !part.contains("second") {
            continue;
        }

        let amount = match parse_first_number(part) {
            Some(v) => v,
            None => continue,
        };

        // Parse interval: "every 30 seconds" or "every second" (= 1s)
        let interval = if part.contains("every second") || part.contains("every 1 second") {
            1.0
        } else if let Some(idx) = part.find("every ") {
            let after = &part[idx + 6..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            num_str.parse::<f32>().unwrap_or(30.0)
        } else {
            30.0
        };

        let effect = if part.contains("Spikes Damage") || part.contains("Spikes damage") {
            ScalingEffect::SpikesDamage
        } else if part.contains("Damage") && part.contains('%') {
            if part.contains("Piercing") { ScalingEffect::PiercingDamagePercent }
            else if part.contains("Chaos") { ScalingEffect::ChaosDamagePercent }
            else if part.contains("Magic") { ScalingEffect::MagicDamagePercent }
            else if part.contains("Siege") { ScalingEffect::SiegeDamagePercent }
            else if part.contains("Normal") { ScalingEffect::NormalDamagePercent }
            else { ScalingEffect::GlobalDamagePercent }
        } else if (part.contains("Receive") || part.contains("receive")) && part.contains("gold") {
            // "Receive 1000 gold every 30 seconds" — instant gold grant on timer
            ScalingEffect::InstantGold
        } else if part.contains("Gold per second") || part.contains("Gold Per Second") {
            // "+1 Gold per second every 30 seconds" — income rate increase on timer
            ScalingEffect::GoldPerSecond
        } else if part.contains("Armor") {
            ScalingEffect::Armor
        } else if part.contains("Max HP") || part.contains("Max Health") {
            ScalingEffect::MaxHp
        } else if part.contains("HP Regen") || part.contains("Health Regen") {
            ScalingEffect::HpRegen
        } else if part.contains("Mana Shield") || part.contains("Mana") {
            ScalingEffect::ManaShield
        } else if part.contains("gold") && !part.contains("per second") {
            ScalingEffect::InstantGold
        } else {
            continue;
        };

        buffs.buffs.push(ScalingBuff {
            effect,
            amount,
            interval,
            timer: interval,
        });
        info!("  Scaling buff: +{} {:?} every {}s", amount, &buffs.buffs.last().unwrap().effect, interval);
    }
}

fn handle_upgrade_purchase(
    mut events: MessageReader<UpgradePurchasedEvent>,
    upgrades: Res<UpgradeDatabase>,
    mut tower_query: Query<(
        &mut Health, &mut Armor, &mut ManaShield, &mut HpRegen,
        &mut SpikesDamage, &mut FlatDamageReduction,
        &mut ManaShieldRegen, &mut ManaShieldOnKill, &mut ManaShieldOnHit,
        &mut HealOnKill, &mut HealOnAttacked,
    ), With<Tower>>,
    mut gold: ResMut<Gold>,
    mut weapon_query: Query<&mut WeaponInstance>,
    mut scaling_buffs: ResMut<ScalingBuffs>,
    mut damage_bonuses: ResMut<DamageBonuses>,
) {
    let Ok((
        mut health, mut armor, mut mana_shield, mut regen,
        mut spikes, mut flat_red,
        mut shield_regen, mut shield_on_kill, mut shield_on_hit,
        mut heal_on_kill, mut heal_on_attacked,
    )) = tower_query.single_mut() else {
        return;
    };

    for event in events.read() {
        let def = &upgrades.upgrades[event.definition_index];
        let desc = &def.description;
        let upgrade_type = def.upgrade_type.as_str();

        let value = parse_first_number(desc).unwrap_or(0.0);

        match upgrade_type {
            "HP" => {
                // +N Max HP
                health.max += value;
                health.current = (health.current + value).min(health.max);

                // Parse secondary effects
                for part in desc.split('.') {
                    let part = part.trim();
                    if part.contains("when enemy dies") || part.contains("when an enemy dies") {
                        // Mask of Death: "Heal 15 health when enemy dies"
                        if let Some(heal_val) = parse_first_number(part) {
                            heal_on_kill.amount += heal_val;
                            info!("  +{} HP on kill", heal_val);
                        }
                    }
                }

                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{} Max HP (now {}/{})", value, health.current, health.max);
            }
            "Regen" => {
                // +N Health Regen
                regen.per_second += value;
                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{} HP Regen (now {}/s)", value, regen.per_second);
            }
            "Armor" => {
                // +N Armor
                armor.value += value;
                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{} Armor (now {})", value, armor.value);
            }
            "Mana Shield" => {
                // +N Mana Shield (flat amount)
                mana_shield.max += value;
                mana_shield.current += value;

                // Parse secondary effects
                if desc.contains("every second") || desc.contains("per second") {
                    // Moonwell: "+10 Mana Shield every second"
                    // Extract the regen number (appears after the first number)
                    let parts: Vec<&str> = desc.split('.').collect();
                    if parts.len() > 1 {
                        if let Some(regen_val) = parse_first_number(parts[1]) {
                            shield_regen.per_second += regen_val;
                            info!("  +{} Mana Shield regen/s", regen_val);
                        }
                    }
                }
                if desc.contains("when an enemy dies") || desc.contains("when enemy dies") {
                    // Maw of Death: "Restore 15 mana when an enemy dies"
                    let parts: Vec<&str> = desc.split('.').collect();
                    if parts.len() > 1 {
                        if let Some(restore_val) = parse_first_number(parts[1]) {
                            shield_on_kill.amount += restore_val;
                            info!("  +{} Mana Shield on kill", restore_val);
                        }
                    }
                }

                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{} Mana Shield (now {}/{})", value, mana_shield.current, mana_shield.max);
            }
            "Spikes" => {
                // +N Spikes Damage
                spikes.flat += value;

                // Parse secondary effects
                for part in desc.split('.') {
                    let part = part.trim();
                    if part.contains("when attacked") {
                        if let Some(heal_val) = parse_first_number(part) {
                            if part.contains("Heal") || part.contains("health") || part.contains("HP") {
                                // Dreadlord Fang: "Heal 4 health when attacked"
                                heal_on_attacked.amount += heal_val;
                                info!("  +{} HP when attacked", heal_val);
                            }
                        }
                    }
                }

                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{} Spikes Damage (now {})", value, spikes.total());
            }
            "Attack Boost" => {
                // +N% Attack Speed — reduce all weapon cooldowns
                let pct = value / 100.0;
                for mut weapon in &mut weapon_query {
                    weapon.attack_cooldown *= 1.0 / (1.0 + pct);
                }
                damage_bonuses.attack_speed += value;
                info!("Upgrade: +{}% Attack Speed", value);
            }
            "Damage Boost" => {
                // +N% damage — apply to matching weapons and track
                let pct = value / 100.0;
                if desc.contains("Normal") {
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Normal) { w.damage *= 1.0 + pct; }
                    }
                    damage_bonuses.normal += value;
                } else if desc.contains("Piercing") {
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Piercing) { w.damage *= 1.0 + pct; }
                    }
                    damage_bonuses.piercing += value;
                } else if desc.contains("Magic") {
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Magic) { w.damage *= 1.0 + pct; }
                    }
                    damage_bonuses.magic += value;
                } else if desc.contains("Siege") {
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Siege) { w.damage *= 1.0 + pct; }
                    }
                    damage_bonuses.siege += value;
                } else if desc.contains("Chaos") {
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Chaos) { w.damage *= 1.0 + pct; }
                    }
                    damage_bonuses.chaos += value;
                } else if desc.contains("Fire") || desc.contains("Poison") || desc.contains("Frost") {
                    info!("Upgrade: element damage boost (not yet implemented)");
                } else {
                    // Global damage boost
                    for mut w in &mut weapon_query {
                        w.damage *= 1.0 + pct;
                    }
                    damage_bonuses.global += value;
                }
                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: +{}% Damage ({})", value, desc);
            }
            "Income" => {
                // Parse each sentence in the description
                for part in desc.split('.') {
                    let part = part.trim();
                    if part.is_empty() { continue; }

                    // Skip kill-counter milestone mechanics (not yet implemented)
                    if part.contains("enemies killed") || part.contains("Purchasing a second") || part.contains("When used") {
                        info!("  Skipping complex mechanic: {}", part);
                        continue;
                    }

                    // Skip "every N seconds" sentences — handled by register_scaling_effects
                    if part.contains("every") && part.contains("second") {
                        continue;
                    }

                    if let Some(num) = parse_first_number(part) {
                        if part.contains("Gold per second") || part.contains("Gold Per Second") {
                            // "+5 Gold per second", "+10 Gold per second"
                            gold.per_second += num;
                            info!("  +{} Gold/sec", num);
                        } else if part.contains("% Gold per second") {
                            // "+10% Gold per second"
                            gold.per_second_bonus_percent += num / 100.0;
                            info!("  +{}% Gold/sec bonus", num);
                        } else if part.contains("Max Health") || part.contains("Max HP") {
                            // "-1000 Max Health" (Philosopher's Stone)
                            health.max += num; // num is negative
                            health.current = health.current.min(health.max);
                            if health.max <= 0.0 { health.current = 0.0; }
                            info!("  {} Max HP (now {}/{})", num, health.current, health.max);
                        } else if part.contains("HP Regen") {
                            // "-100 HP Regen" (Cursed Treasure)
                            regen.per_second += num; // num is negative
                            info!("  {} HP Regen", num);
                        } else if part.contains("Gold") && !part.contains("per second") && !part.contains("enemies") {
                            // "+2000 Gold", "+5000 Gold" (instant gold grant)
                            let gold_amount = num.abs() as u32;
                            gold.current += gold_amount;
                            gold.total_earned += gold_amount;
                            info!("  +{} instant Gold", gold_amount);
                        } else if part.contains("Kill Bounty") {
                            gold.bounty_bonus_percent += num / 100.0;
                            info!("  +{}% Kill Bounty", num);
                        }
                    }
                }
                register_scaling_effects(desc, &mut scaling_buffs);
                info!("Upgrade: {} (Gold/sec now {:.0})", def.name, gold.per_second);
            }
            "Bounty" => {
                // +N% Kill Bounty
                let pct = value / 100.0;
                gold.bounty_bonus_percent += pct;
                info!("Upgrade: +{}% Kill Bounty (now +{}%)", value, gold.bounty_bonus_percent * 100.0);
            }
            "Critical" => {
                // +N% Critical Strike Chance — store on a resource (not yet tracked per-weapon)
                info!("Upgrade: +{}% Crit Chance (tracking not yet implemented)", value);
            }
            "Healing" => {
                info!("Upgrade: Healing bonus (not yet implemented)");
            }
            "Frost" | "Stun" | "Dodge" => {
                info!("Upgrade: {} effect (not yet implemented)", upgrade_type);
            }
            "Utility" => {
                // Special upgrades (Black Market, Duplicator, Multiplication Gems)
                info!("Upgrade: Utility '{}' (special behavior not yet implemented)", def.name);
            }
            _ => {
                info!("Upgrade: Unknown type '{}' for '{}'", upgrade_type, def.name);
            }
        }
    }
}
