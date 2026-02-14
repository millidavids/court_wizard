use bevy::prelude::*;

use super::components::{
    Effectiveness, FlockingVelocity, FrostSlowModifier, InMelee, TargetingVelocity, Team,
    TemporaryHitPoints,
};
use crate::game::components::{Acceleration, Velocity};
use crate::game::constants::{
    MELEE_SLOWDOWN_DISTANCE, MELEE_SLOWDOWN_FACTOR, STEERING_FORCE, VELOCITY_DAMPING,
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
    aura_modifier: Option<f32>,
    terrain_modifier: Option<f32>,
    frost_modifier: Option<f32>,
    cauldron_modifier: Option<f32>,
) {
    // Use pathfinding distance (accounts for obstacles)
    let distance = flow_field_velocity.pathfinding_distance;

    // Distance-based weighting with interpolation
    // Far: prioritize pathfinding, Medium: balanced, Close: prioritize targeting
    let (flow_weight, flocking_weight, targeting_weight) = if distance > 500.0 {
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

    // Combine three velocity sources with distance-based weighting
    let weighted_direction = (flow_field_velocity.velocity * flow_weight
        + flocking_velocity.velocity * flocking_weight
        + targeting_velocity.velocity * targeting_weight)
        .normalize_or_zero();

    // Calculate speed modifiers
    let aura_percentage = aura_modifier.unwrap_or(0.0);
    let terrain_percentage = terrain_modifier.unwrap_or(0.0);
    let frost_percentage = frost_modifier.unwrap_or(0.0);
    let cauldron_percentage = cauldron_modifier.unwrap_or(0.0);
    let total_percentage =
        aura_percentage + terrain_percentage + frost_percentage + cauldron_percentage;
    let speed_multiplier = (1.0 + total_percentage).max(0.0); // Clamp to prevent negative speed

    // Calculate max speed with effectiveness, modifiers, and melee slowdown
    let mut max_speed = movement_speed * effectiveness.multiplier() * speed_multiplier;
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
            // Frost slow has expired, remove the component
            commands.entity(entity).remove::<FrostSlowModifier>();
        }
    }
}
