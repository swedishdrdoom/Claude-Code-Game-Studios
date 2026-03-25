use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

use crate::components::combat::{EnemyKilledEvent, TowerDamageEvent};
use crate::components::economy::Gold;
use crate::components::enemy::*;
use crate::components::run::GameState;
use crate::components::tower::{Health, HealOnAttacked, SpikesDamage, Tower};
use crate::plugins::core::ArenaConfig;

/// Wave escalation state.
#[derive(Resource, Debug)]
pub struct WaveState {
    pub base_spawn_rate: f32,
    pub spawn_accumulator: f32,
    pub spawn_debt: f32,
    pub hp_multiplier: f32,
    pub damage_multiplier: f32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            base_spawn_rate: 0.5,
            spawn_accumulator: 0.0,
            spawn_debt: 0.0,
            hp_multiplier: 1.0,
            damage_multiplier: 1.0,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveState>()
            .add_message::<EnemyKilledEvent>()
            .add_systems(Startup, setup_armor_legend)
            .add_systems(OnEnter(GameState::Boss), spawn_boss)
            .add_systems(Update, (
                spawn_enemies.run_if(in_state(GameState::Playing)),
                check_boss_death.run_if(in_state(GameState::Boss)),
                move_enemies.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::GracePeriod))
                        .or(in_state(GameState::Boss)),
                ),
                enemy_attack_tower.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Boss)),
                ),
                update_wave_escalation.run_if(in_state(GameState::Playing)),
                apply_separation,
                tick_frost_timers,
                update_enemy_health_bars,
                update_enemy_hp_text,
                animate_floating_text,
                spawn_gold_popups,
                update_armor_legend,
            ));
    }
}

/// Piecewise-linear interpolation over keyframes (time_seconds, value).
fn lerp_curve(keyframes: &[(f32, f32)], t: f32) -> f32 {
    if t <= keyframes[0].0 { return keyframes[0].1; }
    let last = keyframes.len() - 1;
    if t >= keyframes[last].0 { return keyframes[last].1; }
    for i in 0..last {
        let (t0, v0) = keyframes[i];
        let (t1, v1) = keyframes[i + 1];
        if t >= t0 && t < t1 {
            return v0 + (v1 - v0) * (t - t0) / (t1 - t0);
        }
    }
    keyframes[last].1
}

// Base (strong) enemy HP curve — piecewise keyframes.
// Accelerates early (punishes gold-rushing), decelerates mid, re-accelerates for spike.
// Weak armor type gets 50% of this value.
const BASE_HP_CURVE: [(f32, f32); 31] = [
    (0.0,   600.0),    // 0:00
    (30.0,  660.0),    // 0:30
    (60.0,  755.0),    // 1:00
    (90.0,  888.0),    // 1:30
    (120.0, 1065.0),   // 2:00
    (150.0, 1290.0),   // 2:30
    (180.0, 1568.0),   // 3:00 — peak growth rate
    (210.0, 1902.0),   // 3:30
    (240.0, 2298.0),   // 4:00
    (270.0, 2760.0),   // 4:30
    (300.0, 3293.0),   // 5:00 — decelerating
    (330.0, 3900.0),   // 5:30
    (360.0, 4587.0),   // 6:00
    (390.0, 5358.0),   // 6:30
    (420.0, 6218.0),   // 7:00
    (450.0, 7170.0),   // 7:30
    (480.0, 8220.0),   // 8:00
    (510.0, 9372.0),   // 8:30
    (540.0, 10631.0),  // 9:00
    (570.0, 11981.0),  // 9:30
    (600.0, 13419.0),  // 10:00 — trough, spike begins
    (630.0, 15029.0),  // 10:30
    (660.0, 17208.0),  // 11:00
    (690.0, 20047.0),  // 11:30
    (720.0, 23756.0),  // 12:00
    (750.0, 28626.0),  // 12:30
    (780.0, 35067.0),  // 13:00
    (810.0, 43658.0),  // 13:30
    (840.0, 55227.0),  // 14:00
    (870.0, 70967.0),  // 14:30
    (900.0, 92612.0),  // 15:00
];

// Weak armor type rotation: Light → Medium → Heavy → Fortified, 30s each, 2 min cycle.
const WEAK_ROTATION: [ArmorType; 4] = [
    ArmorType::Light,
    ArmorType::Medium,
    ArmorType::Heavy,
    ArmorType::Fortified,
];

/// Returns which armor type is currently "weak" (50% HP).
fn current_weak_type(elapsed: f32) -> ArmorType {
    let phase = ((elapsed / 30.0).floor() as usize) % 4;
    WEAK_ROTATION[phase]
}

/// Compute enemy HP based on armor type and elapsed time.
/// HP is a staircase — flat within each 30s window, jumps at boundaries.
/// The current weak type gets 50% HP; all others get full base HP.
fn compute_enemy_hp(armor_type: ArmorType, elapsed: f32) -> f32 {
    // Snap to the start of the current 30s window
    let window_start = (elapsed / 30.0).floor() * 30.0;
    let base = lerp_curve(&BASE_HP_CURVE, window_start);
    if armor_type == current_weak_type(elapsed) {
        base * 0.5
    } else {
        base
    }
}

/// Format HP for display: 600, 1.2k, 35k, 5.0M
fn format_hp(hp: f32) -> String {
    if hp >= 1_000_000.0 {
        format!("{:.1}M", hp / 1_000_000.0)
    } else if hp >= 10_000.0 {
        format!("{:.0}k", hp / 1_000.0)
    } else if hp >= 1_000.0 {
        format!("{:.1}k", hp / 1_000.0)
    } else {
        format!("{:.0}", hp)
    }
}

// Enemy damage per hit — step-based, 30s windows.
// ~+6.5/window until 10:00, then accelerating hard. (~1.5x previous curve)
const BASE_DMG_STEPS: [f32; 30] = [
    15.0,  21.0,  27.0,  33.0,  40.0,  47.0,  53.0,  59.0,  66.0,  72.0,   // 0:00–4:30
    78.0,  86.0,  92.0,  99.0, 105.0, 111.0, 119.0, 125.0, 131.0, 137.0,   // 5:00–9:30
   143.0, 165.0, 195.0, 233.0, 278.0, 345.0, 450.0, 600.0, 825.0, 1200.0,  // 10:00–14:30
];

// Spawn rate (enemies/sec) — step-based, 30s windows.
// Early (0-3m): 0.5/s, Mid (3-10m): 0.5–3/s, Late (10-15m): 3–12/s.
const BASE_SPAWN_STEPS: [f32; 30] = [
    0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 1.0, 1.0,   // 0:00–4:30
    1.0, 1.0, 1.5, 1.5, 1.5, 2.0, 2.0, 2.5, 2.5, 3.0,   // 5:00–9:30
    3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, // 10:00–14:30
];

/// Look up a step-based value by 30s window index.
fn step_value(steps: &[f32; 30], elapsed: f32) -> f32 {
    let window = ((elapsed / 30.0).floor() as usize).min(29);
    steps[window]
}

fn compute_enemy_damage(elapsed: f32) -> f32 {
    step_value(&BASE_DMG_STEPS, elapsed)
}

fn compute_spawn_rate(elapsed: f32) -> f32 {
    step_value(&BASE_SPAWN_STEPS, elapsed)
}

fn update_wave_escalation(
    mut wave: ResMut<WaveState>,
    timer: Res<crate::components::run::RunTimer>,
) {
    let t = timer.elapsed;
    wave.hp_multiplier = 1.0; // HP computed per-enemy via compute_enemy_hp
    wave.damage_multiplier = compute_enemy_damage(t);
    wave.base_spawn_rate = compute_spawn_rate(t);
}

/// 8 cardinal/ordinal directions for spawn lanes.
const SPAWN_DIRECTIONS: [f32; 8] = [
    0.0,                        // East
    TAU * 1.0 / 8.0,           // North-East
    TAU * 2.0 / 8.0,           // North
    TAU * 3.0 / 8.0,           // North-West
    TAU * 4.0 / 8.0,           // West
    TAU * 5.0 / 8.0,           // South-West
    TAU * 6.0 / 8.0,           // South
    TAU * 7.0 / 8.0,           // South-East
];

/// All enemies travel at the same speed: spawn_ring / 20 seconds.
const ENEMY_TRAVEL_TIME: f32 = 20.0;

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    arena: Res<ArenaConfig>,
    timer: Res<crate::components::run::RunTimer>,
) {
    // Tick once per second — spawn a batch
    wave.spawn_accumulator += time.delta_secs();
    if wave.spawn_accumulator < 1.0 { return; }
    wave.spawn_accumulator -= 1.0;

    // Accumulate fractional spawns (handles sub-1 rates like 0.5/s)
    wave.spawn_debt += wave.base_spawn_rate;
    let enemies_this_batch = wave.spawn_debt.floor() as u32;
    wave.spawn_debt -= enemies_this_batch as f32;
    if enemies_this_batch == 0 { return; }

    let mut rng = rand::rng();
    let travel_speed = (arena.spawn_ring_radius - 30.0) / ENEMY_TRAVEL_TIME;

    // Cluster size scales with game phase:
    //   Early (0–5 min): 1–3, Mid (5–10 min): 2–4, Late (10+ min): 3–5
    let max_cluster = (3 + (timer.elapsed / 300.0).floor() as u32).min(5);
    let min_cluster = max_cluster.saturating_sub(2).max(1);

    // Split batch into clusters, each picking its own lane
    let mut remaining = enemies_this_batch;
    while remaining > 0 {
        let size = remaining.min(rng.random_range(min_cluster..=max_cluster));
        remaining -= size;
        let lane = SPAWN_DIRECTIONS[rng.random_range(0..8)];

        for _ in 0..size {
        let jitter = rng.random_range(-0.20..0.20); // ~±11 degrees spread within cluster
        let angle = lane + jitter;
        let x = angle.cos() * arena.spawn_ring_radius;
        let y = angle.sin() * arena.spawn_ring_radius;

        let armor_type = match rng.random_range(0..5) {
            0 => ArmorType::Light,
            1 => ArmorType::Medium,
            2 => ArmorType::Heavy,
            3 => ArmorType::Fortified,
            _ => ArmorType::Unarmored,
        };

        let hp = match armor_type {
            ArmorType::Hero => 10000.0,
            _ => compute_enemy_hp(armor_type, timer.elapsed),
        };

        let color = match armor_type {
            ArmorType::Light => Color::srgb(0.9, 0.9, 0.3),
            ArmorType::Medium => Color::srgb(0.3, 0.8, 0.3),
            ArmorType::Heavy => Color::srgb(0.3, 0.3, 0.9),
            ArmorType::Fortified => Color::srgb(0.7, 0.3, 0.3),
            ArmorType::Unarmored => Color::srgb(0.8, 0.8, 0.8),
            ArmorType::Hero => Color::srgb(0.9, 0.1, 0.9),
        };

        let damage = compute_enemy_damage(timer.elapsed);

        let enemy_size = match armor_type {
            ArmorType::Light => 21.0,
            ArmorType::Medium => 24.0,
            ArmorType::Heavy => 33.0,
            ArmorType::Fortified => 36.0,
            ArmorType::Unarmored => 18.0,
            ArmorType::Hero => 60.0,
        };

        commands.spawn((
            Enemy,
            armor_type,
            EnemyHealth { current: hp, max: hp },
            EnemyArmor::default(),
            MoveSpeed {
                base: travel_speed,
                multiplier: 1.0,
            },
            GoldBounty::default(),
            EnemyAttack {
                damage,
                cooldown: 1.0,
                timer: 0.0,
                range: 30.0,
            },
            FrostStacks::default(),
            Burning::default(),
            Sprite {
                color,
                custom_size: Some(Vec2::splat(enemy_size)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.5),
        )).with_children(|parent: &mut ChildSpawnerCommands| {
            // Health bar background
            parent.spawn((
                EnemyHealthBarBg,
                Sprite {
                    color: Color::srgba(0.2, 0.2, 0.2, 0.8),
                    custom_size: Some(Vec2::new(enemy_size + 4.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, enemy_size * 0.5 + 4.0, 0.1),
            ));
            // Health bar fill
            let bar_width = enemy_size + 2.0;
            parent.spawn((
                EnemyHealthBarFill { full_width: bar_width },
                Sprite {
                    color: Color::srgb(0.1, 0.9, 0.1),
                    custom_size: Some(Vec2::new(bar_width, 2.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, enemy_size * 0.5 + 4.0, 0.2),
            ));
            // HP text below enemy
            parent.spawn((
                EnemyHpText,
                Text2d::new(format_hp(hp)),
                TextFont { font_size: 40.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, -(enemy_size * 0.5 + 8.0), 0.2),
            ));
        });
        } // end for _ in 0..size
        } // end while remaining > 0
}

fn move_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveSpeed, &EnemyAttack, &FrostStacks), With<Enemy>>,
) {
    for (mut transform, speed, attack, frost) in &mut query {
        let pos = transform.translation.truncate();
        let dist = pos.length();

        // Stop moving when within attack range
        if dist > attack.range {
            let mut spd = speed.effective();
            if frost.frozen { spd *= 0.5; } // Frost: 50% move speed reduction
            let direction = -pos.normalize_or_zero();
            let movement = direction * spd * time.delta_secs();
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
        }
    }
}

fn enemy_attack_tower(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyAttack, &mut EnemyHealth, &GoldBounty, &Burning, &FrostStacks), With<Enemy>>,
    mut tower_stats: Query<(&SpikesDamage, &mut Health, &HealOnAttacked), With<Tower>>,
    mut tower_damage: MessageWriter<TowerDamageEvent>,
    mut killed_writer: MessageWriter<EnemyKilledEvent>,
    mut gold: ResMut<Gold>,
) {
    let Ok((spikes, mut tower_health, heal_on_attacked)) = tower_stats.single_mut() else { return };
    let spikes_dmg = spikes.total();
    let heal_amount = heal_on_attacked.amount;

    for (entity, transform, mut attack, mut health, bounty, burning, frost) in &mut enemy_query {
        let dist = transform.translation.truncate().length();

        if dist <= attack.range {
            // Frost: 25% attack speed reduction (timer ticks 75% as fast)
            let dt = if frost.frozen { time.delta_secs() * 0.75 } else { time.delta_secs() };
            attack.timer -= dt;
            if attack.timer <= 0.0 {
                // Enemy attacks tower
                tower_damage.write(TowerDamageEvent {
                    raw_damage: attack.damage,
                });
                attack.timer = attack.cooldown;

                // Heal tower when attacked (Dreadlord Fang, Hungering Maw)
                if heal_amount > 0.0 {
                    tower_health.current = (tower_health.current + heal_amount).min(tower_health.max);
                }

                // Spikes damage back to attacker
                if spikes_dmg > 0.0 {
                    health.current -= spikes_dmg;
                    if health.current <= 0.0 {
                        let bounty_amount = ((bounty.base as f32) * (1.0 + gold.bounty_bonus_percent)) as u32;
                        gold.current += bounty_amount;
                        gold.total_earned += bounty_amount;
                        killed_writer.write(EnemyKilledEvent {
                            position: transform.translation.truncate(),
                            gold_bounty: bounty_amount,
                            had_burning: burning.active,
                            fire_damage: burning.fire_damage,
                        });
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

fn apply_separation(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform), With<Enemy>>,
) {
    let separation_radius = 20.0_f32;
    let separation_strength = 50.0_f32;

    // Collect positions
    let positions: Vec<(Entity, Vec2)> = query
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    for (entity, mut transform) in &mut query {
        let pos = transform.translation.truncate();
        let mut push = Vec2::ZERO;

        for (other_entity, other_pos) in &positions {
            if *other_entity == entity {
                continue;
            }
            let diff = pos - *other_pos;
            let dist = diff.length();
            if dist > 0.0 && dist < separation_radius {
                push += diff.normalize() * (separation_radius - dist) / separation_radius;
            }
        }

        if push.length() > 0.0 {
            let movement = push.normalize() * separation_strength * time.delta_secs();
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
        }
    }
}

fn update_enemy_health_bars(
    enemy_query: Query<(&EnemyHealth, &Children), With<Enemy>>,
    mut fill_query: Query<(&EnemyHealthBarFill, &mut Sprite, &mut Transform)>,
) {
    for (health, children) in &enemy_query {
        let hp_fraction = (health.current / health.max).clamp(0.0, 1.0);
        for child in children.iter() {
            if let Ok((bar, mut sprite, mut transform)) = fill_query.get_mut(child) {
                let new_width = bar.full_width * hp_fraction;
                let height = sprite.custom_size.map(|s| s.y).unwrap_or(2.0);
                sprite.custom_size = Some(Vec2::new(new_width, height));
                // Offset to left-align the fill bar
                transform.translation.x = -(bar.full_width - new_width) * 0.5;

                // Color: green → yellow → red
                sprite.color = if hp_fraction > 0.5 {
                    Color::srgb(0.1, 0.9, 0.1)
                } else if hp_fraction > 0.25 {
                    Color::srgb(0.9, 0.9, 0.1)
                } else {
                    Color::srgb(0.9, 0.1, 0.1)
                };
            }
        }
    }
}

fn spawn_gold_popups(
    mut events: MessageReader<EnemyKilledEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.spawn((
            FloatingText {
                lifetime: 0.8,
                max_lifetime: 0.8,
                rise_speed: 60.0,
            },
            Text2d::new(format!("+{}g", event.gold_bounty)),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::srgb(1.0, 0.85, 0.0)),
            Transform::from_xyz(event.position.x, event.position.y + 20.0, 2.0),
        ));
    }
}

fn animate_floating_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut ft, mut transform, mut color) in &mut query {
        ft.lifetime -= time.delta_secs();
        if ft.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Rise up
        transform.translation.y += ft.rise_speed * time.delta_secs();

        // Fade out
        let alpha = ft.lifetime / ft.max_lifetime;
        color.0 = Color::srgba(1.0, 0.85, 0.0, alpha);
    }
}

fn spawn_boss(
    mut commands: Commands,
    arena: Res<ArenaConfig>,
    enemy_query: Query<Entity, (With<Enemy>, Without<Boss>)>,
) {
    // Clear all remaining non-boss enemies
    for entity in &enemy_query {
        commands.entity(entity).despawn();
    }
    info!("Cleared all enemies for boss phase");

    // Spawn boss from a random angle
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let x = angle.cos() * arena.spawn_ring_radius;
    let y = angle.sin() * arena.spawn_ring_radius;

    let boss_hp = 5_000_000.0;
    let boss_damage = 100.0;
    let boss_size = 92.0; // 80 * 1.15

    commands.spawn((
        Enemy,
        Boss,
        ArmorType::Hero,
        EnemyHealth { current: boss_hp, max: boss_hp },
        EnemyArmor::default(),
        MoveSpeed {
            base: 30.0, // Very slow
            multiplier: 1.0,
        },
        GoldBounty { base: 0 }, // Victory is the reward
        EnemyAttack {
            damage: boss_damage,
            cooldown: 1.5,
            timer: 0.0,
            range: 50.0,
        },
        FrostStacks::default(),
        Burning::default(),
        Sprite {
            color: Color::srgb(0.9, 0.1, 0.9), // Purple
            custom_size: Some(Vec2::splat(boss_size)),
            ..default()
        },
        Transform::from_xyz(x, y, 0.9),
    )).with_children(|parent: &mut ChildSpawnerCommands| {
        // Boss health bar (larger)
        let bar_width = boss_size + 20.0;
        parent.spawn((
            EnemyHealthBarBg,
            Sprite {
                color: Color::srgba(0.2, 0.2, 0.2, 0.8),
                custom_size: Some(Vec2::new(bar_width + 2.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(0.0, boss_size * 0.5 + 8.0, 0.1),
        ));
        parent.spawn((
            EnemyHealthBarFill { full_width: bar_width },
            Sprite {
                color: Color::srgb(0.9, 0.1, 0.1),
                custom_size: Some(Vec2::new(bar_width, 4.0)),
                ..default()
            },
            Transform::from_xyz(0.0, boss_size * 0.5 + 8.0, 0.2),
        ));
        // HP text below boss
        parent.spawn((
            EnemyHpText,
            Text2d::new(format_hp(boss_hp)),
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, -(boss_size * 0.5 + 12.0), 0.2),
        ));
    });

    info!("BOSS spawned with {} HP!", boss_hp);
}

fn check_boss_death(
    boss_query: Query<&EnemyHealth, With<Boss>>,
    mut boss_killed: MessageWriter<crate::components::combat::BossKilledEvent>,
) {
    // If no boss entity exists (it was despawned by damage system), boss is dead
    if boss_query.is_empty() {
        boss_killed.write(crate::components::combat::BossKilledEvent);
        info!("Boss defeated!");
    }
}

fn tick_frost_timers(
    time: Res<Time>,
    mut query: Query<&mut FrostStacks, With<Enemy>>,
) {
    for mut frost in &mut query {
        if frost.frozen {
            frost.freeze_timer -= time.delta_secs();
            if frost.freeze_timer <= 0.0 {
                frost.frozen = false;
            }
        }
    }
}

fn update_enemy_hp_text(
    enemy_query: Query<(&EnemyHealth, &Children), With<Enemy>>,
    mut text_query: Query<&mut Text2d, With<EnemyHpText>>,
) {
    for (health, children) in &enemy_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = format_hp(health.current);
            }
        }
    }
}

fn setup_armor_legend(mut commands: Commands) {
    // Full-width container to center the legend at the top
    commands.spawn((
        ArmorLegendPanel,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(30),
    )).with_children(|parent| {
        // Inner row with background
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(14.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        )).with_children(|row| {
            let entries: [(ArmorType, &str, Color); 5] = [
                (ArmorType::Light,     "Light",     Color::srgb(0.9, 0.9, 0.3)),
                (ArmorType::Medium,    "Medium",    Color::srgb(0.3, 0.8, 0.3)),
                (ArmorType::Heavy,     "Heavy",     Color::srgb(0.3, 0.3, 0.9)),
                (ArmorType::Fortified, "Fortified", Color::srgb(0.7, 0.3, 0.3)),
                (ArmorType::Unarmored, "Unarmored", Color::srgb(0.8, 0.8, 0.8)),
            ];

            for (armor_type, label, color) in entries {
                row.spawn((
                    ArmorLegendEntry(armor_type),
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                )).with_children(|entry| {
                    // Color swatch
                    entry.spawn((
                        Node {
                            width: Val::Px(12.0),
                            height: Val::Px(12.0),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                    // Label text
                    entry.spawn((
                        ArmorLegendText,
                        Text::new(label),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(color),
                    ));
                });
            }
        });
    });
}

fn update_armor_legend(
    timer: Res<crate::components::run::RunTimer>,
    entry_query: Query<(&ArmorLegendEntry, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<ArmorLegendText>>,
) {
    let weak = current_weak_type(timer.elapsed);
    for (entry, children) in &entry_query {
        for child in children.iter() {
            if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                let base_label = match entry.0 {
                    ArmorType::Light => "Light",
                    ArmorType::Medium => "Medium",
                    ArmorType::Heavy => "Heavy",
                    ArmorType::Fortified => "Fortified",
                    ArmorType::Unarmored => "Unarmored",
                    ArmorType::Hero => "Hero",
                };
                if entry.0 == weak {
                    **text = format!("{} (WEAK)", base_label);
                    color.0 = Color::srgb(1.0, 1.0, 1.0);
                } else {
                    **text = base_label.to_string();
                    color.0 = match entry.0 {
                        ArmorType::Light => Color::srgb(0.9, 0.9, 0.3),
                        ArmorType::Medium => Color::srgb(0.3, 0.8, 0.3),
                        ArmorType::Heavy => Color::srgb(0.3, 0.3, 0.9),
                        ArmorType::Fortified => Color::srgb(0.7, 0.3, 0.3),
                        ArmorType::Unarmored | ArmorType::Hero => Color::srgb(0.8, 0.8, 0.8),
                    };
                }
            }
        }
    }
}
