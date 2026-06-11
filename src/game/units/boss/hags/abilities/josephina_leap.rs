use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::core::{restore_hag_walking_pose, set_hag_attack_pose_frame};
use super::super::resources::{HagAssets, HagDeathTracker};
use crate::game::components::Velocity;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    AnimationOverride, BanishedModifier, Corpse, FacingDirection, Knockback, MindControlled, Team,
};

/// Josephina's leap — parabolic arc jump to a random defender, knockback on landing.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn josephina_leap(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    death_tracker: Res<HagDeathTracker>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut josephina_query: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut LeapState,
            &mut Velocity,
            &FacingDirection,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    defenders: Query<
        (&Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        hag_entity,
        mut transform,
        team,
        identity,
        eye_state,
        mut leap,
        mut velocity,
        facing,
        material_handle,
    ) in &mut josephina_query
    {
        let enraged = death_tracker.permanent_deaths >= 2;
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // If eye transferred away mid-leap, cancel and land — also reset Y
            // back to the pre-leap height so successive leaps don't stack
            // elevation each time the eye is yanked mid-air.
            if let LeapState::InAir {
                target, start_pos, ..
            } = &*leap
            {
                transform.translation.x = target.x;
                transform.translation.y = start_pos.y;
                transform.translation.z = target.z;
                *leap = LeapState::Idle {
                    cooldown: LEAP_COOLDOWN,
                };
                restore_hag_walking_pose(&mut materials, material_handle, &hag_assets, *facing);
                commands.entity(hag_entity).remove::<AnimationOverride>();
            }
            continue;
        }

        match &mut *leap {
            LeapState::Idle { cooldown } => {
                *cooldown -= delta;
                if *cooldown > 0.0 {
                    continue;
                }

                // Pick a random defender target within leap range
                let josephina_pos = transform.translation;
                let defender_positions: Vec<Vec3> = defenders
                    .iter()
                    .filter(|(_, t)| team.is_enemy(t))
                    .map(|(t, _)| t.translation)
                    .filter(|pos| {
                        let dx = pos.x - josephina_pos.x;
                        let dz = pos.z - josephina_pos.z;
                        (dx * dx + dz * dz).sqrt() <= LEAP_MAX_RANGE
                    })
                    .collect();

                if defender_positions.is_empty() {
                    continue;
                }

                let idx = game_rng.0.random_range(0..defender_positions.len());
                let target = defender_positions[idx];

                *leap = LeapState::InAir {
                    target,
                    start_pos: transform.translation,
                    progress: 0.0,
                };
                // Pin to the second frame (index 1) of the attack sheet for the
                // duration of the leap. AnimationOverride keeps update_walking_animation
                // from clobbering the pose.
                commands.entity(hag_entity).insert(AnimationOverride);
                set_hag_attack_pose_frame(&mut materials, material_handle, &hag_assets, *facing, 1);

                // Zero velocity during leap
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            LeapState::InAir {
                target,
                start_pos,
                progress,
            } => {
                *progress += delta / LEAP_FLIGHT_DURATION;

                if *progress >= 1.0 {
                    // Land at target, restore Y to pre-leap height
                    transform.translation.x = target.x;
                    transform.translation.y = start_pos.y;
                    transform.translation.z = target.z;
                    // Knockback is applied by josephina_leap_knockback system
                    *leap = LeapState::Landing {
                        timer: 0.3,
                        knockback_applied: false,
                    };
                    // Hold the third frame (index 2) of the attack sheet on landing.
                    set_hag_attack_pose_frame(
                        &mut materials,
                        material_handle,
                        &hag_assets,
                        *facing,
                        2,
                    );
                } else {
                    // Parabolic arc interpolation
                    let t = *progress;
                    let x = start_pos.x + (target.x - start_pos.x) * t;
                    let z = start_pos.z + (target.z - start_pos.z) * t;
                    // Parabolic height: peaks at t=0.5
                    let height_offset = LEAP_MAX_HEIGHT * 4.0 * t * (1.0 - t);
                    transform.translation.x = x;
                    transform.translation.y = start_pos.y + height_offset;
                    transform.translation.z = z;
                }
            }
            LeapState::Landing { timer, .. } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    // After landing, transition to mauling (handled by mauling system)
                    *leap = LeapState::Idle {
                        cooldown: LEAP_COOLDOWN,
                    };
                    restore_hag_walking_pose(&mut materials, material_handle, &hag_assets, *facing);
                    commands.entity(hag_entity).remove::<AnimationOverride>();
                }
            }
        }
    }
}

/// Apply knockback on Josephina's leap landing.
pub fn josephina_leap_knockback(
    mut commands: Commands,
    mut josephina_query: Query<
        (&Transform, &Team, &HagIdentity, &mut LeapState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    for (transform, team, identity, mut leap) in &mut josephina_query {
        if *identity != HagIdentity::Josephina {
            continue;
        }

        // Only apply knockback once at landing moment
        if let LeapState::Landing {
            knockback_applied, ..
        } = leap.as_mut()
        {
            if *knockback_applied {
                continue;
            }
            *knockback_applied = true;

            let land_pos = transform.translation;
            for (entity, target_transform, target_team) in &targets {
                if !team.is_enemy(target_team) {
                    continue;
                }

                let dx = target_transform.translation.x - land_pos.x;
                let dz = target_transform.translation.z - land_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= LEAP_KNOCKBACK_RADIUS {
                    let direction = if dist > 0.1 {
                        Vec3::new(dx, 0.0, dz)
                    } else {
                        Vec3::X
                    };
                    commands.entity(entity).insert(Knockback::new(
                        direction,
                        LEAP_KNOCKBACK_SPEED,
                        LEAP_KNOCKBACK_DURATION,
                    ));
                }
            }
        }
    }
}
