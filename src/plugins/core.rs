use bevy::prelude::*;

use crate::components::combat::{DamageType, TowerDamageEvent, TowerDestroyedEvent, WeaponInstance};
use crate::components::economy::Gold;
use crate::components::run::GameState;
use crate::components::scaling::*;
use crate::components::stats::DamageBonuses;
use crate::components::tower::*;

/// Arena configuration resource.
#[derive(Resource, Debug)]
pub struct ArenaConfig {
    pub radius: f32,
    pub spawn_ring_radius: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            radius: 1800.0,
            spawn_ring_radius: 2200.0,
        }
    }
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArenaConfig>()
            .init_resource::<PurchasedUpgrades>()
            .init_resource::<ScalingBuffs>()
            .init_resource::<DamageBonuses>()
            .add_message::<TowerDamageEvent>()
            .add_systems(Startup, (setup_camera, setup_tower, setup_arena, setup_range_circles))
            .add_systems(Update, zoom_camera.run_if(run_once))
            .add_systems(Update, (
                apply_damage_to_tower,
                regenerate_hp.run_if(in_state(GameState::Playing).or(in_state(GameState::Boss))),
                regenerate_mana_shield.run_if(in_state(GameState::Playing).or(in_state(GameState::Boss))),
                mana_shield_on_kill.run_if(in_state(GameState::Playing).or(in_state(GameState::Boss))),
                heal_on_kill.run_if(in_state(GameState::Playing).or(in_state(GameState::Boss))),
                tick_scaling_buffs.run_if(in_state(GameState::Playing).or(in_state(GameState::Boss))),
                draw_range_gizmos,
            ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
}

fn zoom_camera(
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
) {
    let Ok(mut projection) = camera_query.single_mut() else { return };
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale = 2.5;
    }
}

fn setup_tower(mut commands: Commands) {
    let tower = commands.spawn((
        Tower,
        Health::default(),
        Armor::default(),
        ManaShield::default(),
        HpRegen::default(),
        FlatDamageReduction::default(),
        SpikesDamage::default(),
        HealOnKill::default(),
        HealOnAttacked::default(),
        Sprite {
            color: Color::srgb(0.6, 0.6, 0.7),
            custom_size: Some(Vec2::new(46.0, 69.0)), // 40x60 * 1.15
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
    )).id();
    // Insert remaining components (split to avoid tuple size limit)
    commands.entity(tower).insert((
        ManaShieldRegen::default(),
        ManaShieldOnKill::default(),
        ManaShieldOnHit::default(),
        HealPerHit::default(),
        MaxHpPerHit::default(),
    ));
    info!("Tower spawned at origin with {} HP", STARTING_HP);
}

fn setup_range_circles() {
    // Range circles are drawn via gizmos in draw_range_gizmos
}

fn draw_range_gizmos(mut gizmos: Gizmos) {
    let ranges = [
        (300.0,  Color::srgba(0.5, 0.5, 0.6, 0.4)),
        (600.0,  Color::srgba(0.5, 0.6, 0.5, 0.35)),
        (900.0,  Color::srgba(0.6, 0.5, 0.5, 0.3)),
        (1200.0, Color::srgba(0.6, 0.6, 0.5, 0.25)),
    ];
    for (radius, color) in ranges {
        gizmos.circle_2d(Vec2::ZERO, radius, color);
    }
}

fn setup_arena(
    mut commands: Commands,
    arena: Res<ArenaConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Arena boundary circle (visual only)
    let arena_mesh = Circle::new(arena.radius);
    commands.spawn((
        Mesh2d(meshes.add(arena_mesh)),
        MeshMaterial2d(materials.add(Color::srgba(0.15, 0.15, 0.2, 0.3))),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
    info!("Arena created with radius {}", arena.radius);
}

fn apply_damage_to_tower(
    mut events: MessageReader<TowerDamageEvent>,
    mut tower_query: Query<(&mut Health, &Armor, &mut ManaShield, &FlatDamageReduction), With<Tower>>,
    mut destroyed_events: MessageWriter<TowerDestroyedEvent>,
) {
    let Ok((mut health, armor, mut mana_shield, flat_reduction)) = tower_query.single_mut() else {
        return;
    };

    for event in events.read() {
        let mut remaining = event.raw_damage;

        // Step 1: Mana Shield absorbs raw damage (before armor)
        if mana_shield.current > 0.0 {
            let absorbed = remaining.min(mana_shield.current);
            mana_shield.current -= absorbed;
            remaining -= absorbed;
        }

        // Step 2: Armor reduction on remainder
        if remaining > 0.0 {
            remaining = armor.apply(remaining);
        }

        // Step 3: Flat damage reduction (cannot reduce below 25%)
        if flat_reduction.value > 0.0 && remaining > 0.0 {
            let floor = remaining * 0.25;
            remaining = (remaining - flat_reduction.value).max(floor);
        }

        // Step 4: Apply to HP
        health.current = (health.current - remaining).max(0.0);

        if health.current <= 0.0 {
            destroyed_events.write(TowerDestroyedEvent);
        }
    }
}

fn regenerate_hp(
    time: Res<Time>,
    mut query: Query<(&mut Health, &HpRegen), With<Tower>>,
) {
    let Ok((mut health, regen)) = query.single_mut() else {
        return;
    };
    if regen.per_second > 0.0 {
        health.current = (health.current + regen.per_second * time.delta_secs()).min(health.max);
    }
}

fn regenerate_mana_shield(
    time: Res<Time>,
    mut query: Query<(&mut ManaShield, &ManaShieldRegen), With<Tower>>,
) {
    let Ok((mut shield, regen)) = query.single_mut() else { return };
    if regen.per_second > 0.0 && shield.max > 0.0 {
        shield.current = (shield.current + regen.per_second * time.delta_secs()).min(shield.max);
    }
}

fn mana_shield_on_kill(
    mut events: MessageReader<crate::components::combat::EnemyKilledEvent>,
    mut query: Query<(&mut ManaShield, &ManaShieldOnKill), With<Tower>>,
) {
    let Ok((mut shield, on_kill)) = query.single_mut() else { return };
    if on_kill.amount <= 0.0 || shield.max <= 0.0 { return; }
    for _event in events.read() {
        shield.current = (shield.current + on_kill.amount).min(shield.max);
    }
}

fn heal_on_kill(
    mut events: MessageReader<crate::components::combat::EnemyKilledEvent>,
    mut query: Query<(&mut Health, &HealOnKill), With<Tower>>,
) {
    let Ok((mut health, on_kill)) = query.single_mut() else { return };
    if on_kill.amount <= 0.0 { return; }
    for _event in events.read() {
        health.current = (health.current + on_kill.amount).min(health.max);
    }
}

fn tick_scaling_buffs(
    time: Res<Time>,
    mut buffs: ResMut<ScalingBuffs>,
    mut tower_query: Query<(&mut Health, &mut Armor, &mut ManaShield, &mut HpRegen, &mut SpikesDamage), With<Tower>>,
    mut gold: ResMut<Gold>,
    mut weapon_query: Query<&mut WeaponInstance>,
) {
    let Ok((mut health, mut armor, mut shield, mut regen, mut spikes)) = tower_query.single_mut() else { return };

    for buff in buffs.buffs.iter_mut() {
        buff.timer -= time.delta_secs();
        if buff.timer <= 0.0 {
            buff.timer += buff.interval;

            match buff.effect {
                ScalingEffect::SpikesDamage => {
                    spikes.flat += buff.amount;
                }
                ScalingEffect::GlobalDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        w.damage *= 1.0 + pct;
                    }
                }
                ScalingEffect::GoldPerSecond => {
                    gold.per_second += buff.amount;
                }
                ScalingEffect::Armor => {
                    armor.value += buff.amount;
                }
                ScalingEffect::MaxHp => {
                    health.max += buff.amount;
                    health.current = (health.current + buff.amount).min(health.max);
                }
                ScalingEffect::HpRegen => {
                    regen.per_second += buff.amount;
                }
                ScalingEffect::ManaShield => {
                    shield.max += buff.amount;
                    shield.current = (shield.current + buff.amount).min(shield.max);
                }
                ScalingEffect::InstantGold => {
                    let amount = buff.amount as u32;
                    gold.current += amount;
                    gold.total_earned += amount;
                }
                ScalingEffect::PiercingDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Piercing) { w.damage *= 1.0 + pct; }
                    }
                }
                ScalingEffect::ChaosDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Chaos) { w.damage *= 1.0 + pct; }
                    }
                }
                ScalingEffect::MagicDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Magic) { w.damage *= 1.0 + pct; }
                    }
                }
                ScalingEffect::SiegeDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Siege) { w.damage *= 1.0 + pct; }
                    }
                }
                ScalingEffect::NormalDamagePercent => {
                    let pct = buff.amount / 100.0;
                    for mut w in &mut weapon_query {
                        if matches!(w.damage_type, DamageType::Normal) { w.damage *= 1.0 + pct; }
                    }
                }
            }
        }
    }
}
