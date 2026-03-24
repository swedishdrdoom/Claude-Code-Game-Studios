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
    pub hp_multiplier: f32,
    pub damage_multiplier: f32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            base_spawn_rate: 1.0, // enemies per second at time 0
            spawn_accumulator: 0.0,
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
                update_enemy_health_bars,
                animate_floating_text,
                spawn_gold_popups,
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

// HP multiplier curve (base 300 HP)
// Early: punishing (1x→5x in 3 min) — punishes pure gold builds
// Mid: easing (5x→10x over min 3-6) — breathing room
// Late: steep ramp to 40x at 10 min (12,000 HP)
// Spike: 10-15 min accelerating toward 400x
const HP_CURVE: [(f32, f32); 16] = [
    (0.0,   1.0),    // 300 HP
    (60.0,  2.0),    // 600 HP — minute 1, early pressure
    (120.0, 3.5),    // 1,050 HP
    (180.0, 5.0),    // 1,500 HP — minute 3
    (240.0, 6.5),    // 1,950 HP — easing
    (300.0, 8.0),    // 2,400 HP — minute 5
    (360.0, 10.0),   // 3,000 HP — minute 6
    (420.0, 15.0),   // 4,500 HP — ramping
    (480.0, 22.0),   // 6,600 HP
    (540.0, 30.0),   // 9,000 HP
    (600.0, 40.0),   // 12,000 HP — minute 10 target
    (660.0, 65.0),   // 19,500 HP — spike phase
    (720.0, 100.0),  // 30,000 HP
    (780.0, 160.0),  // 48,000 HP
    (840.0, 250.0),  // 75,000 HP
    (900.0, 400.0),  // 120,000 HP — minute 15
];

// Damage multiplier curve
// Must force armor/defense investment — undefended tower melts when surrounded
const DMG_CURVE: [(f32, f32); 16] = [
    (0.0,   1.0),
    (60.0,  1.3),
    (120.0, 1.6),
    (180.0, 2.0),    // minute 3
    (240.0, 2.5),
    (300.0, 3.0),    // minute 5
    (360.0, 3.5),    // minute 6
    (420.0, 4.5),
    (480.0, 6.0),
    (540.0, 8.0),
    (600.0, 10.0),   // minute 10
    (660.0, 14.0),   // spike
    (720.0, 20.0),
    (780.0, 30.0),
    (840.0, 45.0),
    (900.0, 70.0),   // minute 15
];

// Spawn rate curve (enemies per second)
const SPAWN_CURVE: [(f32, f32); 6] = [
    (0.0,   1.0),
    (180.0, 2.0),    // minute 3
    (360.0, 3.0),    // minute 6
    (600.0, 5.0),    // minute 10
    (750.0, 8.0),
    (900.0, 12.0),   // minute 15
];

fn update_wave_escalation(
    mut wave: ResMut<WaveState>,
    timer: Res<crate::components::run::RunTimer>,
) {
    let t = timer.elapsed;
    wave.hp_multiplier = lerp_curve(&HP_CURVE, t);
    wave.damage_multiplier = lerp_curve(&DMG_CURVE, t);
    wave.base_spawn_rate = lerp_curve(&SPAWN_CURVE, t);
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    arena: Res<ArenaConfig>,
    timer: Res<crate::components::run::RunTimer>,
) {
    wave.spawn_accumulator += wave.base_spawn_rate * time.delta_secs();

    let mut rng = rand::rng();

    while wave.spawn_accumulator >= 1.0 {
        wave.spawn_accumulator -= 1.0;

        // Random angle on spawn ring
        let angle = rng.random_range(0.0..TAU);
        let x = angle.cos() * arena.spawn_ring_radius;
        let y = angle.sin() * arena.spawn_ring_radius;

        // Random armor type (equal weight for now)
        let armor_type = match rng.random_range(0..5) {
            0 => ArmorType::Light,
            1 => ArmorType::Medium,
            2 => ArmorType::Heavy,
            3 => ArmorType::Fortified,
            _ => ArmorType::Unarmored,
        };

        let base_hp = match armor_type {
            ArmorType::Light => 300.0,
            ArmorType::Medium => 300.0,
            ArmorType::Heavy => 300.0,
            ArmorType::Fortified => 300.0,
            ArmorType::Unarmored => 300.0,
            ArmorType::Hero => 10000.0,
        };

        let base_speed = match armor_type {
            ArmorType::Light => 150.0,
            ArmorType::Medium => 100.0,
            ArmorType::Heavy => 60.0,
            ArmorType::Fortified => 50.0,
            ArmorType::Unarmored => 120.0,
            ArmorType::Hero => 80.0,
        };

        let base_damage = match armor_type {
            ArmorType::Light => 5.0,
            ArmorType::Medium => 10.0,
            ArmorType::Heavy => 20.0,
            ArmorType::Fortified => 30.0,
            ArmorType::Unarmored => 3.0,
            ArmorType::Hero => 50.0,
        };

        let color = match armor_type {
            ArmorType::Light => Color::srgb(0.9, 0.9, 0.3),
            ArmorType::Medium => Color::srgb(0.3, 0.8, 0.3),
            ArmorType::Heavy => Color::srgb(0.3, 0.3, 0.9),
            ArmorType::Fortified => Color::srgb(0.7, 0.3, 0.3),
            ArmorType::Unarmored => Color::srgb(0.8, 0.8, 0.8),
            ArmorType::Hero => Color::srgb(0.9, 0.1, 0.9),
        };

        let hp = base_hp * wave.hp_multiplier;
        let damage = base_damage * wave.damage_multiplier;

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
                base: base_speed,
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
        });
    }
}

fn move_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveSpeed, &EnemyAttack), With<Enemy>>,
) {
    for (mut transform, speed, attack) in &mut query {
        let pos = transform.translation.truncate();
        let dist = pos.length();

        // Stop moving when within attack range
        if dist > attack.range {
            let direction = -pos.normalize_or_zero();
            let movement = direction * speed.effective() * time.delta_secs();
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
        }
    }
}

fn enemy_attack_tower(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyAttack, &mut EnemyHealth, &GoldBounty, &Burning), With<Enemy>>,
    mut tower_stats: Query<(&SpikesDamage, &mut Health, &HealOnAttacked), With<Tower>>,
    mut tower_damage: MessageWriter<TowerDamageEvent>,
    mut killed_writer: MessageWriter<EnemyKilledEvent>,
    mut gold: ResMut<Gold>,
) {
    let Ok((spikes, mut tower_health, heal_on_attacked)) = tower_stats.single_mut() else { return };
    let spikes_dmg = spikes.total();
    let heal_amount = heal_on_attacked.amount;

    for (entity, transform, mut attack, mut health, bounty, burning) in &mut enemy_query {
        let dist = transform.translation.truncate().length();

        if dist <= attack.range {
            attack.timer -= time.delta_secs();
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
