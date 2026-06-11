use bevy::prelude::*;

use super::targeting_helpers::{find_best_heal_target, find_heal_priority};
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::brute::components::Brute;
use crate::game::units::commander::components::Commander;
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, Health, MindControlled, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team,
};
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::healer::components::Healer;
use crate::game::units::healer::constants::HEAL_RANGE;
use crate::game::units::infantry::components::DefendersActivated;

/// Updates healer targeting — seeks hurt same-team allies, or falls back to following army.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_healer_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut healers: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Healer>, Without<Corpse>, Without<MindControlled>),
    >,
    potential_targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &Health,
            Option<&Commander>,
            Option<&Brute>,
            Option<&EliteSpeedBonus>,
            Option<&Dispeller>,
            Option<&Healer>,
        ),
        (Without<Corpse>, Without<BanishedModifier>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    // Snapshot ally data for heal targeting
    let ally_snapshot: Vec<(Entity, Vec3, Team, f32, f32, u32)> = potential_targets
        .iter()
        .map(
            |(entity, transform, team, health, commander, brute, elite, dispeller, healer)| {
                let priority = find_heal_priority(
                    commander.is_some(),
                    brute.is_some(),
                    elite.is_some(),
                    dispeller.is_some(),
                    healer.is_some(),
                );
                (
                    entity,
                    transform.translation,
                    *team,
                    health.current,
                    health.max,
                    priority,
                )
            },
        )
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut healers {
        // Skip inactive defender healers
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 1: Find hurt allies to heal / move toward
        let best_hurt_ally =
            find_best_heal_target(&ally_snapshot, entity, transform.translation, *team);

        if let Some((_, ally_pos, _)) = best_hurt_ally {
            let diff = ally_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= HEAL_RANGE {
                // In heal range — stop moving
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward hurt ally
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }

            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Priority 2: Fall back to following army toward nearest enemy
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
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

        if let Some(&(_, target_pos, _)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            // Stay at heal range from enemies
            if distance <= HEAL_RANGE {
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }

        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
}

/// Healer movement system using shared weighted movement.
#[allow(clippy::type_complexity)]
pub fn healer_movement(
    time: Res<Time>,
    mut healer_units: Query<
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
                &Team,
                Has<StagingAttacker>,
                Has<WaveGroup>,
            ),
        ),
        With<Healer>,
    >,
) {
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
        (
            sleeping,
            sleepwalking,
            banished,
            polymorphed,
            sickened,
            frozen,
            stunned,
            petrified,
            team,
            has_staging,
            has_wave_group,
        ),
    ) in &mut healer_units
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

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
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

        // Stop completely when in optimal position (not in melee, not on hazard)
        // Skip for staging units — they need to keep following the flow field
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        if !is_staging && in_melee.is_none() && flow_field_velocity.terrain_cost <= 1.0 {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}
