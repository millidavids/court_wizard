use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::{EyeTransferTimer, HagAssets, HagDeathTracker};
use super::animation::eye_pulsing_animation;
use super::spawn::spawn_eye_visual;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, Health, Invulnerable};

/// Ticks the eye transfer timer and launches eyes in flight to new hag holders.
/// Invulnerability is removed immediately when the eye leaves the source hag.
#[allow(clippy::too_many_arguments)]
pub fn tick_eye_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut timer: ResMut<EyeTransferTimer>,
    mut hags: Query<
        (Entity, &Transform, &mut HagEyeState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    eye_visuals: Query<(Entity, &ChildOf, &EyeVisual)>,
    hag_assets: Res<HagAssets>,
    death_tracker: Res<HagDeathTracker>,
) {
    // Don't tick the timer until at least one hag has finished staging —
    // otherwise eyes would shuffle around before the fight even starts.
    if hags.is_empty() {
        return;
    }

    timer.time_remaining -= time.delta_secs();
    if timer.time_remaining > 0.0 {
        return;
    }

    // Reset timer
    timer.time_remaining = EYE_TRANSFER_BASE_INTERVAL
        + game_rng
            .0
            .random_range(-EYE_TRANSFER_VARIANCE..EYE_TRANSFER_VARIANCE);

    let living_hags: Vec<Entity> = hags.iter().map(|(e, _, _)| e).collect();
    if living_hags.len() < 2 {
        return;
    }

    // Determine how many eyes are still in play based on permanent deaths
    let has_invuln_eye = death_tracker.permanent_deaths < 1;
    let has_ability_eye = death_tracker.permanent_deaths < 2;

    // Find current holders
    let mut current_invuln_holder: Option<Entity> = None;
    let mut current_ability_holder: Option<Entity> = None;
    for (entity, _, eye_state) in &hags {
        if eye_state.has_invulnerability_eye {
            current_invuln_holder = Some(entity);
        }
        if eye_state.has_ability_eye {
            current_ability_holder = Some(entity);
        }
    }

    // Pick new holders (must be different from current holder)
    let new_invuln_holder = if has_invuln_eye {
        let candidates: Vec<Entity> = living_hags
            .iter()
            .copied()
            .filter(|e| Some(*e) != current_invuln_holder)
            .collect();
        if candidates.is_empty() {
            current_invuln_holder // Only one hag alive, keep it
        } else {
            Some(candidates[game_rng.0.random_range(0..candidates.len())])
        }
    } else {
        None
    };

    let new_ability_holder = if has_ability_eye {
        let candidates: Vec<Entity> = living_hags
            .iter()
            .copied()
            .filter(|e| Some(*e) != current_ability_holder && Some(*e) != new_invuln_holder)
            .collect();
        if candidates.is_empty() {
            current_ability_holder // No valid candidate, keep it
        } else {
            Some(candidates[game_rng.0.random_range(0..candidates.len())])
        }
    } else {
        None
    };

    // Process invulnerability eye
    if let Some(new_holder) = new_invuln_holder {
        let needs_flight = current_invuln_holder.is_some_and(|cur| cur != new_holder);

        if needs_flight {
            let source = current_invuln_holder.expect("checked above");
            // Clear source hag's eye state and invulnerability immediately
            if let Ok((_, _, mut eye_state)) = hags.get_mut(source) {
                eye_state.has_invulnerability_eye = false;
            }
            commands.entity(source).remove::<Invulnerable>();
            // Despawn eye visual from source
            for (eye_entity, child_of, eye_visual) in &eye_visuals {
                if child_of.parent() == source && eye_visual.eye_type == EyeType::Invulnerability {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Get source position and spawn flying eye
            if let Ok((_, source_transform, _)) = hags.get(source) {
                let start_pos =
                    source_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_sprite_mesh.clone()),
                    MeshMaterial3d(hag_assets.invulnerability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
                    eye_pulsing_animation(),
                    EyeInFlight {
                        eye_type: EyeType::Invulnerability,
                        target: new_holder,
                        start_pos,
                        progress: 0.0,
                    },
                ));
            }
        } else if current_invuln_holder.is_none() {
            // Eye needs to appear fresh (shouldn't happen after initialize_eyes, but handle it)
            if let Ok((_, _, mut eye_state)) = hags.get_mut(new_holder) {
                eye_state.has_invulnerability_eye = true;
                let both = new_ability_holder == Some(new_holder);
                spawn_eye_visual(
                    &mut commands,
                    new_holder,
                    EyeType::Invulnerability,
                    &hag_assets,
                    both,
                );
            }
        }
        // If staying on same hag, do nothing
    }

    // Process ability eye
    if let Some(new_holder) = new_ability_holder {
        let needs_flight = current_ability_holder.is_some_and(|cur| cur != new_holder);

        if needs_flight {
            let source = current_ability_holder.expect("checked above");
            // Clear source hag's eye state immediately
            if let Ok((_, _, mut eye_state)) = hags.get_mut(source) {
                eye_state.has_ability_eye = false;
            }
            // Despawn eye visual from source
            for (eye_entity, child_of, eye_visual) in &eye_visuals {
                if child_of.parent() == source && eye_visual.eye_type == EyeType::Ability {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Get source position and spawn flying eye
            if let Ok((_, source_transform, _)) = hags.get(source) {
                let start_pos =
                    source_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_sprite_mesh.clone()),
                    MeshMaterial3d(hag_assets.ability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
                    eye_pulsing_animation(),
                    EyeInFlight {
                        eye_type: EyeType::Ability,
                        target: new_holder,
                        start_pos,
                        progress: 0.0,
                    },
                ));
            }
        } else if current_ability_holder.is_none()
            && let Ok((_, _, mut eye_state)) = hags.get_mut(new_holder)
        {
            eye_state.has_ability_eye = true;
            let both = new_invuln_holder == Some(new_holder);
            spawn_eye_visual(
                &mut commands,
                new_holder,
                EyeType::Ability,
                &hag_assets,
                both,
            );
        }
        // If staying on same hag, do nothing
    }

    // Fix X offsets for eyes that stayed on a hag that now lost or gained a companion eye
    // Despawn and re-spawn visuals for hags whose eye count changed but eyes didn't move
    for (entity, _, eye_state) in &hags {
        let had_invuln = current_invuln_holder == Some(entity);
        let had_ability = current_ability_holder == Some(entity);
        let has_invuln = eye_state.has_invulnerability_eye;
        let has_ability = eye_state.has_ability_eye;

        // If this hag still has an eye but the other eye left/arrived, re-offset
        let old_both = had_invuln && had_ability;
        let new_both = has_invuln && has_ability;
        if old_both != new_both && (has_invuln || has_ability) {
            // Despawn existing eye visuals for this hag
            for (eye_entity, child_of, _) in &eye_visuals {
                if child_of.parent() == entity {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Re-spawn with correct offset
            if has_invuln {
                spawn_eye_visual(
                    &mut commands,
                    entity,
                    EyeType::Invulnerability,
                    &hag_assets,
                    new_both,
                );
            }
            if has_ability {
                spawn_eye_visual(
                    &mut commands,
                    entity,
                    EyeType::Ability,
                    &hag_assets,
                    new_both,
                );
            }
        }
    }
}

/// Updates eyes in flight — arcs them toward their target hag and delivers on arrival.
pub fn update_eye_flight(
    time: Res<Time>,
    mut commands: Commands,
    mut eyes: Query<(Entity, &mut EyeInFlight, &mut Transform), Without<Hag>>,
    mut hags: Query<
        (Entity, &Transform, &mut HagEyeState, &Health),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    hag_assets: Res<HagAssets>,
    existing_eye_visuals: Query<(Entity, &ChildOf, &EyeVisual), Without<EyeInFlight>>,
) {
    let delta = time.delta_secs();

    for (eye_entity, mut flight, mut eye_transform) in &mut eyes {
        flight.progress += delta / EYE_TOSS_FLIGHT_DURATION;

        // Get target hag's current position (homing)
        let target_pos = if let Ok((_, target_transform, _, _)) = hags.get(flight.target) {
            target_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0)
        } else {
            // Target died or despawned — just despawn the eye
            commands.entity(eye_entity).try_despawn();
            continue;
        };

        if flight.progress >= 1.0 {
            // Eye arrived — deliver to target hag
            commands.entity(eye_entity).try_despawn();

            if let Ok((_, _, mut eye_state, health)) = hags.get_mut(flight.target) {
                match flight.eye_type {
                    EyeType::Invulnerability => {
                        eye_state.has_invulnerability_eye = true;
                        commands.entity(flight.target).insert(Invulnerable {
                            health_snapshot: health.current,
                        });
                    }
                    EyeType::Ability => eye_state.has_ability_eye = true,
                }

                let has_both = eye_state.has_invulnerability_eye && eye_state.has_ability_eye;

                // If gaining a second eye, re-spawn existing eye with correct offset
                if has_both {
                    for (vis_entity, child_of, _) in &existing_eye_visuals {
                        if child_of.parent() == flight.target {
                            commands.entity(vis_entity).try_despawn();
                        }
                    }
                    // Re-spawn the other eye type with both=true offset
                    let other_type = match flight.eye_type {
                        EyeType::Invulnerability => EyeType::Ability,
                        EyeType::Ability => EyeType::Invulnerability,
                    };
                    spawn_eye_visual(&mut commands, flight.target, other_type, &hag_assets, true);
                }

                spawn_eye_visual(
                    &mut commands,
                    flight.target,
                    flight.eye_type,
                    &hag_assets,
                    has_both,
                );
            }
        } else {
            // Interpolate position with parabolic arc
            let t = flight.progress;
            let x = flight.start_pos.x + (target_pos.x - flight.start_pos.x) * t;
            let z = flight.start_pos.z + (target_pos.z - flight.start_pos.z) * t;
            let base_y = flight.start_pos.y + (target_pos.y - flight.start_pos.y) * t;
            let arc_offset = EYE_TOSS_ARC_HEIGHT * 4.0 * t * (1.0 - t);
            eye_transform.translation = Vec3::new(x, base_y + arc_offset, z);
        }
    }
}
