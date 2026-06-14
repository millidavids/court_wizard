use bevy::prelude::*;

use super::super::components::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker};
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, MovementSpeed, PolymorphedModifier, RootedModifier,
    RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier,
    TargetingVelocity, Team,
};
use crate::game::units::wizard::components::Wizard;

/// Updates infantry targeting velocity toward nearest enemy.
///
/// Infantry always move directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
/// Defender infantry are gated by the DefendersActivated resource.
#[allow(clippy::type_complexity)]
pub fn update_infantry_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut infantry: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut crate::game::units::components::TargetingVelocity,
            Option<&crate::game::units::components::RetaliationTarget>,
        ),
        (
            With<Infantry>,
            Without<crate::game::units::components::Corpse>,
            Without<crate::game::units::components::MindControlled>,
        ),
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

    // Update each infantry's targeting velocity
    for (entity, transform, team, mut targeting_velocity, retaliation) in &mut infantry {
        // Skip inactive defender infantry (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
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

/// Infantry-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn infantry_movement(
    time: Res<Time>,
    mut infantry_units: Query<
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
        With<Infantry>,
    >,
) {
    // Process each infantry unit
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
        (has_sleep, has_sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut infantry_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            has_sleep,
            has_sleepwalking,
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
    }
}
