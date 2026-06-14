use bevy::prelude::*;

use super::super::components::{
    BanishedModifier, Corpse, FlockingVelocity, FrozenSolidModifier, InMelee, Petrified,
    RootedModifier, SickenedModifier, Stunned, TargetingVelocity, Team, TimedModifier,
};
use crate::game::components::{Acceleration, Velocity};
use crate::game::constants::{
    GLOBAL_SPEED_MULTIPLIER, MELEE_SLOWDOWN_DISTANCE, MELEE_SLOWDOWN_FACTOR, STEERING_FORCE,
    VELOCITY_DAMPING,
};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::wizard::components::Wizard;

/// Returns true if the unit is immobilized by any crowd control effect.
/// Centralizes CC checks so new CC types only need updating here.
/// Sleepwalking units (Dreamwalker talent) are NOT immobilized.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn is_cc_immobilized(
    rooted: Option<&RootedModifier>,
    has_sleep: bool,
    has_sleepwalking: bool,
    banished: Option<&BanishedModifier>,
    sickened: Option<&SickenedModifier>,
    frozen: Option<&FrozenSolidModifier>,
    stunned: Option<&Stunned>,
    petrified: Option<&Petrified>,
) -> bool {
    rooted.is_some()
        || (has_sleep && !has_sleepwalking)
        || banished.is_some()
        || sickened.is_some()
        || frozen.is_some()
        || stunned.is_some()
        || petrified.is_some()
}

/// Returns true if the unit is a staging attacker (not yet activated).
/// Handles the 1-frame deferred command delay: units without WaveGroup yet
/// are also considered staging (they haven't been tagged by `tag_new_attackers`).
#[inline]
pub fn is_staging_attacker(team: &Team, has_staging: bool, has_wave_group: bool) -> bool {
    *team == Team::Attackers && (has_staging || !has_wave_group)
}

/// Generic targeting system for melee units.
///
/// Finds the nearest enemy using team-based logic and updates targeting velocity.
/// Also manages the InMelee component based on distance to enemy.
///
/// If `retaliation_target` is `Some(entity)`, that entity is also considered a valid
/// target regardless of team (used when a mind-controlled unit attacks an ally).
#[inline]
pub fn update_melee_unit_targeting(
    unit_snapshot: &[(Entity, Vec3, Team)],
    entity: Entity,
    transform: &Transform,
    team: Team,
    targeting_velocity: &mut TargetingVelocity,
    commands: &mut Commands,
    retaliation_target: Option<Entity>,
) {
    // Find nearest enemy using team-based targeting logic
    let nearest_enemy = unit_snapshot
        .iter()
        .filter(|(other_entity, _, other_team)| {
            *other_entity != entity
                && (retaliation_target == Some(*other_entity) || team.is_enemy(other_team))
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

/// Generic weighted movement system used by infantry, brute, and other melee units.
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
    targeting_velocity: &TargetingVelocity,
    flocking_velocity: &FlockingVelocity,
    flow_field_velocity: &FlowFieldVelocity,
    in_melee: bool,
    commander_aura_modifier: Option<f32>,
    terrain_modifier: Option<f32>,
    slow_modifier: Option<f32>,
    cauldron_modifier: Option<f32>,
    haste_modifier: Option<f32>,
    elite_speed_modifier: Option<f32>,
) {
    // If the unit has reached its flow field destination and has no target,
    // stop entirely so flocking doesn't push it around between waves.
    let no_target = targeting_velocity.velocity.length_squared() < 0.001;
    if flow_field_velocity.at_destination && no_target {
        velocity.x *= VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
        velocity.z *= VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
        return;
    }

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

    // When pathfinding distance is INFINITY, the unit has no valid path to its goal
    // (e.g. completely enclosed by walls). Give targeting full weight so the unit
    // can move toward and attack the nearest wall to break free.
    if distance.is_infinite() && targeting_velocity.velocity.length_squared() > 0.001 {
        flow_weight = 0.0;
        flocking_weight = 0.0;
        targeting_weight = 1.0;
    }

    // On hazardous terrain, boost flow field weight so units follow the rerouted path
    // instead of charging through the hazard toward their target
    if !distance.is_infinite() && flow_field_velocity.terrain_cost > 1.0 {
        flow_weight = 0.8;
        flocking_weight = 0.1;
        targeting_weight = 0.1;
    }

    // When pathfinding distance is much larger than straight-line distance to the
    // target, a wall is likely between the unit and its target. Keep flow field
    // weight high so units navigate around the wall instead of pushing into it.
    // Skip when distance is INFINITY — those units are fully blocked and need
    // targeting weight to attack walls.
    if !distance.is_infinite() && targeting_velocity.velocity.length_squared() > 0.001 {
        let straight_line_distance = targeting_velocity.velocity.length();
        if straight_line_distance > 1.0
            && flow_field_velocity.pathfinding_distance > straight_line_distance * 2.0
        {
            flow_weight = flow_weight.max(0.7);
            targeting_weight = targeting_weight.min(0.1);
            flocking_weight = 1.0 - flow_weight - targeting_weight;
        }
    }

    // Combine three velocity sources with distance-based weighting
    let weighted_direction = (flow_field_velocity.velocity * flow_weight
        + flocking_velocity.velocity * flocking_weight
        + targeting_velocity.velocity * targeting_weight)
        .normalize_or_zero();

    // Calculate speed modifiers
    let aura_percentage = commander_aura_modifier.unwrap_or(0.0);
    let terrain_percentage = terrain_modifier.unwrap_or(0.0);
    let slow_percentage = slow_modifier.unwrap_or(0.0);
    let cauldron_percentage = cauldron_modifier.unwrap_or(0.0);
    let haste_percentage = haste_modifier.unwrap_or(0.0);
    let elite_speed_percentage = elite_speed_modifier.unwrap_or(0.0);
    let total_percentage = aura_percentage
        + terrain_percentage
        + slow_percentage
        + cauldron_percentage
        + haste_percentage
        + elite_speed_percentage;
    let speed_multiplier = (1.0 + total_percentage).max(0.0); // Clamp to prevent negative speed

    // Calculate max speed with modifiers and melee slowdown
    let mut max_speed = movement_speed * GLOBAL_SPEED_MULTIPLIER * speed_multiplier;
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

    // Apply steering force, scaled by max_speed so faster units accelerate proportionally.
    // Without this, the fixed STEERING_FORCE creates an equilibrium speed (due to damping)
    // that caps all units at the same effective speed regardless of movement_speed.
    let speed_scale = max_speed / (200.0 * GLOBAL_SPEED_MULTIPLIER);
    let steering = velocity_change_needed.normalize_or_zero()
        * STEERING_FORCE
        * speed_multiplier
        * speed_scale;
    let steering_magnitude = steering.length();
    let max_steering = velocity_change_needed.length() / time.delta_secs();

    let final_steering = if steering_magnitude > max_steering && max_steering > 0.0 {
        steering.normalize() * max_steering
    } else {
        steering
    };

    acceleration.add_force(final_steering);

    // Apply smelly repulsion directly as a strong force, bypassing normal movement weighting
    if flocking_velocity.smelly_repulsion.length_squared() > 0.001 {
        let smelly_force = flocking_velocity.smelly_repulsion * max_speed;
        acceleration.add_force(smelly_force);
    }

    // Store max_speed for velocity clamping in apply_unit_movement
    velocity.max_speed = max_speed;

    // Apply damping to current velocity (allows external forces like black hole gravity)
    // Frame-rate independent: normalize to 60 FPS reference rate
    let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
    velocity.x *= damping;
    velocity.z *= damping;
}

/// Generic system that ticks all instances of a `TimedModifier` component and removes expired ones.
pub fn update_timed_modifier<
    T: Component<Mutability = bevy::ecs::component::Mutable> + TimedModifier,
>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut T)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier) in query.iter_mut() {
        if modifier.tick(delta) {
            commands.entity(entity).remove::<T>();
        }
    }
}

/// Finds the closest enemy within range of a position. Shared by brute and ogre rock throw.
pub fn find_closest_enemy_in_range(
    origin: Vec3,
    team: &Team,
    range: f32,
    targets: &Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<crate::game::pathfinding::StagingAttacker>,
            Without<Wizard>,
        ),
    >,
) -> Option<Vec3> {
    let mut best: Option<(Vec3, f32)> = None;
    for (target_transform, target_team) in targets.iter() {
        if !team.is_enemy(target_team) {
            continue;
        }
        let dx = target_transform.translation.x - origin.x;
        let dz = target_transform.translation.z - origin.z;
        let distance = (dx * dx + dz * dz).sqrt();
        if distance > range {
            continue;
        }
        if best.is_none_or(|(_, d)| distance < d) {
            best = Some((target_transform.translation, distance));
        }
    }
    best.map(|(pos, _)| pos)
}
