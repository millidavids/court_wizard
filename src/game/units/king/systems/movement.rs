use crate::game::constants::{KINGS_GUARD_COUNT, KINGS_GUARD_ORBIT_RADIUS};
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker};
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, KingsGuard, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team,
};
use crate::game::units::wizard::components::Wizard;

/// Updates King targeting velocity toward nearest enemy.
///
/// The King always moves directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
/// King is gated by the DefendersActivated resource.
#[allow(clippy::type_complexity)]
pub fn update_king_targeting(
    defenders_activated: Res<crate::game::units::infantry::components::DefendersActivated>,
    mut commands: Commands,
    mut king: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&crate::game::units::components::RetaliationTarget>,
        ),
        (With<King>, Without<Corpse>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<crate::game::units::assassin::Assassin>,
            Without<StagingAttacker>,
            Without<crate::game::units::components::Flying>,
            Without<Wizard>,
        ),
    >,
) {
    // Collect snapshot of all unit positions (excludes assassins, staging attackers, and flying units)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update King's targeting velocity
    for (entity, transform, team, mut targeting_velocity, retaliation) in &mut king {
        // Skip inactive King (wait for defenders to activate)
        if !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Use shared melee targeting function
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting_velocity,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// King-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// King slows down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn king_movement(
    time: Res<Time>,
    mut king_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        With<King>,
    >,
) {
    // Process King unit
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, _polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut king_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// King cohesion force system.
///
/// Applies a dynamic cohesion force to defenders, pulling them toward the King.
/// The force strength increases when enemies are near (threatened) and decreases when safe.
/// This is King-specific behavior separate from the generic commander aura system.
///
/// Note: Damage and speed buffs are now handled by the generic commander system.
pub fn king_cohesion_force(
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut defenders: Query<
        (&Transform, &Team, &mut FlockingVelocity),
        (Without<King>, Without<Corpse>),
    >,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // Process each King and apply cohesion to their team's units
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        // Find nearest enemy to this King
        let nearest_enemy_distance = all_units
            .iter()
            .filter(|(_, team)| *team != king_team)
            .map(|(transform, _)| transform.translation.distance(king_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX);

        // Calculate threat level: interpolate between BASE and THREATENED
        let threat_factor = if nearest_enemy_distance > KING_AURA_RADIUS {
            0.0
        } else {
            1.0 - (nearest_enemy_distance / KING_AURA_RADIUS)
        };

        let cohesion_strength =
            KING_COHESION_BASE + (KING_COHESION_THREATENED - KING_COHESION_BASE) * threat_factor;

        // Apply cohesion force to same-team defenders within aura radius
        for (unit_transform, team, mut flocking_velocity) in &mut defenders {
            if team != king_team {
                continue;
            }

            let unit_pos = unit_transform.translation;
            let distance_to_king = unit_pos.distance(king_pos);

            // Check if unit is within aura radius
            if distance_to_king < KING_AURA_RADIUS && distance_to_king > 0.1 {
                // Calculate direction toward King
                let to_king = (king_pos - unit_pos).normalize_or_zero();

                // Add cohesion force to flocking velocity
                // Scale by distance (stronger pull when closer to edge of aura)
                let distance_factor = distance_to_king / KING_AURA_RADIUS;
                let cohesion_force = to_king * cohesion_strength * distance_factor;

                flocking_velocity.velocity += Vec3::new(cohesion_force.x, 0.0, cohesion_force.z);

                // Re-normalize to maintain consistent influence
                flocking_velocity.velocity = flocking_velocity.velocity.normalize_or_zero();
            }
        }
    }
}

/// Snaps King's Guard units to fixed positions around the King each frame.
///
/// Guards orbit the King at a fixed radius. Their positions are set directly
/// rather than using velocity/acceleration, so they stay locked to the King.
/// We also write the per-frame movement delta into `Velocity` so the shared
/// `update_walking_animation` and `update_facing_direction` systems (which
/// query `&Velocity`) match the guard entity and animate it correctly. Without
/// this they'd skip the guard and it would freeze on its idle frame, always
/// facing forward.
pub fn snap_kings_guard_to_king(
    time: Res<Time>,
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut guards: Query<
        (&KingsGuard, &Team, &mut Transform, &mut Velocity),
        (Without<King>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();
    let inv_delta = if delta > 1e-6 { 1.0 / delta } else { 0.0 };
    // Snap each guard to their team's King
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        for (guard, guard_team, mut transform, mut velocity) in &mut guards {
            if guard_team != king_team {
                continue;
            }
            let angle = guard.0 as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
            let new_x = king_pos.x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
            let new_z = king_pos.z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();
            let dx = new_x - transform.translation.x;
            let dz = new_z - transform.translation.z;
            transform.translation.x = new_x;
            transform.translation.z = new_z;
            velocity.x = dx * inv_delta;
            velocity.z = dz * inv_delta;
        }
    }
}
