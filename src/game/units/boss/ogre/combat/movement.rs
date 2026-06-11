use crate::game::pathfinding::FlowFieldVelocity;
use bevy::prelude::*;

use super::super::components::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, InMelee, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity,
};

/// Ogre movement system using weighted velocities.
/// Feeds enrage speed bonus through the haste parameter.
#[allow(clippy::type_complexity)]
pub fn ogre_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &OgreEnrageState,
            &OgreChargeState,
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
        With<Boss>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        enrage_state,
        charge_state,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut bosses
    {
        // Freeze normal movement during charge phases; also zero acceleration
        // so external forces (e.g. black hole gravity) don't drift the ogre off course
        if charge_state.is_movement_locked() {
            velocity.x = 0.0;
            velocity.z = 0.0;
            acceleration.x = 0.0;
            acceleration.z = 0.0;
            continue;
        }

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

        // Combine haste modifier with enrage speed bonus
        let combined_haste =
            Some(haste_modifier.map(|m| m.modifier).unwrap_or(0.0) + enrage_state.speed_bonus);

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
            combined_haste,
            elite_speed.map(|e| e.0),
        );
    }
}
