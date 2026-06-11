use bevy::prelude::*;

use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, MovementSpeed, PolymorphedModifier, RootedModifier,
    RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier,
    Stunned, TargetingVelocity,
};
use crate::game::units::teleporter::components::{Teleporter, TeleporterState};

/// Movement: reuses shared weighted movement. Stops when channeling.
#[allow(clippy::type_complexity)]
pub(crate) fn teleporter_movement(
    time: Res<Time>,
    mut teleporters: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &TeleporterState,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<Teleporter>, Without<Corpse>),
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting,
        flocking,
        flow_field,
        state,
        aura_mod,
        terrain_mod,
        slow_mod,
        (
            rooted,
            haste,
            elite_speed,
            sleeping,
            sleepwalking,
            banished,
            polymorphed,
            sickened,
            frozen,
            stunned,
            petrified,
        ),
    ) in &mut teleporters
    {
        if matches!(state, TeleporterState::Channeling { .. }) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

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

        if polymorphed.is_some() {
            continue;
        }

        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting,
            flocking,
            flow_field,
            false,
            aura_mod.map(|m| m.0),
            terrain_mod.map(|m| m.0),
            slow_mod.map(|m| m.modifier),
            None,
            haste.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}
