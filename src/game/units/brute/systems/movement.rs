use bevy::prelude::*;

use super::super::components::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker};
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, InMelee, MindControlled, MovementSpeed,
    PolymorphedModifier, RetaliationTarget, RootedModifier, RoughTerrainModifier, SickenedModifier,
    SleepModifier, Sleepwalking, SlowMovementModifier, TargetingVelocity, Team,
};

/// Updates brute targeting velocity toward nearest enemy.
pub(in crate::game) fn update_brute_targeting(
    mut commands: Commands,
    mut brutes: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&RetaliationTarget>,
        ),
        (With<Brute>, Without<MindControlled>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Brute>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, brute_transform, brute_team, mut targeting, retaliation) in &mut brutes {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            brute_transform,
            *brute_team,
            &mut targeting,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// Brute movement system using weighted velocities.
#[allow(clippy::type_complexity)]
pub(in crate::game) fn brute_movement(
    time: Res<Time>,
    mut brutes: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&InMelee>,
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
        With<Brute>,
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
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut brutes
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
