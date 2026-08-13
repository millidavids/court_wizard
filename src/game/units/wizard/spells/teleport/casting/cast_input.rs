//! Teleport input handling and indicator circle management.

use super::super::arrival::apply_post_teleport_effects;
use bevy::prelude::*;

use super::super::components::{
    LingeringGateMarker, TeleportCaster, TeleportDestinationCircle, TeleportSourceCircle,
    TeleportTalentParams,
};
use super::super::constants::*;
use super::finalize::teleport_casting_logic;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::{MouseLeftReleased, MouseRightPressed};
use crate::game::units::components::{Corpse, Team, Teleportable};
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::messages::announce_area_cast;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, clamp_to_spell_range,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::resources::NetworkConnection;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> TeleportTalentParams {
    let mut params = TeleportTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Teleport, 0);
    let t2 = talents.get_selection(Spell::Teleport, 1);
    let t3 = talents.get_selection(Spell::Teleport, 2);

    match t1 {
        Some(0) => params.radius_mult = WIDE_APERTURE_RADIUS_MULT,
        Some(1) => params.cast_time_mult = HASTY_TRANSLOCATION_CAST_TIME_MULT,
        Some(2) => params.lingering_gate = true,
        _ => {}
    }

    match t2 {
        Some(0) => params.disorienting_arrival = true,
        Some(1) => params.swap_mode = true,
        Some(2) => params.emergency_recall = true,
        _ => {}
    }

    match t3 {
        Some(0) => params.dimensional_rift = true,
        Some(1) => params.teleport_up = true,
        Some(2) => params.scatterport = true,
        _ => {}
    }

    params
}

/// Handles right-click to cancel/reset the teleport spell.
///
/// This system runs independently of the main casting system to ensure
/// right-click always cancels, even when other conditions would block casting.
pub fn handle_teleport_cancel(
    mut mouse_right_pressed: MessageReader<MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, Entity), With<LocalWizard>>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    // Only process if right-click occurred
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    // Get wizard and caster
    let Ok((mut casting_state, wizard_entity)) = wizard_query.single_mut() else {
        return;
    };

    let mut caster = if let Ok(c) = caster_query.single_mut() {
        c
    } else {
        commands.entity(wizard_entity).insert(TeleportCaster::new());
        return;
    };

    // Despawn any active circles
    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).try_despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).try_despawn();
    }

    // Reset all state
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    caster.lingering_gate_active = false;
    casting_state.cancel();
    mouse_state.left_consumed = true; // Prevent immediate restart if left button still held
}

/// Local wizard Teleport casting — reads mouse input, manages indicator circles.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn handle_teleport_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        (
            With<LocalWizard>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut caster_query: Query<&mut TeleportCaster>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        (
            With<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    mut source_query: Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
    units_query: Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    target_assist: Res<TargetAssistWorldPos>,
    (
        mut connection,
        sfx,
        game_config,
        active_talents,
        mut talent_progress,
        mut pending_cast_events,
    ): (
        Option<ResMut<NetworkConnection>>,
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
    local_origin: Res<LocalSpellOrigin>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Teleport {
        return;
    }

    // Safety check
    if mouse_state.left_consumed {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands.entity(wizard_entity).insert(TeleportCaster::new());
        return;
    };

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    let cast_result = teleport_casting_logic(
        &mut game_rng.0,
        &input,
        &time,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        &mut caster,
        &mut commands,
        &visual_assets,
        &units_query,
        &source_query,
        &talent_params,
        &mut pending_cast_events,
    );

    // === Local-only: manage indicator circles ===

    let effective_circle_radius = CIRCLE_RADIUS * talent_params.radius_mult;

    // Phase 1: Spawn/update destination crosshair
    if !caster.has_destination() && !talent_params.emergency_recall && !talent_params.teleport_up {
        match *casting_state {
            CastingState::Resting => {
                // Waiting for user to click
            }
            CastingState::Casting { .. } => {
                // Destination crosshair — spawn if needed, update position
                // Swap talent: show full-size circle so the player can see the capture area
                if caster.destination_circle.is_none() {
                    if let Some(pos) = clamped_pos {
                        let radius = if talent_params.swap_mode {
                            effective_circle_radius * primed_spell.empowerment
                        } else {
                            primed_spell.scale(CROSSHAIR_RADIUS)
                        };

                        let crosshair_entity = commands
                            .spawn((
                                Mesh3d(visual_assets.unit_circle.clone()),
                                MeshMaterial3d(visual_assets.teleport_destination.clone()),
                                Transform::from_xyz(pos.x, 1.0, pos.z)
                                    .with_rotation(Quat::from_rotation_x(
                                        -std::f32::consts::FRAC_PI_2,
                                    ))
                                    .with_scale(Vec3::splat(radius)),
                                TeleportDestinationCircle::new(radius),
                                OnGameplayScreen,
                            ))
                            .id();

                        caster.destination_circle = Some(crosshair_entity);
                    }
                } else if let Some(circle_entity) = caster.destination_circle
                    && let Ok((mut transform, _)) = destination_query.get_mut(circle_entity)
                    && let Some(pos) = clamped_pos
                {
                    transform.translation.x = pos.x;
                    transform.translation.z = pos.z;
                }
            }
            _ => {}
        }
    } else {
        // Phase 2: Spawn/update source circle (also used for Emergency Recall)
        if let CastingState::Casting { elapsed } = *casting_state {
            let cast_time = SECOND_CAST_TIME * talent_params.cast_time_mult;
            if caster.source_circle.is_none() {
                if let Some(pos) = clamped_pos {
                    let circle_entity = commands
                        .spawn((
                            Mesh3d(visual_assets.unit_circle.clone()),
                            MeshMaterial3d(visual_assets.teleport_source.clone()),
                            Transform::from_xyz(pos.x, 1.0, pos.z)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::ZERO),
                            TeleportSourceCircle::new(pos, primed_spell.empowerment),
                            OnGameplayScreen,
                        ))
                        .id();

                    caster.source_circle = Some(circle_entity);
                }
            } else if let Some(circle_entity) = caster.source_circle
                && let Ok((mut transform, mut indicator)) = source_query.get_mut(circle_entity)
                && let Some(pos) = clamped_pos
            {
                transform.translation.x = pos.x;
                transform.translation.z = pos.z;

                let growth = (elapsed / cast_time).min(1.0);
                let radius = effective_circle_radius * indicator.empowerment;
                transform.scale = Vec3::splat(radius * growth);

                indicator.position = pos;
                indicator.time_alive += time.delta_secs();
            }
        }
    }

    // Cleanup circles on completion or first-phase release
    if cast_result.completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        // Play sound at source position (where units teleport from)
        if let Some((source_x, source_z, _, _, _)) = cast_result.teleport_params {
            let source_pos = Vec3::new(source_x, 0.0, source_z);
            announce_area_cast(
                &mut commands,
                Spell::Teleport,
                source_pos,
                CIRCLE_RADIUS,
                primed_spell.empowerment,
            );
            audio::play_sfx(
                &mut commands,
                &sfx.teleport_cast,
                source_pos,
                &game_config,
                &sfx,
            );
        }

        // Track talent progress
        let teleported_count = cast_result.teleported_entities.len() as u32;
        if teleported_count > 0
            && let Some(progress) = talent_progress.as_deref_mut()
        {
            progress.increment(Spell::Teleport, teleported_count);
        }

        // Apply post-teleport talent effects directly to teleported entities
        if let (Some(source_pos), Some(dest_pos)) = (cast_result.source_pos, cast_result.dest_pos) {
            let rift_entity = apply_post_teleport_effects(
                &mut commands,
                &talent_params,
                source_pos,
                dest_pos,
                &cast_result.teleported_entities,
            );

            // Spawn VFX (spatial distortion)
            if let Some(rift_entity) = rift_entity {
                super::super::vfx_systems::spawn_rift_vfx(
                    &mut commands,
                    rift_entity,
                    source_pos,
                    dest_pos,
                    cast_result.effective_radius,
                );
            } else if talent_params.teleport_up {
                // Gravitational Surge: only VFX at the source (no destination)
                super::super::vfx_systems::spawn_teleport_vfx(
                    &mut commands,
                    source_pos,
                    source_pos,
                    cast_result.effective_radius,
                );
            } else {
                super::super::vfx_systems::spawn_teleport_vfx(
                    &mut commands,
                    source_pos,
                    dest_pos,
                    cast_result.effective_radius,
                );
            }
        }

        // Handle Lingering Gate: keep destination for reuse
        if cast_result.keep_destination {
            // Don't despawn the destination circle — mark as lingering
            if let Some(dest_entity) = caster.destination_circle {
                commands
                    .entity(dest_entity)
                    .insert(LingeringGateMarker::new(LINGERING_GATE_DURATION));
            }
            caster.lingering_gate_active = true;
        } else {
            if let Some(dest_entity) = caster.destination_circle {
                commands.entity(dest_entity).try_despawn();
            }
            caster.destination_circle = None;
        }

        if let Some(source_entity) = caster.source_circle {
            commands.entity(source_entity).try_despawn();
        }
        caster.source_circle = None;
        mouse_state.left_consumed = true;

        // Send teleport params over the network so the host can move units
        if let Some((source_x, source_z, dest_x, dest_z, radius)) = cast_result.teleport_params
            && let Some(ref mut conn) = connection
        {
            conn.outgoing_messages.push(
                crate::networking::protocol::NetworkMessage::TeleportUnits {
                    source_x,
                    source_z,
                    dest_x,
                    dest_z,
                    radius,
                },
            );
        }
    }

    if cast_result.first_phase_released {
        mouse_state.left_consumed = true;
    }
}
