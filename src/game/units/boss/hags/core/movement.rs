use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker};
use crate::game::units::components::{
    BanishedModifier, CommanderAuraSpeedModifier, Corpse, EliteSpeedBonus, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, InMelee, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity,
};
use crate::game::units::king::components::King;

/// Hag movement system using weighted velocities.
#[allow(clippy::type_complexity)]
pub fn hag_movement(
    time: Res<Time>,
    mut hags: Query<
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
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
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
    ) in &mut hags
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

        // All hags use weighted movement (flow field + targeting)
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

/// Applies a gentle separation force between hags to keep them spread apart.
/// Skipped during staging — all three hags share the center staging point and
/// shouldn't be shoved apart on the way there.
pub fn hag_separation(
    time: Res<Time>,
    mut hags: Query<
        (Entity, &Transform, &mut Velocity),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
) {
    let delta = time.delta_secs();
    let positions: Vec<(Entity, Vec3)> = hags.iter().map(|(e, t, _)| (e, t.translation)).collect();

    for (entity, _transform, mut velocity) in &mut hags {
        let my_pos = positions
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, p)| *p);
        let Some(my_pos) = my_pos else { continue };

        for (other_entity, other_pos) in &positions {
            if *other_entity == entity {
                continue;
            }
            let dx = my_pos.x - other_pos.x;
            let dz = my_pos.z - other_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < HAG_SEPARATION_DISTANCE && dist > 0.1 {
                // Quadratic falloff — soft at the outer edge so flow-field
                // guidance dominates, but ramps up sharply when the sprites
                // are about to overlap. Per-second so it's frame-rate independent.
                let linear = 1.0 - (dist / HAG_SEPARATION_DISTANCE);
                let factor = linear * linear;
                let push_x = (dx / dist) * HAG_SEPARATION_STRENGTH * factor * delta;
                let push_z = (dz / dist) * HAG_SEPARATION_STRENGTH * factor * delta;
                velocity.x += push_x;
                velocity.z += push_z;
            }
        }
    }
}

/// Stops Justina advancing once she's within `JUSTINA_KITE_DISTANCE` of the king,
/// so she kites with her ranged abilities instead of charging into melee.
pub fn justina_kite_distance(
    mut justina_query: Query<
        (&Transform, &HagIdentity, &mut Velocity),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    king_query: Query<&Transform, (With<King>, Without<Hag>, Without<Corpse>)>,
) {
    let Ok(king_transform) = king_query.single() else {
        return;
    };
    let king_pos = king_transform.translation;
    let kite_dist_sq = JUSTINA_KITE_DISTANCE * JUSTINA_KITE_DISTANCE;

    for (transform, identity, mut velocity) in &mut justina_query {
        if *identity != HagIdentity::Justina {
            continue;
        }
        let dx = transform.translation.x - king_pos.x;
        let dz = transform.translation.z - king_pos.z;
        if dx * dx + dz * dz <= kite_dist_sq {
            velocity.x = 0.0;
            velocity.z = 0.0;
        }
    }
}
