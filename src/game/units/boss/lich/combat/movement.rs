use bevy::prelude::*;

use super::super::components::*;
use crate::game::components::{Acceleration, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{Corpse, MovementSpeed, TargetingVelocity};

/// Lich movement for all phases.
/// - Approaching: follows flow field toward staging point
/// - Summoning: stationary
/// - Combat: targeting + flow field toward defenders
pub(crate) fn lich_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &TargetingVelocity,
            &FlowFieldVelocity,
            &MovementSpeed,
            &LichPhase,
        ),
        (With<Lich>, Without<Corpse>),
    >,
) {
    for (transform, mut velocity, mut acceleration, targeting, flow_field, speed, phase) in
        &mut query
    {
        match phase {
            LichPhase::Summoning => {
                // Stationary — zero everything
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.reset();
            }
            LichPhase::Approaching => {
                // Steer directly toward the staging point (can't use the staging
                // flow field because is_staging_attacker is Team::Attackers only).
                let max_speed = speed.0 * GLOBAL_SPEED_MULTIPLIER;
                let pos = transform.translation;
                let staging = STAGING_POINTS[CENTER_STAGING_INDEX];
                let to_staging = Vec3::new(staging.0 - pos.x, 0.0, staging.1 - pos.z);

                if to_staging.length_squared() > 1.0 {
                    let target_vel = to_staging.normalize() * max_speed;
                    let steer = STEERING_FORCE * time.delta_secs();
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                }

                velocity.max_speed = max_speed;
                let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
                velocity.x *= damping;
                velocity.z *= damping;
            }
            LichPhase::Combat => {
                // Combine targeting and flow field
                let max_speed = speed.0 * GLOBAL_SPEED_MULTIPLIER;
                let combined = Vec3::new(
                    targeting.velocity.x * 0.7 + flow_field.velocity.x * 0.3,
                    0.0,
                    targeting.velocity.z * 0.7 + flow_field.velocity.z * 0.3,
                );

                if combined.length_squared() > 0.001 {
                    let target_vel = combined.normalize() * max_speed;
                    let steer = STEERING_FORCE * time.delta_secs();
                    acceleration.x = (target_vel.x - velocity.x).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                    acceleration.z = (target_vel.z - velocity.z).clamp(-steer, steer)
                        / time.delta_secs().max(0.001);
                }

                velocity.max_speed = max_speed;
                let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
                velocity.x *= damping;
                velocity.z *= damping;
            }
        }
    }
}
