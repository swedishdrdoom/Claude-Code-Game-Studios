use bevy::prelude::*;

use std::collections::HashMap;

use crate::components::combat::WeaponInstance;
use crate::components::economy::*;
use crate::components::run::{GameState, RunTimer};
use crate::components::tower::{Armor, Health, PurchasedUpgrades, SpikesDamage, Tower};
use crate::plugins::content::{WeaponDatabase, UpgradeDatabase};
use crate::plugins::economy::generate_shop_inventory;

// === Marker Components ===

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct GoldPerSecText;

#[derive(Component)]
pub struct ManaShieldText;

#[derive(Component)]
pub struct StatsText;

#[derive(Component)]
pub struct DamageBonusesText;

#[derive(Component)]
pub struct EquippedListText;

#[derive(Component)]
pub struct ShopPanel;

/// A clickable shop slot. Row 0 = weapons, Row 1 = upgrades.
#[derive(Component)]
pub struct ShopSlotButton {
    pub row: usize,  // 0 = weapon, 1 = upgrade
    pub col: usize,  // 0-3
}

#[derive(Component)]
pub struct ShopSlotText {
    pub row: usize,
    pub col: usize,
}

#[derive(Component)]
pub struct RerollButton;

#[derive(Component)]
pub struct RerollText;

#[derive(Component)]
pub struct SpeedButton;

#[derive(Component)]
pub struct SpeedButtonText;

#[derive(Component)]
pub struct GameOverPanel;

#[derive(Component)]
pub struct RetryButton;

#[derive(Component)]
pub struct GameOverText;

/// Speed multiplier resource.
#[derive(Resource, Debug)]
pub struct GameSpeed {
    pub multiplier: f64,
    pub paused: bool,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self { multiplier: 1.0, paused: false }
    }
}

#[derive(Component)]
pub struct ShopTimerText;

#[derive(Component)]
pub struct TooltipPanel;

#[derive(Component)]
pub struct TooltipText;

#[derive(Component)]
pub struct StartScreenRoot;

#[derive(Component)]
pub struct StartGameButton;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSpeed>()
            .add_systems(Startup, (setup_hud, setup_shop, setup_start_screen))
            .add_systems(OnExit(GameState::MainMenu), cleanup_start_screen)
            .add_systems(Update, handle_start_button.run_if(in_state(GameState::MainMenu)))
            .add_systems(Update, (
                update_hp_text,
                update_gold_text,
                update_timer_text,
                update_gold_per_sec_text,
                update_mana_shield_text,
                update_stats_text,
                update_equipped_list,
                update_damage_bonuses_text,
                update_shop_display,
                update_shop_timer_text,
                handle_shop_clicks,
                handle_reroll_click,
                handle_speed_button,
                apply_game_speed,
                update_tooltip,
            ))
            .add_systems(OnEnter(GameState::Victory), show_game_over)
            .add_systems(OnEnter(GameState::Defeat), show_game_over)
            .add_systems(Update, handle_retry_button.run_if(
                in_state(GameState::Victory).or(in_state(GameState::Defeat)),
            ));
    }
}

fn rarity_color(rarity: &Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::srgb(0.6, 0.6, 0.6),
        Rarity::Uncommon => Color::srgb(0.2, 0.8, 0.2),
        Rarity::Rare => Color::srgb(0.3, 0.5, 1.0),
        Rarity::Epic => Color::srgb(0.7, 0.3, 0.9),
    }
}

/// Darkened rarity color for slot backgrounds.
fn rarity_bg_color(rarity: &Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::srgb(0.18, 0.18, 0.18),
        Rarity::Uncommon => Color::srgb(0.08, 0.20, 0.08),
        Rarity::Rare => Color::srgb(0.08, 0.12, 0.28),
        Rarity::Epic => Color::srgb(0.18, 0.08, 0.22),
    }
}

fn rarity_color_str(rarity: &str) -> Color {
    match rarity {
        "Common" => Color::srgb(0.6, 0.6, 0.6),
        "Uncommon" => Color::srgb(0.2, 0.8, 0.2),
        "Rare" => Color::srgb(0.3, 0.5, 1.0),
        "Epic" => Color::srgb(0.7, 0.3, 0.9),
        _ => Color::WHITE,
    }
}

// === HUD Setup ===

fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                HpText,
                Text::new("HP: 1500 / 1500"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.9, 0.3, 0.3)),
            ));
            parent.spawn((
                ManaShieldText,
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.4, 0.6, 1.0)),
            ));
            parent.spawn((
                StatsText,
                Text::new(""),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));
            parent.spawn((
                GoldText,
                Text::new("Gold: 5000"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
            ));
            parent.spawn((
                GoldPerSecText,
                Text::new("+50g/s"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.7, 0.2)),
            ));
            parent.spawn((
                TimerText,
                Text::new("0:00"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
            ));
            // Speed toggle button
            parent.spawn((
                SpeedButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
            )).with_children(|btn: &mut ChildSpawnerCommands| {
                btn.spawn((
                    SpeedButtonText,
                    Text::new("1x Speed"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
            // Equipped weapons/upgrades list
            parent.spawn((
                EquippedListText,
                Text::new(""),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));
        });

    // Damage bonuses panel — top right
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                DamageBonusesText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
            ));
        });
}

// === Shop Setup ===

fn setup_shop(mut commands: Commands) {
    // Tooltip panel — above the shop, centered
    commands.spawn((
        TooltipPanel,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(230.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(40),
    )).with_children(|parent| {
        parent.spawn((
            Node {
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.95)),
        )).with_children(|bg| {
            bg.spawn((
                TooltipText,
                Text::new(""),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
            ));
        });
    });

    // Outer container — anchored to bottom center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Percent(50.0),
                // Will be shifted left by half its width via the inner panel
                ..default()
            },
        ))
        .with_children(|outer: &mut ChildSpawnerCommands| {
            outer.spawn((
                ShopPanel,
                Node {
                    // Shift left by ~50% of panel width to center it
                    left: Val::Px(-440.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
            ))
            .with_children(|panel: &mut ChildSpawnerCommands| {
                // Title
                panel.spawn((
                    Text::new("SHOP"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                // Weapon row label
                panel.spawn((
                    Text::new("Weapons"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                // Weapon row (4 slots)
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                }).with_children(|row: &mut ChildSpawnerCommands| {
                    for col in 0..4 {
                        spawn_shop_slot(row, 0, col);
                    }
                });

                // Upgrade row label
                panel.spawn((
                    Text::new("Upgrades"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                // Upgrade row (4 slots)
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                }).with_children(|row: &mut ChildSpawnerCommands| {
                    for col in 0..4 {
                        spawn_shop_slot(row, 1, col);
                    }
                });

                // Reroll button + shop timer
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                }).with_children(|row: &mut ChildSpawnerCommands| {
                    // Reroll button
                    row.spawn((
                        RerollButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                    )).with_children(|btn: &mut ChildSpawnerCommands| {
                        btn.spawn((
                            RerollText,
                            Text::new("Reroll: 100g"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Timer
                    row.spawn((
                        ShopTimerText,
                        Text::new("30s"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            });
        });
}

fn spawn_shop_slot(parent: &mut ChildSpawnerCommands, row: usize, col: usize) {
    parent.spawn((
        ShopSlotButton { row, col },
        Button,
        Node {
            width: Val::Px(170.0),   // 130 * 1.3
            height: Val::Px(65.0),   // 50 * 1.3
            padding: UiRect::all(Val::Px(6.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
    )).with_children(|slot: &mut ChildSpawnerCommands| {
        slot.spawn((
            ShopSlotText { row, col },
            Text::new("Empty"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
        ));
    });
}

// === HUD Updates ===

fn update_hp_text(
    tower_query: Query<&Health, With<Tower>>,
    mut text_query: Query<&mut Text, With<HpText>>,
) {
    let Ok(health) = tower_query.single() else { return };
    let Ok(mut text) = text_query.single_mut() else { return };
    **text = format!("HP: {} / {}", health.current as u32, health.max as u32);
}

fn update_gold_text(
    gold: Res<Gold>,
    mut text_query: Query<&mut Text, With<GoldText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };
    **text = format!("Gold: {}", gold.current);
}

fn update_timer_text(
    timer: Res<RunTimer>,
    state: Res<State<GameState>>,
    mut text_query: Query<&mut Text, With<TimerText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };
    match state.get() {
        GameState::GracePeriod => {
            **text = format!("Get Ready: {:.0}", timer.grace_remaining);
        }
        GameState::Playing => {
            let minutes = (timer.elapsed as u32) / 60;
            let seconds = (timer.elapsed as u32) % 60;
            **text = format!("{}:{:02}", minutes, seconds);
        }
        GameState::Boss => **text = "BOSS!".to_string(),
        GameState::Victory => **text = "VICTORY!".to_string(),
        GameState::Defeat => **text = "DEFEAT!".to_string(),
        _ => {}
    }
}

fn update_gold_per_sec_text(
    gold: Res<Gold>,
    mut text_query: Query<&mut Text, With<GoldPerSecText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };
    let effective = gold.per_second * (1.0 + gold.per_second_bonus_percent);
    **text = format!("+{:.0}g/s", effective);
}

fn update_mana_shield_text(
    tower_query: Query<&crate::components::tower::ManaShield, With<Tower>>,
    mut text_query: Query<&mut Text, With<ManaShieldText>>,
) {
    let Ok(shield) = tower_query.single() else { return };
    let Ok(mut text) = text_query.single_mut() else { return };
    if shield.max > 0.0 {
        **text = format!("Shield: {} / {}", shield.current as u32, shield.max as u32);
    } else {
        **text = String::new();
    }
}

fn update_stats_text(
    tower_query: Query<(&Armor, &crate::components::tower::HpRegen, &SpikesDamage), With<Tower>>,
    gold: Res<Gold>,
    mut text_query: Query<&mut Text, With<StatsText>>,
) {
    let Ok((armor, regen, spikes)) = tower_query.single() else { return };
    let Ok(mut text) = text_query.single_mut() else { return };

    let mut stats = Vec::new();
    if armor.value > 0.0 {
        let reduction = armor.damage_reduction() * 100.0;
        stats.push(format!("Armor: {:.0} ({:.0}% reduction)", armor.value, reduction));
    }
    if regen.per_second > 0.0 {
        stats.push(format!("Regen: {:.0}/s", regen.per_second));
    }
    if spikes.flat > 0.0 {
        stats.push(format!("Spikes: {:.0}", spikes.total()));
    }
    if gold.bounty_bonus_percent > 0.0 {
        stats.push(format!("Bounty: +{:.0}%", gold.bounty_bonus_percent * 100.0));
    }

    **text = stats.join(" | ");
}

fn update_damage_bonuses_text(
    bonuses: Res<crate::components::stats::DamageBonuses>,
    mut text_query: Query<&mut Text, With<DamageBonusesText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };

    let mut lines = String::from("-- Damage Bonuses --\n");

    if bonuses.global > 0.0 { lines.push_str(&format!("All: +{:.0}%\n", bonuses.global)); }
    if bonuses.normal > 0.0 { lines.push_str(&format!("Normal: +{:.0}%\n", bonuses.normal)); }
    if bonuses.piercing > 0.0 { lines.push_str(&format!("Piercing: +{:.0}%\n", bonuses.piercing)); }
    if bonuses.magic > 0.0 { lines.push_str(&format!("Magic: +{:.0}%\n", bonuses.magic)); }
    if bonuses.siege > 0.0 { lines.push_str(&format!("Siege: +{:.0}%\n", bonuses.siege)); }
    if bonuses.chaos > 0.0 { lines.push_str(&format!("Chaos: +{:.0}%\n", bonuses.chaos)); }
    if bonuses.attack_speed > 0.0 { lines.push_str(&format!("Atk Speed: +{:.0}%\n", bonuses.attack_speed)); }

    if lines == "-- Damage Bonuses --\n" {
        **text = String::new(); // Hide panel when no bonuses
    } else {
        **text = lines;
    }
}

fn update_equipped_list(
    weapon_query: Query<&WeaponInstance>,
    upgrades: Res<PurchasedUpgrades>,
    mut text_query: Query<&mut Text, With<EquippedListText>>,
) {
    let Ok(mut text) = text_query.single_mut() else { return };

    let mut lines = String::new();

    // Count weapons by name
    let mut weapon_counts: HashMap<String, u32> = HashMap::new();
    let mut weapon_order: Vec<String> = Vec::new();
    for weapon in &weapon_query {
        let count = weapon_counts.entry(weapon.name.clone()).or_insert(0);
        if *count == 0 {
            weapon_order.push(weapon.name.clone());
        }
        *count += 1;
    }

    if !weapon_order.is_empty() {
        lines.push_str("-- Weapons --\n");
        for name in &weapon_order {
            let count = weapon_counts[name];
            if count > 1 {
                lines.push_str(&format!("{}x {}\n", count, name));
            } else {
                lines.push_str(&format!("{}\n", name));
            }
        }
    }

    // Count upgrades by name
    if !upgrades.items.is_empty() {
        let mut upgrade_counts: HashMap<String, u32> = HashMap::new();
        let mut upgrade_order: Vec<String> = Vec::new();
        for name in &upgrades.items {
            let count = upgrade_counts.entry(name.clone()).or_insert(0);
            if *count == 0 {
                upgrade_order.push(name.clone());
            }
            *count += 1;
        }

        lines.push_str("-- Upgrades --\n");
        for name in &upgrade_order {
            let count = upgrade_counts[name];
            if count > 1 {
                lines.push_str(&format!("{}x {}\n", count, name));
            } else {
                lines.push_str(&format!("{}\n", name));
            }
        }
    }

    if lines.is_empty() {
        lines = "No items".to_string();
    }

    **text = lines;
}

// === Shop Updates ===

fn update_shop_display(
    shop: Res<ShopState>,
    gold: Res<Gold>,
    mut slot_text_query: Query<(&mut Text, &mut TextColor, &ShopSlotText)>,
    mut slot_bg_query: Query<(&mut BackgroundColor, &ShopSlotButton), Without<ShopSlotText>>,
) {
    // Update slot text and colors
    for (mut text, mut text_color, slot_info) in &mut slot_text_query {
        let item = if slot_info.row == 0 {
            &shop.weapon_slots[slot_info.col]
        } else {
            &shop.upgrade_slots[slot_info.col]
        };

        match item {
            Some(shop_item) => {
                let price = shop_item.price();
                let affordable = gold.current >= price;
                if affordable {
                    **text = format!("{}\n{}g", shop_item.name, price);
                    *text_color = TextColor(rarity_color(&shop_item.rarity));
                } else {
                    **text = format!("{}\n{}g", shop_item.name, price);
                    *text_color = TextColor(Color::srgb(0.4, 0.4, 0.4));
                };
            }
            None => {
                **text = "SOLD".to_string();
                *text_color = TextColor(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }

    // Update slot backgrounds — tinted by rarity
    for (mut bg, slot_btn) in &mut slot_bg_query {
        let item = if slot_btn.row == 0 {
            &shop.weapon_slots[slot_btn.col]
        } else {
            &shop.upgrade_slots[slot_btn.col]
        };

        match item {
            Some(shop_item) => {
                let affordable = gold.current >= shop_item.price();
                *bg = if affordable {
                    BackgroundColor(rarity_bg_color(&shop_item.rarity))
                } else {
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12))
                };
            }
            None => {
                *bg = BackgroundColor(Color::srgb(0.08, 0.08, 0.08));
            }
        }
    }
}

fn update_shop_timer_text(
    shop: Res<ShopState>,
    state: Res<State<GameState>>,
    mut text_query: Query<&mut Text, With<ShopTimerText>>,
    mut reroll_text_query: Query<&mut Text, (With<RerollText>, Without<ShopTimerText>)>,
    mut reroll_bg: Query<&mut BackgroundColor, With<RerollButton>>,
    gold: Res<Gold>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        **text = format!("{:.0}s", shop.refresh_timer);
    }

    let is_grace = *state.get() == GameState::GracePeriod;
    let free_rerolls_left = if is_grace { 5u32.saturating_sub(shop.rerolls_this_cycle) } else { 0 };

    let cost = shop.reroll_cost();
    let can_reroll = if free_rerolls_left > 0 {
        true
    } else {
        gold.current >= cost && shop.rerolls_this_cycle < MAX_REROLLS_PER_CYCLE
    };

    if let Ok(mut text) = reroll_text_query.single_mut() {
        if free_rerolls_left > 0 {
            **text = format!("Reroll: FREE ({} left)", free_rerolls_left);
        } else {
            **text = format!("Reroll: {}g ({}/{})", cost, shop.rerolls_this_cycle, MAX_REROLLS_PER_CYCLE);
        }
    }

    if let Ok(mut bg) = reroll_bg.single_mut() {
        *bg = if can_reroll {
            BackgroundColor(Color::srgb(0.3, 0.3, 0.5))
        } else {
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2))
        };
    }
}

// === Tooltip ===

fn update_tooltip(
    slot_query: Query<(&Interaction, &ShopSlotButton)>,
    shop: Res<ShopState>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
    mut tooltip_vis: Query<&mut Visibility, With<TooltipPanel>>,
    mut tooltip_text: Query<(&mut Text, &mut TextColor), With<TooltipText>>,
) {
    let Ok(mut vis) = tooltip_vis.single_mut() else { return };
    let Ok((mut text, mut text_color)) = tooltip_text.single_mut() else { return };

    // Find hovered slot
    let mut hovered_item = None;
    for (interaction, slot) in &slot_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let item = if slot.row == 0 {
                &shop.weapon_slots[slot.col]
            } else {
                &shop.upgrade_slots[slot.col]
            };
            if let Some(shop_item) = item {
                hovered_item = Some((slot.row, shop_item.clone()));
            }
            break;
        }
    }

    let Some((row, item)) = hovered_item else {
        *vis = Visibility::Hidden;
        return;
    };

    *vis = Visibility::Visible;
    *text_color = TextColor(rarity_color(&item.rarity));

    if row == 0 {
        // Weapon tooltip
        let def = &weapons.weapons[item.definition_index];
        let mut lines = vec![
            format!("{} [{}]", def.name, def.rarity),
            format!("{} | {}", def.weapon_type, def.attack_type),
            format!("{}dmg  {:.1}s cd  {} DPS  Range {}", def.damage, def.attack_cooldown, def.dps, def.range),
        ];
        let mods = extract_modifiers(&def.ability);
        if !mods.is_empty() {
            lines.push(mods);
        }
        if !def.ability.is_empty() {
            lines.push(def.ability.clone());
        }
        lines.push(format!("Cost: {}g", item.price()));
        **text = lines.join("\n");
    } else {
        // Upgrade tooltip
        let def = &upgrades.upgrades[item.definition_index];
        **text = format!(
            "{} [{}]\nType: {}\n{}\nCost: {}g",
            def.name, def.rarity, def.upgrade_type, def.description, item.price()
        );
    }
}

/// Extract short modifier tags from a weapon ability string.
fn extract_modifiers(ability: &str) -> String {
    let mut mods = Vec::new();
    let lower = ability.to_lowercase();
    if lower.contains("frost") || lower.contains("freeze") { mods.push("Frost"); }
    if lower.contains("fire") || lower.contains("burn") || lower.contains("ignite") { mods.push("Fire"); }
    if lower.contains("poison") { mods.push("Poison"); }
    if lower.contains("heal") { mods.push("Heal"); }
    if lower.contains("mana shield") { mods.push("Mana Shield"); }
    if lower.contains("max hp") { mods.push("+Max HP"); }
    if lower.contains("stun") { mods.push("Stun"); }
    if mods.is_empty() { return String::new(); }
    format!("Mods: {}", mods.join(", "))
}

// === Shop Interaction ===

fn handle_shop_clicks(
    interaction_query: Query<(&Interaction, &ShopSlotButton), Changed<Interaction>>,
    mut shop: ResMut<ShopState>,
    mut gold: ResMut<Gold>,
    mut purchased_upgrades: ResMut<PurchasedUpgrades>,
    mut weapon_events: MessageWriter<WeaponPurchasedEvent>,
    mut upgrade_events: MessageWriter<UpgradePurchasedEvent>,
    timer: Res<RunTimer>,
) {
    // Lock shop at 15 minutes (boss phase)
    if timer.elapsed >= crate::components::run::RUN_DURATION {
        return;
    }

    for (interaction, slot) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let item = if slot.row == 0 {
            shop.weapon_slots[slot.col].clone()
        } else {
            shop.upgrade_slots[slot.col].clone()
        };

        let Some(shop_item) = item else {
            continue; // Empty slot
        };

        let price = shop_item.price();
        if gold.current < price {
            continue; // Can't afford
        }

        // Deduct gold
        gold.current -= price;
        gold.total_spent += price;

        // Send purchase event
        match shop_item.item_type {
            ShopItemType::Weapon => {
                weapon_events.write(WeaponPurchasedEvent {
                    weapon_name: shop_item.name.clone(),
                    definition_index: shop_item.definition_index,
                });
            }
            ShopItemType::Upgrade => {
                purchased_upgrades.items.push(shop_item.name.clone());
                upgrade_events.write(UpgradePurchasedEvent {
                    upgrade_name: shop_item.name.clone(),
                    definition_index: shop_item.definition_index,
                });
            }
        }

        info!("Purchased {} for {} gold", shop_item.name, price);

        // Clear the slot
        if slot.row == 0 {
            shop.weapon_slots[slot.col] = None;
        } else {
            shop.upgrade_slots[slot.col] = None;
        }
    }
}

fn handle_reroll_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RerollButton>)>,
    mut shop: ResMut<ShopState>,
    mut gold: ResMut<Gold>,
    state: Res<State<GameState>>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
    timer: Res<RunTimer>,
) {
    if timer.elapsed >= crate::components::run::RUN_DURATION { return; }

    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let is_grace = *state.get() == GameState::GracePeriod;
        let free_rerolls_left = if is_grace { 5u32.saturating_sub(shop.rerolls_this_cycle) } else { 0 };
        let is_free = free_rerolls_left > 0;

        let cost = if is_free { 0 } else { shop.reroll_cost() };

        if !is_free && (gold.current < cost || shop.rerolls_this_cycle >= MAX_REROLLS_PER_CYCLE) {
            continue;
        }
        if is_free && shop.rerolls_this_cycle >= 5 {
            // Used all free rerolls during grace, switch to paid
            let paid_cost = shop.reroll_cost();
            if gold.current < paid_cost || shop.rerolls_this_cycle >= MAX_REROLLS_PER_CYCLE {
                continue;
            }
            gold.current -= paid_cost;
            gold.total_spent += paid_cost;
        } else if !is_free {
            gold.current -= cost;
            gold.total_spent += cost;
        }

        if !is_free {
            shop.total_rerolls_this_run += 1;
        }
        shop.rerolls_this_cycle += 1;

        generate_shop_inventory(&mut shop, &weapons, &upgrades);
        if is_free {
            info!("Free reroll during grace period ({}/5 used)", shop.rerolls_this_cycle);
        } else {
            info!("Rerolled shop for {} gold (reroll #{})", cost, shop.total_rerolls_this_run);
        }
    }
}

// === Speed Control ===

fn handle_speed_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SpeedButton>)>,
    mut speed: ResMut<GameSpeed>,
    mut text_query: Query<&mut Text, With<SpeedButtonText>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Toggle 1x → 2x → 1x
        speed.multiplier = if speed.multiplier < 1.5 { 2.0 } else { 1.0 };
        if let Ok(mut text) = text_query.single_mut() {
            **text = format!("{}x Speed", speed.multiplier as u32);
        }
        info!("Game speed set to {}x", speed.multiplier);
    }
}

fn apply_game_speed(
    speed: Res<GameSpeed>,
    mut time: ResMut<Time<Virtual>>,
) {
    if speed.paused {
        time.set_relative_speed(0.0);
    } else {
        time.set_relative_speed(speed.multiplier as f32);
    }
}

// === Game Over / Retry ===

fn show_game_over(
    mut commands: Commands,
    state: Res<State<GameState>>,
    timer: Res<RunTimer>,
    gold: Res<Gold>,
) {
    let is_victory = *state.get() == GameState::Victory;
    let title = if is_victory { "VICTORY!" } else { "DEFEAT" };
    let title_color = if is_victory {
        Color::srgb(0.2, 1.0, 0.3)
    } else {
        Color::srgb(1.0, 0.2, 0.2)
    };

    let minutes = (timer.elapsed as u32) / 60;
    let seconds = (timer.elapsed as u32) % 60;

    commands.spawn((
        GameOverPanel,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent: &mut ChildSpawnerCommands| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        )).with_children(|panel: &mut ChildSpawnerCommands| {
            // Title
            panel.spawn((
                GameOverText,
                Text::new(title),
                TextFont { font_size: 40.0, ..default() },
                TextColor(title_color),
            ));
            // Stats
            panel.spawn((
                Text::new(format!("Time: {}:{:02}", minutes, seconds)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
            ));
            panel.spawn((
                Text::new(format!("Gold earned: {}", gold.total_earned)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
            ));
            // Retry button
            panel.spawn((
                RetryButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.5, 0.2)),
            )).with_children(|btn: &mut ChildSpawnerCommands| {
                btn.spawn((
                    Text::new("RETRY"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn handle_retry_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    game_over_query: Query<Entity, With<GameOverPanel>>,
    enemy_query: Query<Entity, With<crate::components::enemy::Enemy>>,
    projectile_query: Query<Entity, With<crate::components::combat::Projectile>>,
    weapon_query: Query<Entity, With<WeaponInstance>>,
    mut gold: ResMut<Gold>,
    mut shop: ResMut<ShopState>,
    mut scaling_buffs: ResMut<crate::components::scaling::ScalingBuffs>,
    mut damage_bonuses: ResMut<crate::components::stats::DamageBonuses>,
    mut timer: ResMut<RunTimer>,
    mut purchased_upgrades: ResMut<PurchasedUpgrades>,
    mut tower_query: Query<(
        &mut Health, &mut Armor, &mut crate::components::tower::ManaShield,
        &mut crate::components::tower::HpRegen, &mut crate::components::tower::FlatDamageReduction,
        &mut SpikesDamage, &mut crate::components::tower::ManaShieldRegen,
        &mut crate::components::tower::ManaShieldOnKill, &mut crate::components::tower::ManaShieldOnHit,
        &mut crate::components::tower::HealOnKill, &mut crate::components::tower::HealOnAttacked,
        &mut crate::components::tower::HealPerHit, &mut crate::components::tower::MaxHpPerHit,
    ), With<Tower>>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Despawn game over panel
        for entity in &game_over_query {
            commands.entity(entity).despawn();
        }
        // Despawn all enemies
        for entity in &enemy_query {
            commands.entity(entity).despawn();
        }
        // Despawn all projectiles
        for entity in &projectile_query {
            commands.entity(entity).despawn();
        }
        // Despawn all weapons
        for entity in &weapon_query {
            commands.entity(entity).despawn();
        }
        // Reset gold
        *gold = Gold::default();
        // Reset shop
        *shop = ShopState::default();
        // Reset timer
        *timer = RunTimer::default();
        // Reset purchased upgrades
        purchased_upgrades.items.clear();
        scaling_buffs.buffs.clear();
        *damage_bonuses = crate::components::stats::DamageBonuses::default();
        // Reset tower stats
        if let Ok((mut health, mut armor, mut shield, mut regen, mut flat_red, mut spikes, mut shield_regen, mut shield_on_kill, mut shield_on_hit, mut hp_on_kill, mut hp_on_attacked, mut heal_per_hit, mut maxhp_per_hit)) = tower_query.single_mut() {
            *health = Health::default();
            *armor = Armor::default();
            *shield = crate::components::tower::ManaShield::default();
            *regen = crate::components::tower::HpRegen::default();
            *flat_red = crate::components::tower::FlatDamageReduction::default();
            *spikes = SpikesDamage::default();
            *shield_regen = crate::components::tower::ManaShieldRegen::default();
            *shield_on_kill = crate::components::tower::ManaShieldOnKill::default();
            *shield_on_hit = crate::components::tower::ManaShieldOnHit::default();
            *hp_on_kill = crate::components::tower::HealOnKill::default();
            *hp_on_attacked = crate::components::tower::HealOnAttacked::default();
            *heal_per_hit = crate::components::tower::HealPerHit::default();
            *maxhp_per_hit = crate::components::tower::MaxHpPerHit::default();
        }

        next_state.set(GameState::GracePeriod);
        info!("Retrying — new run!");
    }
}

// === Start Screen ===

fn setup_start_screen(mut commands: Commands) {
    info!("Setting up start screen");
    commands.spawn((
        StartScreenRoot,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.15, 0.98)),
        GlobalZIndex(200),
    )).with_children(|root| {
        // Center column
        root.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(30.0),
            max_width: Val::Px(700.0),
            ..default()
        }).with_children(|col| {
            // Title
            col.spawn((
                Text::new("Tower of Doom"),
                TextFont { font_size: 42.0, ..default() },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            // 3-card row
            col.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                ..default()
            }).with_children(|row| {
                let cards = [
                    ("1", "Buy weapons and upgrades for your tower"),
                    ("2", "Don't forget to invest in gold upgrades"),
                    ("3", "Survive as long as possible"),
                ];
                for (num, text) in cards {
                    row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(16.0)),
                            width: Val::Px(200.0),
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.25, 0.9)),
                    )).with_children(|card| {
                        // Number circle
                        card.spawn((
                            Node {
                                width: Val::Px(36.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.8, 0.7, 0.2)),
                        )).with_children(|circle| {
                            circle.spawn((
                                Text::new(num),
                                TextFont { font_size: 22.0, ..default() },
                                TextColor(Color::srgb(0.05, 0.05, 0.1)),
                            ));
                        });
                        // Description
                        card.spawn((
                            Text::new(text),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::srgb(0.8, 0.8, 0.9)),
                        ));
                    });
                }
            });

            // Tip text
            col.spawn((
                Node {
                    max_width: Val::Px(600.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.8)),
            )).with_children(|tip| {
                tip.spawn((
                    Text::new("You will have a 30 second preparation time to reroll your initial shop, and buy upgrades before the horde appears"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.7, 0.8)),
                ));
            });

            // Start Game button
            col.spawn((
                StartGameButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(14.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("Start Game"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn cleanup_start_screen(
    mut commands: Commands,
    query: Query<Entity, With<StartScreenRoot>>,
) {
    info!("Cleaning up start screen");
    for entity in &query {
        commands.entity(entity).despawn();
    }
}


fn handle_start_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::GracePeriod);
            info!("Starting game!");
        }
    }
}
