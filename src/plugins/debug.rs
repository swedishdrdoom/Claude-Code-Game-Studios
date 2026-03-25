use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

use crate::components::combat::{Projectile, WeaponInstance};
use crate::components::economy::*;
use crate::components::enemy::*;
use crate::components::run::GameState;
use crate::components::scaling::ScalingBuffs;
use crate::components::stats::DamageBonuses;
use crate::components::tower::*;
use crate::plugins::content::{UpgradeDatabase, WeaponDatabase};
use crate::plugins::core::ArenaConfig;
use crate::plugins::enemy::WaveState;
use crate::plugins::ui::GameSpeed;

const ITEMS_PER_PAGE: usize = 20;

// === Resources ===

#[derive(Resource)]
pub struct DebugState {
    pub active: bool,
    pub browser_open: BrowserMode,
    pub browser_page: usize,
    pub difficulty_pct: i32,
    pub show_setup: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            active: false,
            browser_open: BrowserMode::None,
            browser_page: 0,
            difficulty_pct: 0,
            show_setup: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurveType {
    #[default]
    Linear,
    Exponential,
}

#[derive(Resource, Debug, Clone)]
pub struct DifficultyConfig {
    pub curve: CurveType,
    pub hp_rate: f32,
    pub dmg_rate: f32,
    pub spawn_rate: f32,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self::exponential_defaults()
    }
}

impl DifficultyConfig {
    fn linear_defaults() -> Self {
        Self { curve: CurveType::Linear, hp_rate: 0.010, dmg_rate: 0.003, spawn_rate: 0.005 }
    }
    fn exponential_defaults() -> Self {
        Self { curve: CurveType::Exponential, hp_rate: 1.0101, dmg_rate: 1.0051, spawn_rate: 1.0025 }
    }
    fn hp_at(&self, t: f32) -> f32 {
        match self.curve {
            CurveType::Linear => 1.0 + self.hp_rate * t,
            CurveType::Exponential => self.hp_rate.powf(t),
        }
    }
    fn dmg_at(&self, t: f32) -> f32 {
        match self.curve {
            CurveType::Linear => 1.0 + self.dmg_rate * t,
            CurveType::Exponential => self.dmg_rate.powf(t),
        }
    }
    fn spawn_at(&self, t: f32) -> f32 {
        match self.curve {
            CurveType::Linear => 1.0 + self.spawn_rate * t,
            CurveType::Exponential => self.spawn_rate.powf(t),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum BrowserMode {
    #[default]
    None,
    Weapons,
    Upgrades,
}

// === Components ===

#[derive(Component)]
struct DebugHelpPanel;
#[derive(Component)]
struct DebugModeIndicator;
#[derive(Component)]
struct DifficultyIndicator;
#[derive(Component)]
struct BrowserPanel;
#[derive(Component)]
struct BrowserItemButton { index: usize, is_weapon: bool }
#[derive(Component)]
struct BrowserPageButton { delta: i32 }
#[derive(Component)]
struct SetupPanel;
#[derive(Component)]
struct SetupButton(SetupAction);
#[derive(Component)]
struct SetupText(u8);
#[derive(Component)]
struct SetupCurveText;
#[derive(Component)]
struct SetupPreviewText;

#[derive(Debug, Clone, Copy, PartialEq)]
enum SetupAction {
    ToggleCurve,
    AdjustRate { param: u8, positive: bool },
}

// === Plugin ===

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .init_resource::<DifficultyConfig>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(OnEnter(GameState::GracePeriod), on_enter_grace_period)
            .add_systems(Update, (
                toggle_debug_mode,
                handle_debug_actions,
                handle_debug_extras,
                handle_debug_restart,
                handle_browser_toggle,
                handle_browser_navigation,
                handle_browser_clicks,
                update_debug_indicator,
                manage_setup_panel,
                handle_setup_clicks,
                update_setup_texts,
            ))
            .add_systems(PostUpdate, apply_difficulty_modifier);
    }
}

// === Startup ===

fn setup_debug_ui(mut commands: Commands) {
    commands.spawn((
        DebugModeIndicator,
        Text::new(""),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 0.3, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(2.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));

    commands.spawn((
        DebugHelpPanel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(160.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(
                "-- DEBUG CONTROLS --\n\
                 ` : Toggle debug mode\n\
                 1 : This help panel\n\
                 2 : Pause / unpause\n\
                 3 : Spawn 20 enemies\n\
                 4 : Clear all enemies\n\
                 5 : Grant +10,000 gold\n\
                 6 : Weapon browser\n\
                 7 : Upgrade browser\n\
                 8 : Restart run\n\
                 9 : Start wave (skip grace)\n\
                 0 : Clear all upgrades\n\
                 +/- : Difficulty % tweak\n\
                 Tab : Difficulty setup\n\
                 Esc : Close browser"
            ),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.9, 0.7, 0.3)),
        ));
    });

    commands.spawn((
        DifficultyIndicator,
        Text::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(1.0, 0.6, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(18.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

fn on_enter_grace_period(mut dbg: ResMut<DebugState>) {
    dbg.show_setup = true;
}

// === Debug Mode Toggle ===

fn toggle_debug_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut dbg: ResMut<DebugState>,
    mut commands: Commands,
    browser_query: Query<Entity, With<BrowserPanel>>,
    setup_query: Query<Entity, With<SetupPanel>>,
) {
    if keys.just_pressed(KeyCode::Backquote) {
        dbg.active = !dbg.active;
        if !dbg.active {
            for entity in &browser_query { commands.entity(entity).despawn(); }
            for entity in &setup_query { commands.entity(entity).despawn(); }
            dbg.browser_open = BrowserMode::None;
            dbg.browser_page = 0;
        }
        let state = if dbg.active { "ON" } else { "OFF" };
        info!("[DEBUG] Debug mode {}", state);
    }
}

fn update_debug_indicator(
    dbg: Res<DebugState>,
    speed: Res<GameSpeed>,
    mut mode_text: Query<&mut Text, (With<DebugModeIndicator>, Without<DifficultyIndicator>)>,
    mut diff_text: Query<&mut Text, (With<DifficultyIndicator>, Without<DebugModeIndicator>)>,
) {
    if let Ok(mut text) = mode_text.single_mut() {
        if dbg.active {
            let pause_str = if speed.paused { " [PAUSED]" } else { "" };
            **text = format!("DEBUG MODE{}", pause_str);
        } else {
            **text = String::new();
        }
    }
    if let Ok(mut text) = diff_text.single_mut() {
        if dbg.active && dbg.difficulty_pct != 0 {
            let sign = if dbg.difficulty_pct > 0 { "+" } else { "" };
            **text = format!("Difficulty: {}{}% HP & Dmg", sign, dbg.difficulty_pct);
        } else {
            **text = String::new();
        }
    }
}

// === Debug Actions ===

fn handle_debug_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mut dbg: ResMut<DebugState>,
    mut speed: ResMut<GameSpeed>,
    mut gold: ResMut<Gold>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut help_panel: Query<&mut Visibility, With<DebugHelpPanel>>,
    enemy_query: Query<Entity, With<Enemy>>,
    arena: Res<ArenaConfig>,
    wave: Res<WaveState>,
    state: Res<State<GameState>>,
    run_timer: Res<crate::components::run::RunTimer>,
) {
    if !dbg.active { return; }

    if keys.just_pressed(KeyCode::Digit1) {
        if let Ok(mut vis) = help_panel.single_mut() {
            *vis = match *vis {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
    if keys.just_pressed(KeyCode::Digit2) {
        speed.paused = !speed.paused;
        let s = if speed.paused { "Paused" } else { "Unpaused" };
        info!("[DEBUG] {}", s);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        spawn_debug_enemies(&mut commands, &arena, &wave, &dbg, run_timer.elapsed);
        info!("[DEBUG] Spawned 20 enemies");
    }
    if keys.just_pressed(KeyCode::Digit4) {
        let mut count = 0u32;
        for entity in &enemy_query { commands.entity(entity).despawn(); count += 1; }
        info!("[DEBUG] Cleared {} enemies", count);
    }
    if keys.just_pressed(KeyCode::Digit5) {
        gold.current += 10_000;
        gold.total_earned += 10_000;
        info!("[DEBUG] Granted 10,000 gold (now {})", gold.current);
    }
    if keys.just_pressed(KeyCode::Digit9) {
        if *state.get() == GameState::GracePeriod {
            dbg.show_setup = false;
            next_state.set(GameState::Playing);
            info!("[DEBUG] Skipped grace period — wave started");
        }
    }
    if keys.just_pressed(KeyCode::Equal) {
        dbg.difficulty_pct = (dbg.difficulty_pct + 25).min(500);
        info!("[DEBUG] Difficulty: +{}%", dbg.difficulty_pct);
    }
    if keys.just_pressed(KeyCode::Minus) {
        dbg.difficulty_pct = (dbg.difficulty_pct - 25).max(-75);
        info!("[DEBUG] Difficulty: {}%", dbg.difficulty_pct);
    }
    if keys.just_pressed(KeyCode::Tab) {
        dbg.show_setup = !dbg.show_setup;
    }
}

// === Clear Upgrades (key 0) ===

fn handle_debug_extras(
    keys: Res<ButtonInput<KeyCode>>,
    dbg: Res<DebugState>,
    mut commands: Commands,
    mut purchased_upgrades: ResMut<PurchasedUpgrades>,
    mut scaling_buffs: ResMut<ScalingBuffs>,
    mut damage_bonuses: ResMut<DamageBonuses>,
    mut gold: ResMut<Gold>,
    weapon_query: Query<(Entity, &WeaponInstance)>,
    mut weapon_events: MessageWriter<WeaponPurchasedEvent>,
    mut tower_query: Query<(
        &mut Health, &mut Armor, &mut ManaShield,
        &mut HpRegen, &mut FlatDamageReduction,
        &mut SpikesDamage, &mut ManaShieldRegen,
        &mut ManaShieldOnKill, &mut ManaShieldOnHit,
        &mut HealOnKill, &mut HealOnAttacked,
        &mut HealPerHit, &mut MaxHpPerHit,
    ), With<Tower>>,
) {
    if !dbg.active || !keys.just_pressed(KeyCode::Digit0) { return; }

    let weapon_indices: Vec<usize> = weapon_query.iter().map(|(_, w)| w.definition_index).collect();
    for (entity, _) in &weapon_query { commands.entity(entity).despawn(); }

    if let Ok((
        mut health, mut armor, mut shield, mut regen, mut flat_red,
        mut spikes, mut shield_regen, mut shield_on_kill, mut shield_on_hit,
        mut hp_on_kill, mut hp_on_attacked, mut heal_per_hit, mut maxhp_per_hit,
    )) = tower_query.single_mut() {
        *health = Health::default(); *armor = Armor::default();
        *shield = ManaShield::default(); *regen = HpRegen::default();
        *flat_red = FlatDamageReduction::default(); *spikes = SpikesDamage::default();
        *shield_regen = ManaShieldRegen::default(); *shield_on_kill = ManaShieldOnKill::default();
        *shield_on_hit = ManaShieldOnHit::default(); *hp_on_kill = HealOnKill::default();
        *hp_on_attacked = HealOnAttacked::default(); *heal_per_hit = HealPerHit::default();
        *maxhp_per_hit = MaxHpPerHit::default();
    }

    purchased_upgrades.items.clear();
    scaling_buffs.buffs.clear();
    *damage_bonuses = DamageBonuses::default();
    gold.per_second = crate::components::economy::BASE_GOLD_PER_SECOND;
    gold.per_second_bonus_percent = 0.0;
    gold.bounty_bonus_percent = 0.0;

    let weapon_count = weapon_indices.len();
    for def_idx in weapon_indices {
        weapon_events.write(WeaponPurchasedEvent {
            weapon_name: String::new(),
            definition_index: def_idx,
        });
    }
    info!("[DEBUG] Cleared all upgrades, respawned {} weapons with original stats", weapon_count);
}

// === Difficulty Override (PostUpdate) ===

fn apply_difficulty_modifier(
    _dbg: Res<DebugState>,
    _cfg: Res<DifficultyConfig>,
    _timer: Res<crate::components::run::RunTimer>,
    _wave: ResMut<WaveState>,
) {
    // Disabled — using base piecewise curves from enemy plugin
}

// === Restart (key 8) ===

fn handle_debug_restart(
    keys: Res<ButtonInput<KeyCode>>,
    mut dbg: ResMut<DebugState>,
    mut speed: ResMut<GameSpeed>,
    mut gold: ResMut<Gold>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut shop: ResMut<ShopState>,
    mut scaling_buffs: ResMut<ScalingBuffs>,
    mut damage_bonuses: ResMut<DamageBonuses>,
    mut timer: ResMut<crate::components::run::RunTimer>,
    mut purchased_upgrades: ResMut<PurchasedUpgrades>,
    mut wave_state: ResMut<WaveState>,
    cleanup_query: Query<Entity, Or<(With<Enemy>, With<Projectile>, With<WeaponInstance>)>>,
    ui_cleanup: Query<Entity, Or<(With<crate::plugins::ui::GameOverPanel>, With<BrowserPanel>, With<SetupPanel>)>>,
    mut tower_query: Query<(
        &mut Health, &mut Armor, &mut ManaShield,
        &mut HpRegen, &mut FlatDamageReduction,
        &mut SpikesDamage, &mut ManaShieldRegen,
        &mut ManaShieldOnKill, &mut ManaShieldOnHit,
        &mut HealOnKill, &mut HealOnAttacked,
        &mut HealPerHit, &mut MaxHpPerHit,
    ), With<Tower>>,
) {
    if !dbg.active || !keys.just_pressed(KeyCode::Digit8) { return; }

    dbg.browser_open = BrowserMode::None;
    dbg.browser_page = 0;
    dbg.show_setup = true;

    for entity in &ui_cleanup { commands.entity(entity).despawn(); }
    for entity in &cleanup_query { commands.entity(entity).despawn(); }

    *gold = Gold::default();
    *shop = ShopState::default();
    *timer = crate::components::run::RunTimer::default();
    purchased_upgrades.items.clear();
    scaling_buffs.buffs.clear();
    *damage_bonuses = DamageBonuses::default();
    *wave_state = WaveState::default();
    speed.paused = false;

    if let Ok((
        mut health, mut armor, mut shield, mut regen, mut flat_red,
        mut spikes, mut shield_regen, mut shield_on_kill, mut shield_on_hit,
        mut hp_on_kill, mut hp_on_attacked, mut heal_per_hit, mut maxhp_per_hit,
    )) = tower_query.single_mut() {
        *health = Health::default(); *armor = Armor::default();
        *shield = ManaShield::default(); *regen = HpRegen::default();
        *flat_red = FlatDamageReduction::default(); *spikes = SpikesDamage::default();
        *shield_regen = ManaShieldRegen::default(); *shield_on_kill = ManaShieldOnKill::default();
        *shield_on_hit = ManaShieldOnHit::default(); *hp_on_kill = HealOnKill::default();
        *hp_on_attacked = HealOnAttacked::default(); *heal_per_hit = HealPerHit::default();
        *maxhp_per_hit = MaxHpPerHit::default();
    }

    next_state.set(GameState::GracePeriod);
    info!("[DEBUG] Run restarted");
}

// === Enemy Spawning ===

fn spawn_debug_enemies(commands: &mut Commands, arena: &ArenaConfig, wave: &WaveState, dbg: &DebugState, elapsed: f32) {
    let diff_scale = 1.0 + (dbg.difficulty_pct as f32 / 100.0);
    let mut rng = rand::rng();

    for _ in 0..20 {
        let angle = rng.random_range(0.0..TAU);
        let x = angle.cos() * arena.spawn_ring_radius;
        let y = angle.sin() * arena.spawn_ring_radius;

        let armor_type = match rng.random_range(0..5) {
            0 => ArmorType::Light, 1 => ArmorType::Medium,
            2 => ArmorType::Heavy, 3 => ArmorType::Fortified,
            _ => ArmorType::Unarmored,
        };

        let (base_hp, base_speed, base_damage, color, enemy_size) = match armor_type {
            ArmorType::Light     => (300.0, 150.0, 5.0, Color::srgb(0.9, 0.9, 0.3), 21.0),
            ArmorType::Medium    => (300.0, 100.0, 10.0, Color::srgb(0.3, 0.8, 0.3), 24.0),
            ArmorType::Heavy     => (300.0, 60.0, 20.0, Color::srgb(0.3, 0.3, 0.9), 33.0),
            ArmorType::Fortified => (300.0, 50.0, 30.0, Color::srgb(0.7, 0.3, 0.3), 36.0),
            ArmorType::Unarmored => (300.0, 120.0, 3.0, Color::srgb(0.8, 0.8, 0.8), 18.0),
            ArmorType::Hero      => (10000.0, 80.0, 50.0, Color::srgb(0.9, 0.1, 0.9), 60.0),
        };

        let hp = base_hp * wave.hp_multiplier * diff_scale;
        let damage = base_damage * wave.damage_multiplier * diff_scale;

        commands.spawn((
            Enemy, armor_type,
            EnemyHealth { current: hp, max: hp },
            EnemyArmor::default(),
            MoveSpeed { base: base_speed, multiplier: 1.0 },
            GoldBounty::default(),
            EnemyAttack { damage, cooldown: 1.0, timer: 0.0, range: 30.0 },
            FrostStacks::default(), Burning::default(),
            Sprite { color, custom_size: Some(Vec2::splat(enemy_size)), ..default() },
            Transform::from_xyz(x, y, 0.5),
        )).with_children(|parent| {
            parent.spawn((
                EnemyHealthBarBg,
                Sprite { color: Color::srgba(0.2, 0.2, 0.2, 0.8), custom_size: Some(Vec2::new(enemy_size + 4.0, 3.0)), ..default() },
                Transform::from_xyz(0.0, enemy_size * 0.5 + 4.0, 0.1),
            ));
            let bar_width = enemy_size + 2.0;
            parent.spawn((
                EnemyHealthBarFill { full_width: bar_width },
                Sprite { color: Color::srgb(0.1, 0.9, 0.1), custom_size: Some(Vec2::new(bar_width, 2.0)), ..default() },
                Transform::from_xyz(0.0, enemy_size * 0.5 + 4.0, 0.2),
            ));
        });
    }
}

// === Setup Panel ===

fn manage_setup_panel(
    dbg: Res<DebugState>,
    cfg: Res<DifficultyConfig>,
    setup_query: Query<Entity, With<SetupPanel>>,
    mut commands: Commands,
) {
    let panel_exists = !setup_query.is_empty();
    let should_show = dbg.show_setup && dbg.active;

    if should_show && !panel_exists {
        spawn_setup_panel(&mut commands, &cfg);
    } else if !should_show && panel_exists {
        for entity in &setup_query { commands.entity(entity).despawn(); }
    }
}

fn spawn_setup_panel(commands: &mut Commands, cfg: &DifficultyConfig) {
    commands.spawn((
        SetupPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Percent(15.0),
            width: Val::Px(460.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(14.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.1, 0.92)),
        GlobalZIndex(48),
    )).with_children(|panel| {
        panel.spawn((
            Text::new("DIFFICULTY SETUP — Tab to close"),
            TextFont { font_size: 15.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
            Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
        ));

        // Curve toggle
        panel.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        }).with_children(|row| {
            row.spawn((
                Text::new("Curve:"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            row.spawn((
                SetupButton(SetupAction::ToggleCurve),
                Button,
                Node { padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)), ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.35)),
            )).with_children(|btn| {
                btn.spawn((
                    SetupCurveText,
                    Text::new(curve_label(cfg.curve)),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(Color::srgb(0.3, 0.9, 0.9)),
                ));
            });
        });

        // Rate rows
        spawn_rate_row(panel, "HP Rate", 0, cfg.hp_rate, cfg.curve);
        spawn_rate_row(panel, "Dmg Rate", 1, cfg.dmg_rate, cfg.curve);
        spawn_rate_row(panel, "Spawn Rate", 2, cfg.spawn_rate, cfg.curve);

        // Preview
        panel.spawn((
            Text::new("-- Preview --"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node { margin: UiRect::top(Val::Px(4.0)), ..default() },
        ));
        panel.spawn((
            SetupPreviewText,
            Text::new(build_preview(cfg)),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.75, 0.85, 0.75)),
        ));
    });
}

fn spawn_rate_row(panel: &mut ChildSpawnerCommands, label: &str, param: u8, value: f32, curve: CurveType) {
    panel.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        ..default()
    }).with_children(|row| {
        row.spawn((
            Text::new(format!("{}:", label)),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node { min_width: Val::Px(100.0), ..default() },
        ));
        row.spawn((
            SetupButton(SetupAction::AdjustRate { param, positive: false }),
            Button,
            Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)), ..default() },
            BackgroundColor(Color::srgb(0.25, 0.1, 0.1)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("[-]"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(1.0, 0.4, 0.4)),
            ));
        });
        row.spawn((
            SetupText(param),
            Text::new(format_rate(value, curve)),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::WHITE),
            Node { min_width: Val::Px(80.0), ..default() },
        ));
        row.spawn((
            SetupButton(SetupAction::AdjustRate { param, positive: true }),
            Button,
            Node { padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)), ..default() },
            BackgroundColor(Color::srgb(0.1, 0.25, 0.1)),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("[+]"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.4, 1.0, 0.4)),
            ));
        });
    });
}

fn curve_label(curve: CurveType) -> &'static str {
    match curve { CurveType::Linear => "Linear", CurveType::Exponential => "Exponential" }
}

fn format_rate(v: f32, curve: CurveType) -> String {
    match curve { CurveType::Linear => format!("{:.3}", v), CurveType::Exponential => format!("{:.4}", v) }
}

fn build_preview(cfg: &DifficultyConfig) -> String {
    let times = [(300.0, "5min"), (600.0, "10min"), (900.0, "15min")];
    times.iter().map(|(t, label)| {
        format!("{}: HP {:.1}x  Dmg {:.1}x  Spawn {:.1}/s", label, cfg.hp_at(*t), cfg.dmg_at(*t), cfg.spawn_at(*t))
    }).collect::<Vec<_>>().join("\n")
}

fn rate_step(curve: CurveType) -> f32 {
    match curve { CurveType::Linear => 0.001, CurveType::Exponential => 0.0002 }
}

fn clamp_rate(v: f32, curve: CurveType) -> f32 {
    match curve { CurveType::Linear => v.clamp(0.000, 0.100), CurveType::Exponential => v.clamp(1.0000, 1.0150) }
}

fn handle_setup_clicks(
    interaction_query: Query<(&Interaction, &SetupButton), Changed<Interaction>>,
    mut cfg: ResMut<DifficultyConfig>,
) {
    for (interaction, btn) in &interaction_query {
        if *interaction != Interaction::Pressed { continue; }
        match btn.0 {
            SetupAction::ToggleCurve => {
                *cfg = match cfg.curve {
                    CurveType::Linear => DifficultyConfig::exponential_defaults(),
                    CurveType::Exponential => DifficultyConfig::linear_defaults(),
                };
            }
            SetupAction::AdjustRate { param, positive } => {
                let step = rate_step(cfg.curve);
                let d = if positive { step } else { -step };
                match param {
                    0 => cfg.hp_rate = clamp_rate(cfg.hp_rate + d, cfg.curve),
                    1 => cfg.dmg_rate = clamp_rate(cfg.dmg_rate + d, cfg.curve),
                    2 => cfg.spawn_rate = clamp_rate(cfg.spawn_rate + d, cfg.curve),
                    _ => {}
                }
            }
        }
    }
}

fn update_setup_texts(
    cfg: Res<DifficultyConfig>,
    mut rate_texts: Query<(&SetupText, &mut Text), (Without<SetupCurveText>, Without<SetupPreviewText>)>,
    mut curve_texts: Query<&mut Text, (With<SetupCurveText>, Without<SetupText>, Without<SetupPreviewText>)>,
    mut preview_texts: Query<&mut Text, (With<SetupPreviewText>, Without<SetupText>, Without<SetupCurveText>)>,
) {
    if !cfg.is_changed() { return; }
    for (st, mut text) in &mut rate_texts {
        let v = match st.0 { 0 => cfg.hp_rate, 1 => cfg.dmg_rate, 2 => cfg.spawn_rate, _ => 0.0 };
        **text = format_rate(v, cfg.curve);
    }
    for mut text in &mut curve_texts { **text = curve_label(cfg.curve).to_string(); }
    for mut text in &mut preview_texts { **text = build_preview(&cfg); }
}

// === Browser Toggle (keys 6, 7, Esc) ===

fn handle_browser_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut dbg: ResMut<DebugState>,
    mut commands: Commands,
    browser_query: Query<Entity, With<BrowserPanel>>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
) {
    if !dbg.active { return; }
    let mut requested = None;
    if keys.just_pressed(KeyCode::Digit6) {
        requested = Some(if dbg.browser_open == BrowserMode::Weapons { BrowserMode::None } else { BrowserMode::Weapons });
    }
    if keys.just_pressed(KeyCode::Digit7) {
        requested = Some(if dbg.browser_open == BrowserMode::Upgrades { BrowserMode::None } else { BrowserMode::Upgrades });
    }
    if keys.just_pressed(KeyCode::Escape) && dbg.browser_open != BrowserMode::None {
        requested = Some(BrowserMode::None);
    }
    let Some(new_mode) = requested else { return };
    for entity in &browser_query { commands.entity(entity).despawn(); }
    dbg.browser_page = 0;
    match new_mode {
        BrowserMode::Weapons => spawn_weapon_browser(&mut commands, &weapons, 0),
        BrowserMode::Upgrades => spawn_upgrade_browser(&mut commands, &upgrades, 0),
        BrowserMode::None => {}
    }
    dbg.browser_open = new_mode;
}

fn handle_browser_navigation(
    interaction_query: Query<(&Interaction, &BrowserPageButton), Changed<Interaction>>,
    mut dbg: ResMut<DebugState>,
    mut commands: Commands,
    browser_query: Query<Entity, With<BrowserPanel>>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
) {
    for (interaction, page_btn) in &interaction_query {
        if *interaction != Interaction::Pressed { continue; }
        let total_items = match dbg.browser_open {
            BrowserMode::Weapons => weapons.weapons.len(),
            BrowserMode::Upgrades => upgrades.upgrades.len(),
            BrowserMode::None => return,
        };
        let total_pages = (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        let new_page = (dbg.browser_page as i32 + page_btn.delta).clamp(0, total_pages as i32 - 1) as usize;
        if new_page == dbg.browser_page { return; }
        dbg.browser_page = new_page;
        for entity in &browser_query { commands.entity(entity).despawn(); }
        match dbg.browser_open {
            BrowserMode::Weapons => spawn_weapon_browser(&mut commands, &weapons, new_page),
            BrowserMode::Upgrades => spawn_upgrade_browser(&mut commands, &upgrades, new_page),
            BrowserMode::None => {}
        }
        return;
    }
}

// === Browser Spawning ===

fn rarity_color_from_str(rarity: &str) -> Color {
    match rarity {
        "Common" => Color::srgb(0.6, 0.6, 0.6), "Uncommon" => Color::srgb(0.2, 0.8, 0.2),
        "Rare" => Color::srgb(0.3, 0.5, 1.0), "Epic" => Color::srgb(0.7, 0.3, 0.9),
        _ => Color::WHITE,
    }
}

const RARITY_ORDER: [&str; 4] = ["Common", "Uncommon", "Rare", "Epic"];

fn rarity_sort_key(rarity: &str) -> usize {
    RARITY_ORDER.iter().position(|r| *r == rarity).unwrap_or(99)
}

fn spawn_weapon_browser(commands: &mut Commands, weapons: &WeaponDatabase, page: usize) {
    let mut sorted: Vec<(usize, &crate::data::weapons::WeaponDefinition)> =
        weapons.weapons.iter().enumerate().collect();
    sorted.sort_by(|a, b| rarity_sort_key(&a.1.rarity).cmp(&rarity_sort_key(&b.1.rarity)).then(a.1.name.cmp(&b.1.name)));

    let total = sorted.len();
    let pages = (total + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
    let start = page * ITEMS_PER_PAGE;
    let end = (start + ITEMS_PER_PAGE).min(total);
    let items = &sorted[start..end];

    commands.spawn((
        BrowserPanel,
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GlobalZIndex(50),
    )).with_children(|overlay| {
        overlay.spawn(Node { width: Val::Px(800.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(2.0), ..default() })
        .with_children(|panel| {
            panel.spawn((
                Text::new(format!("WEAPONS  --  Click to grant (free)  |  Page {}/{}  |  6 or Esc to close", page + 1, pages)),
                TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(1.0, 0.9, 0.3)),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            ));
            let mut last_rarity = "";
            for &(idx, def) in items {
                if def.rarity != last_rarity {
                    last_rarity = &def.rarity;
                    panel.spawn((Text::new(format!("--- {} ---", def.rarity.to_uppercase())), TextFont { font_size: 13.0, ..default() }, TextColor(rarity_color_from_str(&def.rarity)), Node { margin: UiRect::top(Val::Px(6.0)), ..default() }));
                }
                let ability_str = if def.ability.is_empty() { String::new() } else { format!(" | {}", def.ability) };
                let label = format!("{} -- {} {} -- {}dmg {:.1}s ({} DPS) R:{}{}", def.name, def.weapon_type, def.attack_type, def.damage, def.attack_cooldown, def.dps, def.range, ability_str);
                panel.spawn((BrowserItemButton { index: idx, is_weapon: true }, Button, Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.15, 0.2)))).with_children(|btn| {
                    btn.spawn((Text::new(label), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.8))));
                });
            }
            spawn_page_nav(panel, page, pages);
        });
    });
}

fn spawn_upgrade_browser(commands: &mut Commands, upgrades: &UpgradeDatabase, page: usize) {
    let mut sorted: Vec<(usize, &crate::data::upgrades::UpgradeDefinition)> =
        upgrades.upgrades.iter().enumerate().collect();
    sorted.sort_by(|a, b| rarity_sort_key(&a.1.rarity).cmp(&rarity_sort_key(&b.1.rarity)).then(a.1.name.cmp(&b.1.name)));

    let total = sorted.len();
    let pages = (total + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
    let start = page * ITEMS_PER_PAGE;
    let end = (start + ITEMS_PER_PAGE).min(total);
    let items = &sorted[start..end];

    commands.spawn((
        BrowserPanel,
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GlobalZIndex(50),
    )).with_children(|overlay| {
        overlay.spawn(Node { width: Val::Px(800.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(2.0), ..default() })
        .with_children(|panel| {
            panel.spawn((
                Text::new(format!("UPGRADES  --  Click to grant (free)  |  Page {}/{}  |  7 or Esc to close", page + 1, pages)),
                TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(1.0, 0.9, 0.3)),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            ));
            let mut last_rarity = "";
            for &(idx, def) in items {
                if def.rarity != last_rarity {
                    last_rarity = &def.rarity;
                    panel.spawn((Text::new(format!("--- {} ---", def.rarity.to_uppercase())), TextFont { font_size: 13.0, ..default() }, TextColor(rarity_color_from_str(&def.rarity)), Node { margin: UiRect::top(Val::Px(6.0)), ..default() }));
                }
                let label = format!("{} [{}] -- {}", def.name, def.upgrade_type, def.description);
                panel.spawn((BrowserItemButton { index: idx, is_weapon: false }, Button, Node { padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.15, 0.2)))).with_children(|btn| {
                    btn.spawn((Text::new(label), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.8))));
                });
            }
            spawn_page_nav(panel, page, pages);
        });
    });
}

fn spawn_page_nav(panel: &mut ChildSpawnerCommands, page: usize, total_pages: usize) {
    panel.spawn(Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: Val::Px(16.0), margin: UiRect::top(Val::Px(10.0)), ..default() })
    .with_children(|row| {
        let prev_color = if page > 0 { Color::srgb(0.3, 0.3, 0.5) } else { Color::srgb(0.15, 0.15, 0.2) };
        row.spawn((BrowserPageButton { delta: -1 }, Button, Node { padding: UiRect::axes(Val::Px(20.0), Val::Px(6.0)), ..default() }, BackgroundColor(prev_color))).with_children(|btn| {
            btn.spawn((Text::new("<< Prev"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE)));
        });
        row.spawn((Text::new(format!("{} / {}", page + 1, total_pages)), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.7, 0.7, 0.7)), Node { align_self: AlignSelf::Center, ..default() }));
        let next_color = if page + 1 < total_pages { Color::srgb(0.3, 0.3, 0.5) } else { Color::srgb(0.15, 0.15, 0.2) };
        row.spawn((BrowserPageButton { delta: 1 }, Button, Node { padding: UiRect::axes(Val::Px(20.0), Val::Px(6.0)), ..default() }, BackgroundColor(next_color))).with_children(|btn| {
            btn.spawn((Text::new("Next >>"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE)));
        });
    });
}

fn handle_browser_clicks(
    interaction_query: Query<(&Interaction, &BrowserItemButton), Changed<Interaction>>,
    mut weapon_events: MessageWriter<WeaponPurchasedEvent>,
    mut upgrade_events: MessageWriter<UpgradePurchasedEvent>,
    mut purchased_upgrades: ResMut<PurchasedUpgrades>,
    weapons: Res<WeaponDatabase>,
    upgrades: Res<UpgradeDatabase>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed { continue; }
        if button.is_weapon {
            let def = &weapons.weapons[button.index];
            weapon_events.write(WeaponPurchasedEvent { weapon_name: def.name.clone(), definition_index: button.index });
            info!("[DEBUG] Granted weapon: {}", def.name);
        } else {
            let def = &upgrades.upgrades[button.index];
            purchased_upgrades.items.push(def.name.clone());
            upgrade_events.write(UpgradePurchasedEvent { upgrade_name: def.name.clone(), definition_index: button.index });
            info!("[DEBUG] Granted upgrade: {} ({})", def.name, def.upgrade_type);
        }
    }
}
