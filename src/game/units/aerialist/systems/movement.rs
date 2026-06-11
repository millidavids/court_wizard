use bevy::prelude::*;
use rand::Rng;

use super::super::components::Aerialist;
use super::super::constants::{AERIALIST_FLY_HEIGHT, AERIALIST_MIN_SPEED, AERIALIST_TURN_RATE};
use crate::game::components::Velocity;
use crate::game::pathfinding::{FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, HasteModifier, MovementSpeed,
    PolymorphedModifier, RootedModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team,
};

/// Aerialist movement: momentum-based flying with wide sweeping arcs.
///
/// Unlike ground units, aerialists never stop moving. They maintain a minimum
/// speed and gradually steer toward their desired direction, creating long
/// swooping flyover paths. They attack mid-flight.
#[allow(clippy::type_complexity)]
pub(crate) fn aerialist_movement(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut aerialist_units: Query<
        (
            &mut Velocity,
            &MovementSpeed,
            &TargetingVelocity,
            &FlowFieldVelocity,
            (
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&SlowMovementModifier>,
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
        With<Aerialist>,
    >,
) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }

    for (
        mut velocity,
        movement_speed,
        targeting_velocity,
        flow_field_velocity,
        (rooted, haste_modifier, slow_modifier),
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
    ) in &mut aerialist_units
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
            velocity.x = angle.cos() * AERIALIST_MIN_SPEED;
            velocity.z = angle.sin() * AERIALIST_MIN_SPEED;
            continue;
        }

        // Calculate desired direction from flow field + targeting blend
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        let desired_dir = if is_staging {
            // While staging, follow the flow field directly
            flow_field_velocity.velocity
        } else {
            // Blend flow field and targeting based on distance to target
            let dist = targeting_velocity.distance_to_target;
            let flow_weight = if dist > 500.0 {
                0.7
            } else if dist > 200.0 {
                0.5
            } else {
                0.2
            };
            let target_weight = 1.0 - flow_weight;
            flow_field_velocity.velocity * flow_weight + targeting_velocity.velocity * target_weight
        };

        // Calculate target speed with modifiers
        let mut speed = movement_speed.0 * crate::game::constants::GLOBAL_SPEED_MULTIPLIER;
        if let Some(slow) = slow_modifier {
            speed *= slow.modifier;
        }
        if let Some(haste) = haste_modifier {
            speed *= 1.0 + haste.modifier;
        }
        speed = speed.max(AERIALIST_MIN_SPEED);

        // Current velocity direction
        let current_vel = Vec3::new(velocity.x, 0.0, velocity.z);
        let current_speed = current_vel.length();

        // If we have no velocity yet, pick a direction
        if current_speed < 1.0 {
            let dir = if desired_dir.length_squared() > 0.001 {
                desired_dir.normalize()
            } else {
                // Random initial direction
                let angle = game_rng.0.random::<f32>() * std::f32::consts::TAU;
                Vec3::new(angle.cos(), 0.0, angle.sin())
            };
            velocity.x = dir.x * speed;
            velocity.z = dir.z * speed;
            continue;
        }

        let current_dir = current_vel / current_speed;

        // Gradually rotate current direction toward desired direction
        let target_dir = if desired_dir.length_squared() > 0.001 {
            desired_dir.normalize()
        } else {
            current_dir
        };

        // Calculate angle between current and desired direction
        let dot = current_dir.dot(target_dir).clamp(-1.0, 1.0);
        let angle_between = dot.acos();

        // Limit turn rate per frame
        let max_turn = AERIALIST_TURN_RATE * delta;
        let new_dir = if angle_between <= max_turn || angle_between < 0.001 {
            target_dir
        } else {
            // Slerp-like interpolation on XZ plane
            let cross_y = current_dir.x * target_dir.z - current_dir.z * target_dir.x;
            let turn_sign = if cross_y >= 0.0 { 1.0 } else { -1.0 };
            let cos_turn = max_turn.cos();
            let sin_turn = max_turn.sin() * turn_sign;
            Vec3::new(
                current_dir.x * cos_turn - current_dir.z * sin_turn,
                0.0,
                current_dir.x * sin_turn + current_dir.z * cos_turn,
            )
            .normalize_or_zero()
        };

        // Smoothly adjust speed toward target speed
        let new_speed = current_speed + (speed - current_speed) * (3.0 * delta).min(1.0);
        let final_speed = new_speed.max(AERIALIST_MIN_SPEED);

        velocity.x = new_dir.x * final_speed;
        velocity.z = new_dir.z * final_speed;
    }
}

/// Clamps aerialist Y position to fly height after all movement is applied.
pub(crate) fn clamp_aerialist_height(
    mut aerialists: Query<&mut Transform, (With<Aerialist>, Without<Corpse>)>,
) {
    for mut transform in &mut aerialists {
        transform.translation.y = AERIALIST_FLY_HEIGHT;
    }
}
