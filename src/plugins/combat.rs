use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::components::combat::*;
use crate::components::enemy::*;
use crate::components::run::GameState;
use crate::components::tower::Tower;
use crate::data::damage_matrix::DamageMatrix;
use crate::components::economy::Gold;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_message::<EnemyKilledEvent>()
            .add_systems(Update, (
                fire_weapons.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Boss)),
                ),
                move_projectiles.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Boss)),
                ),
                apply_damage_to_enemies.run_if(
                    in_state(GameState::Playing)
                        .or(in_state(GameState::Boss)),
                ),
                animate_attack_vfx,
                draw_attack_vfx_gizmos,
                animate_cone_vfx,
                draw_cone_gizmos,
            ));
    }
}


fn fire_weapons(
    time: Res<Time>,
    mut weapon_query: Query<&mut WeaponInstance>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut tower_query: Query<(&Transform, &mut crate::components::tower::Health), With<Tower>>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    let Ok((tower_transform, mut tower_health)) = tower_query.single_mut() else {
        return;
    };
    let tower_pos = tower_transform.translation.truncate();

    let mut rng = rand::rng();

    // Collect enemies in a vec for random selection
    let enemies: Vec<(Entity, Vec2)> = enemy_query
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    if enemies.is_empty() {
        return;
    }

    for mut weapon in &mut weapon_query {
        weapon.cooldown_timer -= time.delta_secs();

        if weapon.cooldown_timer <= 0.0 {
            // Find enemies in range
            let in_range: Vec<(Entity, Vec2)> = enemies
                .iter()
                .filter(|(_, pos)| pos.distance(tower_pos) <= weapon.range)
                .copied()
                .collect();

            if in_range.is_empty() {
                continue;
            }

            weapon.cooldown_timer = weapon.attack_cooldown;

            match &weapon.attack_pattern {
                AttackPattern::SingleTarget | AttackPattern::Splash { .. } | AttackPattern::Bounce { .. } => {
                    let idx = rng.random_range(0..in_range.len());
                    let (target, target_pos) = in_range[idx];
                    spawn_projectile(
                        &mut commands,
                        tower_pos,
                        target,
                        target_pos,
                        weapon.damage,
                        weapon.damage_type,
                        weapon.attack_pattern.clone(),
                        &weapon.name,
                        weapon.applies_frost,
                    );
                }
                AttackPattern::Barrage { target_count } => {
                    // Barrage fires 1 projectile at each of N different enemies
                    let count = (*target_count as usize).min(in_range.len());
                    let mut indices: Vec<usize> = (0..in_range.len()).collect();
                    for i in 0..count {
                        let pick = rng.random_range(i..indices.len());
                        indices.swap(i, pick);
                        let (target, target_pos) = in_range[indices[i]];
                        spawn_projectile(
                            &mut commands,
                            tower_pos,
                            target,
                            target_pos,
                            weapon.damage,
                            weapon.damage_type,
                            AttackPattern::SingleTarget,
                            &weapon.name,
                            weapon.applies_frost,
                        );
                    }
                }
                AttackPattern::Area { radius } => {
                    // radius=0 means "Enemies in Range" — use weapon range, centered on tower
                    let (effective_radius, center) = if *radius <= 0.0 {
                        (weapon.range, tower_pos)
                    } else {
                        let idx = rng.random_range(0..in_range.len());
                        (*radius, in_range[idx].1)
                    };
                    let mut is_first = true;
                    for (target_entity, target_pos) in &in_range {
                        if target_pos.distance(center) <= effective_radius {
                            damage_writer.write(DamageEvent {
                                target: *target_entity,
                                damage: weapon.damage,
                                damage_type: weapon.damage_type,
                                position: *target_pos,
                                attack_pattern: weapon.attack_pattern.clone(),
                                is_primary_hit: is_first,
                                applies_frost: weapon.applies_frost,
                            });
                            is_first = false;
                        }
                    }
                    // Spawn area VFX at impact center — expanding circle
                    spawn_attack_vfx(&mut commands, center, effective_radius, weapon.damage_type, 0.3);
                }
                AttackPattern::Wave { bonus_range } => {
                    let wave_range = weapon.range + bonus_range;
                    // Pick a random target to aim the cone at
                    let idx = rng.random_range(0..in_range.len());
                    let (_, aim_pos) = in_range[idx];
                    let cone_dir = (aim_pos - tower_pos).normalize_or_zero();
                    let cone_half_angle = 0.4; // ~23 degrees each side, ~46 degree cone

                    let mut is_first = true;
                    for (target_entity, target_pos) in &in_range {
                        let to_enemy = (*target_pos - tower_pos).normalize_or_zero();
                        let dot = cone_dir.dot(to_enemy);
                        let dist = target_pos.distance(tower_pos);
                        // Enemy must be within cone angle AND within range
                        if dot >= (1.0 - cone_half_angle) && dist <= wave_range {
                            damage_writer.write(DamageEvent {
                                target: *target_entity,
                                damage: weapon.damage,
                                damage_type: weapon.damage_type,
                                position: *target_pos,
                                attack_pattern: weapon.attack_pattern.clone(),
                                is_primary_hit: is_first,
                                applies_frost: weapon.applies_frost,
                            });
                            is_first = false;
                        }
                    }
                    // Spawn cone VFX — expands outward from tower in cone direction
                    spawn_cone_vfx(
                        &mut commands,
                        tower_pos,
                        cone_dir,
                        cone_half_angle,
                        wave_range,
                        weapon.damage_type,
                        0.5,
                    );
                }
            }

            // Blood Bomb: grant Max HP once per attack (only this weapon triggers it)
            if weapon.max_hp_per_attack > 0.0 {
                tower_health.max += weapon.max_hp_per_attack;
                tower_health.current += weapon.max_hp_per_attack;
            }
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    origin: Vec2,
    target: Entity,
    target_pos: Vec2,
    damage: f32,
    damage_type: DamageType,
    attack_pattern: AttackPattern,
    weapon_name: &str,
    applies_frost: bool,
) {
    // Color projectiles by damage type for visual clarity
    let color = damage_type_color_opaque(damage_type);

    // Size projectiles by attack pattern
    let size = match &attack_pattern {
        AttackPattern::SingleTarget => 6.0,
        AttackPattern::Splash { .. } => 9.0,
        AttackPattern::Bounce { .. } => 8.0,
        AttackPattern::Barrage { .. } => 5.0,
        _ => 7.0,
    };

    commands.spawn((
        Projectile,
        ProjectileData {
            target: Some(target),
            target_position: target_pos,
            speed: 600.0,
            damage,
            damage_type,
            attack_pattern,
            source_weapon: weapon_name.to_string(),
            applies_frost,
            hits: Vec::new(),
        },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(origin.x, origin.y, 0.8),
    ));
}

/// Spawn a barrage projectile that targets a ground position (no entity target).
/// On arrival it deals splash damage to all enemies within splash_radius.
fn spawn_barrage_projectile(
    commands: &mut Commands,
    origin: Vec2,
    land_pos: Vec2,
    damage: f32,
    damage_type: DamageType,
    splash_radius: f32,
    weapon_name: &str,
    applies_frost: bool,
) {
    let color = damage_type_color_opaque(damage_type);

    commands.spawn((
        Projectile,
        ProjectileData {
            target: None, // Ground-targeted, no live entity
            target_position: land_pos,
            speed: 500.0,
            damage,
            damage_type,
            attack_pattern: AttackPattern::Barrage { target_count: 1 },
            source_weapon: weapon_name.to_string(),
            applies_frost,
            hits: Vec::new(),
        },
        BarrageSplash {
            splash_radius,
        },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(5.0)),
            ..default()
        },
        Transform::from_xyz(origin.x, origin.y, 0.8),
    ));
}

/// Opaque color for projectile sprites (no alpha).
fn damage_type_color_opaque(dt: DamageType) -> Color {
    match dt {
        DamageType::Normal   => Color::srgb(0.9, 0.9, 0.9), // white
        DamageType::Piercing => Color::srgb(0.3, 1.0, 0.3), // green
        DamageType::Siege    => Color::srgb(1.0, 0.5, 0.2), // orange
        DamageType::Magic    => Color::srgb(0.5, 0.3, 1.0), // purple
        DamageType::Chaos    => Color::srgb(1.0, 0.1, 0.3), // red
    }
}

/// Semi-transparent color for VFX overlays.
fn damage_type_color(dt: DamageType) -> Color {
    match dt {
        DamageType::Normal   => Color::srgba(0.9, 0.9, 0.9, 0.6),
        DamageType::Piercing => Color::srgba(0.3, 1.0, 0.3, 0.6),
        DamageType::Siege    => Color::srgba(1.0, 0.5, 0.2, 0.6),
        DamageType::Magic    => Color::srgba(0.5, 0.3, 1.0, 0.6),
        DamageType::Chaos    => Color::srgba(1.0, 0.1, 0.3, 0.6),
    }
}

fn spawn_attack_vfx(
    commands: &mut Commands,
    center: Vec2,
    radius: f32,
    damage_type: DamageType,
    duration: f32,
) {
    commands.spawn((
        AttackVfx {
            lifetime: duration,
            max_lifetime: duration,
            start_radius: 10.0,
            end_radius: radius,
            center,
            damage_type,
        },
        Transform::from_xyz(center.x, center.y, 0.6),
        Visibility::default(),
    ));
}

/// Spawn a cone-shaped VFX entity. The cone is drawn by gizmos each frame
/// and the entity tracks expansion progress and lifetime.
fn spawn_cone_vfx(
    commands: &mut Commands,
    origin: Vec2,
    direction: Vec2,
    half_angle: f32,
    max_radius: f32,
    damage_type: DamageType,
    duration: f32,
) {
    // Spawn an invisible entity — gizmos handle the rendering
    commands.spawn((
        ConeVfx {
            origin,
            direction,
            half_angle,
            max_radius,
            lifetime: duration,
            max_lifetime: duration,
            damage_type,
        },
        // Need a Transform + Visibility for the entity to exist in the world,
        // but rendering is done via gizmos not sprites.
        Transform::from_xyz(origin.x, origin.y, 0.7),
        Visibility::default(),
    ));
}

fn animate_attack_vfx(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackVfx)>,
) {
    for (entity, mut vfx) in &mut query {
        vfx.lifetime -= time.delta_secs();
        if vfx.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Draw splash/area VFX as expanding hollow circle outlines using gizmos.
fn draw_attack_vfx_gizmos(
    mut gizmos: Gizmos,
    query: Query<&AttackVfx>,
) {
    for vfx in &query {
        let progress = 1.0 - (vfx.lifetime / vfx.max_lifetime);
        let current_radius = vfx.start_radius + (vfx.end_radius - vfx.start_radius) * progress;
        let alpha = (vfx.lifetime / vfx.max_lifetime) * 0.6;

        let color = match vfx.damage_type {
            DamageType::Normal   => Color::srgba(0.9, 0.9, 0.9, alpha),
            DamageType::Piercing => Color::srgba(0.3, 1.0, 0.3, alpha),
            DamageType::Siege    => Color::srgba(1.0, 0.5, 0.2, alpha),
            DamageType::Magic    => Color::srgba(0.5, 0.3, 1.0, alpha),
            DamageType::Chaos    => Color::srgba(1.0, 0.1, 0.3, alpha),
        };

        if current_radius < 1.0 { continue; }

        gizmos.circle_2d(vfx.center, current_radius, color);
    }
}

/// Tick ConeVfx lifetimes and despawn when expired.
fn animate_cone_vfx(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ConeVfx)>,
) {
    for (entity, mut cone) in &mut query {
        cone.lifetime -= time.delta_secs();
        if cone.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Draw all active ConeVfx as gizmo arcs each frame.
/// The cone expands from origin outward over its lifetime.
fn draw_cone_gizmos(
    mut gizmos: Gizmos,
    query: Query<&ConeVfx>,
) {
    for cone in &query {
        let progress = 1.0 - (cone.lifetime / cone.max_lifetime);
        let current_radius = cone.max_radius * progress;
        let alpha = (cone.lifetime / cone.max_lifetime) * 0.5;

        let color = match cone.damage_type {
            DamageType::Normal   => Color::srgba(0.9, 0.9, 0.9, alpha),
            DamageType::Piercing => Color::srgba(0.3, 1.0, 0.3, alpha),
            DamageType::Siege    => Color::srgba(1.0, 0.5, 0.2, alpha),
            DamageType::Magic    => Color::srgba(0.5, 0.3, 1.0, alpha),
            DamageType::Chaos    => Color::srgba(1.0, 0.1, 0.3, alpha),
        };

        if current_radius < 1.0 {
            continue;
        }

        // Compute the base angle of the direction vector
        let base_angle = cone.direction.y.atan2(cone.direction.x);

        // Draw the two edge lines of the cone
        let left_angle = base_angle + cone.half_angle;
        let right_angle = base_angle - cone.half_angle;

        let left_end = cone.origin + Vec2::new(left_angle.cos(), left_angle.sin()) * current_radius;
        let right_end = cone.origin + Vec2::new(right_angle.cos(), right_angle.sin()) * current_radius;

        gizmos.line_2d(cone.origin, left_end, color);
        gizmos.line_2d(cone.origin, right_end, color);

        // Draw arc segments along the outer edge of the cone
        let arc_segments = 12;
        let angle_step = (cone.half_angle * 2.0) / arc_segments as f32;
        for i in 0..arc_segments {
            let a1 = right_angle + angle_step * i as f32;
            let a2 = right_angle + angle_step * (i + 1) as f32;
            let p1 = cone.origin + Vec2::new(a1.cos(), a1.sin()) * current_radius;
            let p2 = cone.origin + Vec2::new(a2.cos(), a2.sin()) * current_radius;
            gizmos.line_2d(p1, p2, color);
        }

        // Draw a few intermediate arcs for a filled look
        for ring in 1..4 {
            let ring_radius = current_radius * (ring as f32 / 4.0);
            let ring_alpha = alpha * (1.0 - ring as f32 / 4.0);
            let ring_color = match cone.damage_type {
                DamageType::Normal   => Color::srgba(0.9, 0.9, 0.9, ring_alpha),
                DamageType::Piercing => Color::srgba(0.3, 1.0, 0.3, ring_alpha),
                DamageType::Siege    => Color::srgba(1.0, 0.5, 0.2, ring_alpha),
                DamageType::Magic    => Color::srgba(0.5, 0.3, 1.0, ring_alpha),
                DamageType::Chaos    => Color::srgba(1.0, 0.1, 0.3, ring_alpha),
            };
            for i in 0..arc_segments {
                let a1 = right_angle + angle_step * i as f32;
                let a2 = right_angle + angle_step * (i + 1) as f32;
                let p1 = cone.origin + Vec2::new(a1.cos(), a1.sin()) * ring_radius;
                let p2 = cone.origin + Vec2::new(a2.cos(), a2.sin()) * ring_radius;
                gizmos.line_2d(p1, p2, ring_color);
            }
        }
    }
}

fn move_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut proj_query: Query<(Entity, &mut Transform, &mut ProjectileData, Option<&BarrageSplash>), With<Projectile>>,
    enemy_query: Query<(Entity, &Transform), (With<Enemy>, Without<Projectile>)>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    for (proj_entity, mut proj_transform, mut data, barrage_splash) in &mut proj_query {
        // Update target position if target is still alive
        let target_alive = if let Some(target_entity) = data.target {
            if let Ok((_, target_transform)) = enemy_query.get(target_entity) {
                data.target_position = target_transform.translation.truncate();
                true
            } else {
                // Target died -- clear entity ref, keep last known position
                data.target = None;
                false
            }
        } else {
            false
        };

        // Move toward target position (alive or last known)
        let proj_pos = proj_transform.translation.truncate();
        let direction = (data.target_position - proj_pos).normalize_or_zero();
        let movement = direction * data.speed * time.delta_secs();
        proj_transform.translation.x += movement.x;
        proj_transform.translation.y += movement.y;

        // Check arrival
        let new_dist = proj_transform.translation.truncate().distance(data.target_position);
        if new_dist < 10.0 {
            let hit_pos = data.target_position;

            // Handle barrage splash projectiles (ground-targeted, splash on landing)
            if let Some(splash) = barrage_splash {
                let splash_radius = splash.splash_radius;
                let current_damage = data.damage;
                let current_damage_type = data.damage_type;

                // Damage all enemies within splash radius of landing point
                let mut is_first = true;
                for (enemy_entity, enemy_transform) in &enemy_query {
                    let enemy_pos = enemy_transform.translation.truncate();
                    if enemy_pos.distance(hit_pos) <= splash_radius {
                        damage_writer.write(DamageEvent {
                            target: enemy_entity,
                            damage: current_damage,
                            damage_type: current_damage_type,
                            position: enemy_pos,
                            attack_pattern: AttackPattern::SingleTarget,
                            is_primary_hit: is_first,
                            applies_frost: data.applies_frost,
                        });
                        is_first = false;
                    }
                }
                // Spawn a small splash VFX at the landing point
                spawn_attack_vfx(&mut commands, hit_pos, splash_radius * 0.5, data.damage_type, 0.2);
                commands.entity(proj_entity).despawn();
                continue;
            }

            if target_alive {
                let current_target = data.target.unwrap();
                let current_damage = data.damage;
                let current_damage_type = data.damage_type;
                let current_pattern = data.attack_pattern.clone();

                // Deal damage to primary target
                damage_writer.write(DamageEvent {
                    target: current_target,
                    damage: current_damage,
                    damage_type: current_damage_type,
                    position: hit_pos,
                    attack_pattern: current_pattern.clone(),
                    is_primary_hit: true,
                    applies_frost: data.applies_frost,
                });
                data.hits.push(current_target);

                // Handle splash -- damage all nearby enemies
                if let AttackPattern::Splash { radius } = &current_pattern {
                    let splash_radius = radius.min(150.0); // Cap splash to 150
                    for (enemy_entity, enemy_transform) in &enemy_query {
                        if enemy_entity == current_target {
                            continue;
                        }
                        let enemy_pos = enemy_transform.translation.truncate();
                        if enemy_pos.distance(hit_pos) <= splash_radius {
                            damage_writer.write(DamageEvent {
                                target: enemy_entity,
                                damage: current_damage,
                                damage_type: current_damage_type,
                                position: enemy_pos,
                                attack_pattern: AttackPattern::SingleTarget,
                                is_primary_hit: false,
                                applies_frost: data.applies_frost,
                            });
                        }
                    }
                    // Splash VFX at impact
                    spawn_attack_vfx(&mut commands, hit_pos, splash_radius, current_damage_type, 0.3);
                }

                // Handle bounce -- find next target and redirect
                if let AttackPattern::Bounce { max_targets } = &current_pattern {
                    if data.hits.len() < *max_targets as usize {
                        let mut best: Option<(Entity, Vec2, f32)> = None;
                        for (enemy_entity, enemy_transform) in &enemy_query {
                            if data.hits.contains(&enemy_entity) {
                                continue;
                            }
                            let enemy_pos = enemy_transform.translation.truncate();
                            let dist = enemy_pos.distance(hit_pos);
                            if dist < 300.0 { // Bounce range: 300 units
                                if best.is_none() || dist < best.unwrap().2 {
                                    best = Some((enemy_entity, enemy_pos, dist));
                                }
                            }
                        }
                        if let Some((next_target, next_pos, _)) = best {
                            data.target = Some(next_target);
                            data.target_position = next_pos;
                            continue; // Don't despawn -- keep bouncing
                        }
                    }
                }
            }
            // else: target was dead, projectile arrived at last known position -- no damage

            // Despawn projectile (no more bounces, or target dead, or single hit)
            commands.entity(proj_entity).despawn();
        }
    }
}

fn apply_damage_to_enemies(
    mut commands: Commands,
    mut events: MessageReader<DamageEvent>,
    mut enemy_query: Query<(&mut EnemyHealth, &ArmorType, &EnemyArmor, &Transform, &GoldBounty, &Burning, &mut FrostStacks), With<Enemy>>,
    damage_matrix: Res<DamageMatrix>,
    mut killed_writer: MessageWriter<EnemyKilledEvent>,
    mut gold: ResMut<Gold>,
    mut tower_query: Query<(
        &mut crate::components::tower::Health,
        &mut crate::components::tower::ManaShield,
        &crate::components::tower::ManaShieldOnHit,
        &crate::components::tower::HealPerHit,
    ), With<Tower>>,
) {
    let Ok((mut tower_health, mut tower_shield, shield_on_hit, heal_per_hit)) = tower_query.single_mut() else {
        return;
    };

    let mut hits_count = 0u32;

    for event in events.read() {
        let Ok((mut health, armor_type, enemy_armor, transform, bounty, burning, mut frost)) = enemy_query.get_mut(event.target) else {
            continue;
        };

        // Apply damage matrix multiplier, then enemy armor reduction
        let multiplier = damage_matrix.get_multiplier(event.damage_type, *armor_type);
        let after_matrix = event.damage * multiplier;
        let final_damage = enemy_armor.apply(after_matrix).max(1.0);

        health.current -= final_damage;
        hits_count += 1;

        // Apply frost slow (3 second duration, refreshes on each hit)
        if event.applies_frost {
            frost.frozen = true;
            frost.freeze_timer = 3.0;
        }

        if health.current <= 0.0 {
            // Award bounty
            let bounty_amount = ((bounty.base as f32) * (1.0 + gold.bounty_bonus_percent)) as u32;
            gold.current += bounty_amount;
            gold.total_earned += bounty_amount;

            let pos = transform.translation.truncate();

            killed_writer.write(EnemyKilledEvent {
                position: pos,
                gold_bounty: bounty_amount,
                had_burning: burning.active,
                fire_damage: burning.fire_damage,
            });

            commands.entity(event.target).despawn();
        }
    }

    // Apply on-hit effects based on total hits this frame
    if hits_count > 0 {
        // Mana Shield on hit (Soulstealer: +3 per hit — grows max and current)
        if shield_on_hit.amount > 0.0 {
            let gain = shield_on_hit.amount * hits_count as f32;
            tower_shield.max += gain;
            tower_shield.current += gain;
        }
        // HP on hit (Healing Sprayer, Holy Bolt, Chain Heal, Chaos Swarm)
        if heal_per_hit.amount > 0.0 {
            let heal = heal_per_hit.amount * hits_count as f32;
            tower_health.current = (tower_health.current + heal).min(tower_health.max);
        }
    }
}
