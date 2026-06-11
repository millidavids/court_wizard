use bevy::prelude::*;

use super::super::components::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, EliteSpeedBonus, FrozenSolidModifier,
    HasteModifier, MovementSpeed, PolymorphedModifier, RootedModifier, RoughTerrainModifier,
    SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier, TargetingVelocity, Team,
};

/// Archer-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn archer_movement(
    time: Res<Time>,
    mut archer_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &crate::game::units::components::FlockingVelocity,
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
        With<Archer>,
    >,
) {
    // Process each archer unit
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
    ) in &mut archer_units
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

        // Archer-specific: Stop completely when in optimal shooting range (not in melee).
        // But keep moving if:
        //  - staging (needs to follow flow field to staging point)
        //  - standing on hazardous terrain (fire, spikes)
        //  - no target in range (needs to follow flow field back to spawn)
        //  - path is fully blocked (wall-attack system needs velocity)
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        if !is_staging
            && in_melee.is_none()
            && flow_field_velocity.terrain_cost <= 1.0
            && !flow_field_velocity.pathfinding_distance.is_infinite()
            && targeting_velocity.distance_to_target < f32::MAX
        {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                // Override velocity and acceleration to completely stop archer when in shooting stance
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}
