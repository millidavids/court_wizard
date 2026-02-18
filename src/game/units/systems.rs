use bevy::prelude::*;
use rand::Rng;

use super::components::{
    Corpse, Effectiveness, ElectricCharge, FireDoT, FlockingVelocity, FrostSlowModifier,
    HasteModifier, Health, InMelee, OriginalMaterial, PendingDamageEffect, RootedModifier,
    SpikeGrowthSlowModifier, TargetingVelocity, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use super::constants::{
    ELECTRIC_ARC_COLOR, ELECTRIC_ARC_DAMAGE, ELECTRIC_ARC_LIFETIME, ELECTRIC_ARC_MAX_TARGETS,
    ELECTRIC_ARC_RANGE, ELECTRIC_ARC_WIDTH, ELECTRIC_EFFECT_COLOR, ELECTRIC_EFFECT_FLICKER_SPEED,
    ELECTRIC_EFFECT_MAX_INTENSITY, ELECTRIC_EFFECT_MIN_INTENSITY, FIRE_EFFECT_COLOR,
    FIRE_EFFECT_MAX_INTENSITY, FIRE_EFFECT_MIN_INTENSITY, FIRE_EFFECT_PULSE_SPEED,
    FROST_EFFECT_COLOR, FROST_EFFECT_INTENSITY, FROST_SLOW_DURATION, FROST_SLOW_PER_STACK,
};
use super::damage::DamageType;
use crate::game::components::{Acceleration, OnGameplayScreen, Velocity};
use crate::game::constants::{
    GLOBAL_SPEED_MULTIPLIER, MELEE_SLOWDOWN_DISTANCE, MELEE_SLOWDOWN_FACTOR, STEERING_FORCE,
    VELOCITY_DAMPING,
};
use crate::game::pathfinding::FlowFieldVelocity;

/// Generic targeting system for melee units.
///
/// Finds the nearest enemy using team-based logic and updates targeting velocity.
/// Also manages the InMelee component based on distance to enemy.
///
/// # Parameters
/// - `unit_snapshot`: Pre-collected snapshot of all unit positions (entity, pos, team)
/// - `entity`: The entity being updated
/// - `transform`: The unit's transform
/// - `team`: The unit's team
/// - `targeting_velocity`: Mutable targeting velocity to update
/// - `commands`: Commands to insert/remove InMelee component
#[inline]
pub fn update_melee_unit_targeting(
    unit_snapshot: &[(Entity, Vec3, Team)],
    entity: Entity,
    transform: &Transform,
    team: Team,
    targeting_velocity: &mut TargetingVelocity,
    commands: &mut Commands,
) {
    // Find nearest enemy using team-based targeting logic
    let nearest_enemy = unit_snapshot
        .iter()
        .filter(|(other_entity, _, other_team)| {
            *other_entity != entity
                && match (team, other_team) {
                    (Team::Undead, Team::Undead) => false,
                    (Team::Undead, _) => true,
                    (_, Team::Undead) => true,
                    _ => *other_team != team,
                }
        })
        .min_by(|a, b| {
            let dist_a = (transform.translation.x - a.1.x).powi(2)
                + (transform.translation.z - a.1.z).powi(2);
            let dist_b = (transform.translation.x - b.1.x).powi(2)
                + (transform.translation.z - b.1.z).powi(2);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    // Set targeting velocity toward target
    if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
        let direction = (target_pos - transform.translation).normalize_or_zero();
        targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);

        // Store distance for formation weighting (XZ plane only)
        let dx = transform.translation.x - target_pos.x;
        let dz = transform.translation.z - target_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();
        targeting_velocity.distance_to_target = distance;

        // Check if enemy is in melee range
        if distance < MELEE_SLOWDOWN_DISTANCE {
            commands.entity(entity).insert(InMelee(enemy_team));
        } else {
            commands.entity(entity).remove::<InMelee>();
        }
    } else {
        // No enemies found, clear targeting
        targeting_velocity.velocity = Vec3::ZERO;
        targeting_velocity.distance_to_target = f32::MAX;
        commands.entity(entity).remove::<InMelee>();
    }
}

/// Generic weighted movement system used by infantry, behemoth, and other melee units.
///
/// Combines three velocity sources with distance-based weighting:
/// - Flow field: Pathfinding around obstacles
/// - Flocking: Separation from nearby allies
/// - Targeting: Direct movement toward/away from enemies
///
/// This function implements the core movement logic and returns the final steering force.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn calculate_weighted_movement(
    time: &Time,
    velocity: &mut Velocity,
    acceleration: &mut Acceleration,
    movement_speed: f32,
    effectiveness: &Effectiveness,
    targeting_velocity: &TargetingVelocity,
    flocking_velocity: &FlockingVelocity,
    flow_field_velocity: &FlowFieldVelocity,
    in_melee: bool,
    commander_aura_modifier: Option<f32>,
    terrain_modifier: Option<f32>,
    frost_modifier: Option<f32>,
    spike_growth_modifier: Option<f32>,
    cauldron_modifier: Option<f32>,
    haste_modifier: Option<f32>,
    elite_speed_modifier: Option<f32>,
) {
    // Use pathfinding distance (accounts for obstacles)
    let distance = flow_field_velocity.pathfinding_distance;

    // Distance-based weighting with interpolation
    // Far: prioritize pathfinding, Medium: balanced, Close: prioritize targeting
    let (mut flow_weight, mut flocking_weight, mut targeting_weight) = if distance > 500.0 {
        (0.7, 0.2, 0.1)
    } else if distance > 200.0 {
        // Interpolate between far and medium
        let t = (500.0 - distance) / 300.0;
        let flow = 0.7 - (0.2 * t);
        let targeting = 0.1 + (0.2 * t);
        (flow, 0.2, targeting)
    } else if distance > 50.0 {
        // Interpolate between medium and close
        let t = (200.0 - distance) / 150.0;
        let flow = 0.5 - (0.3 * t);
        let targeting = 0.3 + (0.3 * t);
        (flow, 0.2, targeting)
    } else {
        // In melee range
        (0.1, 0.1, 0.8)
    };

    // On hazardous terrain, boost flow field weight so units follow the rerouted path
    // instead of charging through the hazard toward their target
    if flow_field_velocity.terrain_cost > 1.0 {
        flow_weight = 0.8;
        flocking_weight = 0.1;
        targeting_weight = 0.1;
    }

    // Combine three velocity sources with distance-based weighting
    let weighted_direction = (flow_field_velocity.velocity * flow_weight
        + flocking_velocity.velocity * flocking_weight
        + targeting_velocity.velocity * targeting_weight)
        .normalize_or_zero();

    // Calculate speed modifiers
    let aura_percentage = commander_aura_modifier.unwrap_or(0.0);
    let terrain_percentage = terrain_modifier.unwrap_or(0.0);
    let frost_percentage = frost_modifier.unwrap_or(0.0);
    let spike_growth_percentage = spike_growth_modifier.unwrap_or(0.0);
    let cauldron_percentage = cauldron_modifier.unwrap_or(0.0);
    let haste_percentage = haste_modifier.unwrap_or(0.0);
    let elite_speed_percentage = elite_speed_modifier.unwrap_or(0.0);
    let total_percentage = aura_percentage
        + terrain_percentage
        + frost_percentage
        + spike_growth_percentage
        + cauldron_percentage
        + haste_percentage
        + elite_speed_percentage;
    let speed_multiplier = (1.0 + total_percentage).max(0.0); // Clamp to prevent negative speed

    // Calculate max speed with effectiveness, modifiers, and melee slowdown
    let mut max_speed =
        movement_speed * GLOBAL_SPEED_MULTIPLIER * effectiveness.multiplier() * speed_multiplier;
    if in_melee {
        max_speed *= MELEE_SLOWDOWN_FACTOR;
    }

    // Calculate steering force with clamping to prevent overshooting
    let desired_velocity = weighted_direction * max_speed;
    let velocity_change_needed = Vec3::new(
        desired_velocity.x - velocity.x,
        0.0,
        desired_velocity.z - velocity.z,
    );

    // Apply steering force, clamped to achieve max_speed over time without overshooting
    let steering = velocity_change_needed.normalize_or_zero() * STEERING_FORCE * speed_multiplier;
    let steering_magnitude = steering.length();
    let max_steering = velocity_change_needed.length() / time.delta_secs();

    let final_steering = if steering_magnitude > max_steering && max_steering > 0.0 {
        steering.normalize() * max_steering
    } else {
        steering
    };

    acceleration.add_force(final_steering);

    // Apply damping to current velocity (allows external forces like black hole gravity)
    velocity.x *= VELOCITY_DAMPING;
    velocity.z *= VELOCITY_DAMPING;
}

/// Updates all temporary hit points timers and removes expired components.
///
/// This system runs each frame to:
/// - Decrement time_remaining on all TemporaryHitPoints components
/// - Remove components that have expired (time <= 0 or amount <= 0)
pub fn update_temporary_hit_points(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TemporaryHitPoints)>,
) {
    let delta = time.delta_secs();

    for (entity, mut temp_hp) in query.iter_mut() {
        if temp_hp.update(delta) {
            // Temp HP has expired, remove the component
            commands.entity(entity).remove::<TemporaryHitPoints>();
        }
    }
}

/// Updates all frost slow modifiers and removes expired components.
///
/// This system runs each frame to:
/// - Decrement time_remaining on all FrostSlowModifier components
/// - Remove components that have expired (time <= 0)
pub fn update_frost_slow_modifiers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FrostSlowModifier)>,
) {
    let delta = time.delta_secs();

    for (entity, mut frost_slow) in query.iter_mut() {
        if frost_slow.update(delta) {
            commands.entity(entity).remove::<FrostSlowModifier>();
        }
    }
}

/// Updates all rooted modifiers and removes expired components.
pub fn update_rooted_modifiers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RootedModifier)>,
) {
    let delta = time.delta_secs();

    for (entity, mut rooted) in query.iter_mut() {
        if rooted.update(delta) {
            commands.entity(entity).remove::<RootedModifier>();
        }
    }
}

/// Updates all haste modifiers and removes expired components.
pub fn update_haste_modifiers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HasteModifier)>,
) {
    let delta = time.delta_secs();

    for (entity, mut haste) in query.iter_mut() {
        if haste.update(delta) {
            commands.entity(entity).remove::<HasteModifier>();
        }
    }
}

/// Updates all spike growth slow modifiers and removes expired components.
pub fn update_spike_growth_slow_modifiers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpikeGrowthSlowModifier)>,
) {
    let delta = time.delta_secs();

    for (entity, mut slow) in query.iter_mut() {
        if slow.update(delta) {
            commands.entity(entity).remove::<SpikeGrowthSlowModifier>();
        }
    }
}

/// Processes `PendingDamageEffect` markers and creates/stacks persistent effects.
///
/// Each frame, reads all PendingDamageEffect components, determines the damage type,
/// and either creates a new persistent effect component or stacks onto an existing one.
pub fn process_pending_damage_effects(
    mut commands: Commands,
    pending_query: Query<(Entity, &PendingDamageEffect)>,
    mut fire_query: Query<&mut FireDoT>,
    mut frost_query: Query<&mut FrostSlowModifier>,
    mut electric_query: Query<&mut ElectricCharge>,
) {
    for (entity, pending) in pending_query.iter() {
        match pending.damage_type {
            DamageType::Fire => {
                if let Ok(mut fire_dot) = fire_query.get_mut(entity) {
                    fire_dot.stack(pending.damage);
                } else {
                    commands.entity(entity).insert(FireDoT::new(pending.damage));
                }
            }
            DamageType::Frost => {
                if let Ok(mut frost_slow) = frost_query.get_mut(entity) {
                    frost_slow.stack(FROST_SLOW_PER_STACK, FROST_SLOW_DURATION);
                } else {
                    commands.entity(entity).insert(FrostSlowModifier::new(
                        FROST_SLOW_PER_STACK,
                        FROST_SLOW_DURATION,
                    ));
                }
            }
            DamageType::Electric => {
                if let Ok(mut charge) = electric_query.get_mut(entity) {
                    charge.stack(pending.damage);
                } else {
                    commands
                        .entity(entity)
                        .insert(ElectricCharge::new(pending.damage));
                }
            }
            // Force, Necrotic, Nature — no persistent effect
            _ => {}
        }

        commands.entity(entity).remove::<PendingDamageEffect>();
    }
}

/// Ticks FireDoT damage on affected units and removes expired DoTs.
///
/// DoT damage is applied directly to health (does not trigger more DoT).
pub fn update_fire_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut FireDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut fire_dot, mut health, temp_hp) in query.iter_mut() {
        let (tick_damage, expired) = fire_dot.update(delta);

        if let Some(damage) = tick_damage {
            apply_damage_to_unit(&mut health, temp_hp.map(|t| t.into_inner()), damage);
        }

        if expired {
            commands.entity(entity).remove::<FireDoT>();
        }
    }
}

/// Visual marker for electric arc effects (auto-despawns after lifetime).
#[derive(Component)]
pub struct ElectricArcVisual {
    pub lifetime: f32,
    pub time_alive: f32,
}

/// Ticks ElectricCharge on affected units, rolls for arcs, and spawns arc visuals.
///
/// Arc damage inserts `PendingDamageEffect` on targets so it builds charge on neighbors.
#[allow(clippy::too_many_arguments)]
pub fn update_electric_charge(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut charge_query: Query<(Entity, &mut ElectricCharge, &Transform, &Team), Without<Corpse>>,
    target_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    let mut rng = rand::thread_rng();

    // Collect arc events to process after iteration (avoids borrow conflicts)
    let mut arc_events: Vec<(Vec3, Entity, Vec3)> = Vec::new();

    for (entity, mut charge, transform, _team) in charge_query.iter_mut() {
        let expired = charge.update(delta);
        if expired {
            commands.entity(entity).remove::<ElectricCharge>();
            continue;
        }

        if !charge.can_arc() {
            continue;
        }

        // Roll for arc
        let roll: f32 = rng.r#gen();
        if roll >= charge.arc_chance {
            continue;
        }

        // Find nearby arc targets (any non-corpse unit)
        let source_pos = transform.translation;
        let mut targets: Vec<(Entity, Vec3, f32)> = target_query
            .iter()
            .filter(|(target_entity, _, _)| *target_entity != entity)
            .filter_map(|(target_entity, target_transform, _)| {
                let dist = Vec3::new(
                    source_pos.x - target_transform.translation.x,
                    0.0,
                    source_pos.z - target_transform.translation.z,
                )
                .length();
                if dist <= ELECTRIC_ARC_RANGE {
                    Some((target_entity, target_transform.translation, dist))
                } else {
                    None
                }
            })
            .collect();

        if targets.is_empty() {
            continue;
        }

        // Sort by distance and take up to max targets
        targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        targets.truncate(ELECTRIC_ARC_MAX_TARGETS);

        charge.reset_arc_cooldown();

        for (target_entity, target_pos, _) in &targets {
            arc_events.push((source_pos, *target_entity, *target_pos));
        }
    }

    // Process arc events: deal damage and spawn visuals
    let arc_material = materials.add(StandardMaterial {
        base_color: ELECTRIC_ARC_COLOR,
        unlit: true,
        ..default()
    });
    let arc_mesh = meshes.add(Rectangle::new(ELECTRIC_ARC_WIDTH, ELECTRIC_ARC_WIDTH));

    for (source_pos, target_entity, target_pos) in arc_events {
        // Insert pending damage effect on arc target (builds charge on them too)
        commands.entity(target_entity).insert(PendingDamageEffect {
            damage_type: DamageType::Electric,
            damage: ELECTRIC_ARC_DAMAGE,
        });

        // Spawn arc visual (simple straight line between source and target)
        let midpoint = (source_pos + target_pos) / 2.0;
        let direction = (target_pos - source_pos).normalize_or_zero();
        let length = source_pos.distance(target_pos);
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        commands.spawn((
            ElectricArcVisual {
                lifetime: ELECTRIC_ARC_LIFETIME,
                time_alive: 0.0,
            },
            Mesh3d(arc_mesh.clone()),
            MeshMaterial3d(arc_material.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(1.0, length / ELECTRIC_ARC_WIDTH, 1.0)),
            OnGameplayScreen,
        ));
    }
}

/// Updates and despawns electric arc visuals after their lifetime expires.
pub fn update_electric_arc_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ElectricArcVisual)>,
) {
    let delta = time.delta_secs();

    for (entity, mut arc) in query.iter_mut() {
        arc.time_alive += delta;
        if arc.time_alive >= arc.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Updates visual tinting on units affected by persistent damage effects.
///
/// Three-phase logic per entity:
/// 1. Unit has effects but no OriginalMaterial: clone the material, store original
/// 2. Unit has effects and OriginalMaterial: blend effect colors onto cloned material
/// 3. Unit has OriginalMaterial but no effects: restore original, remove OriginalMaterial
pub fn update_persistent_effect_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&FireDoT>,
            Option<&FrostSlowModifier>,
            Option<&ElectricCharge>,
            Option<&OriginalMaterial>,
        ),
        Or<(
            With<FireDoT>,
            With<FrostSlowModifier>,
            With<ElectricCharge>,
            With<OriginalMaterial>,
        )>,
    >,
) {
    let elapsed = time.elapsed_secs();

    for (entity, material_handle, fire, frost, electric, original_mat) in &query {
        let has_any_effect = fire.is_some() || frost.is_some() || electric.is_some();

        if has_any_effect && original_mat.is_none() {
            // Phase 1: First effect applied — clone the material and store original
            let current_handle = material_handle.0.clone();
            let Some(current_material) = materials.get(&current_handle) else {
                continue;
            };
            let cloned = current_material.clone();
            let cloned_handle = materials.add(cloned);
            commands
                .entity(entity)
                .insert(OriginalMaterial(current_handle));
            commands
                .entity(entity)
                .insert(MeshMaterial3d(cloned_handle));
        } else if has_any_effect {
            // Phase 2: Blend effect colors onto the cloned material
            let original = original_mat.expect("has_any_effect && !is_none checked");
            let Some(original_material) = materials.get(&original.0) else {
                continue;
            };
            let base_linear = original_material.base_color.to_linear();

            let mut result_linear = base_linear;

            if fire.is_some() {
                let pulse = (elapsed * FIRE_EFFECT_PULSE_SPEED).sin() * 0.5 + 0.5;
                let intensity = FIRE_EFFECT_MIN_INTENSITY
                    + pulse * (FIRE_EFFECT_MAX_INTENSITY - FIRE_EFFECT_MIN_INTENSITY);
                let fire_linear = FIRE_EFFECT_COLOR.to_linear();
                result_linear = result_linear.mix(&fire_linear, intensity);
            }

            if frost.is_some() {
                let frost_linear = FROST_EFFECT_COLOR.to_linear();
                result_linear = result_linear.mix(&frost_linear, FROST_EFFECT_INTENSITY);
            }

            if electric.is_some() {
                let flicker = (elapsed * ELECTRIC_EFFECT_FLICKER_SPEED).sin() * 0.5 + 0.5;
                let intensity = ELECTRIC_EFFECT_MIN_INTENSITY
                    + flicker * (ELECTRIC_EFFECT_MAX_INTENSITY - ELECTRIC_EFFECT_MIN_INTENSITY);
                let electric_linear = ELECTRIC_EFFECT_COLOR.to_linear();
                result_linear = result_linear.mix(&electric_linear, intensity);
            }

            if let Some(cloned_material) = materials.get_mut(material_handle) {
                cloned_material.base_color = Color::from(result_linear);
            }
        } else if original_mat.is_some() {
            // Phase 3: All effects expired — restore original material
            let original = original_mat.expect("is_some checked");
            commands
                .entity(entity)
                .insert(MeshMaterial3d(original.0.clone()));
            commands.entity(entity).remove::<OriginalMaterial>();
        }
    }
}
