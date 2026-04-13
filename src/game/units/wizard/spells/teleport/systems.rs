//! Systems for the Teleport spell.

use bevy::prelude::*;
use rand::Rng;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    DimensionalRift, DisorientingHaste, LingeringGateMarker, RiftCooldown, TeleportCaster,
    TeleportDestinationCircle, TeleportSourceCircle, TeleportTalentParams,
};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::{
    BATTLEFIELD_SIZE, DEFENDER_GRID_CENTER_ANGLE, DEFENDER_GRID_GROUND_RANGE, SPELL_ORIGIN,
    WIZARD_POSITION,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::{MouseLeftReleased, MouseRightPressed};
use crate::game::units::DamageType;
use crate::game::units::components::{Airborne, Corpse, Stunned, Team, Teleportable};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, clamp_to_spell_range,
    xz_distance,
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

/// Result from teleport casting logic, used to communicate state back to the wrapper.
struct TeleportCastResult {
    /// Whether the spell completed (teleport executed).
    completed: bool,
    /// Whether the first phase was finalized (destination locked in on release).
    first_phase_released: bool,
    /// Teleport parameters for network sync: (source_x, source_z, dest_x, dest_z, radius).
    teleport_params: Option<(f32, f32, f32, f32, f32)>,
    /// Entities that were teleported (for talent effects and progress tracking).
    teleported_entities: Vec<Entity>,
    /// Whether to keep the destination (Lingering Gate talent).
    keep_destination: bool,
    /// Source and dest positions for post-teleport effects.
    source_pos: Option<Vec3>,
    dest_pos: Option<Vec3>,
    /// The effective radius used for this teleport.
    effective_radius: f32,
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
        ),
    >,
    target_assist: Res<TargetAssistWorldPos>,
    (mut connection, sfx, game_config, active_talents, mut talent_progress): (
        Option<ResMut<NetworkConnection>>,
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
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
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    let cast_result = teleport_casting_logic(
        &mut game_rng.0,
        &input,
        &time,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &visual_assets,
        &units_query,
        &source_query,
        &talent_params,
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
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        // Play sound at source position (where units teleport from)
        if let Some((source_x, source_z, _, _, _)) = cast_result.teleport_params {
            let source_pos = Vec3::new(source_x, 0.0, source_z);
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
                super::vfx_systems::spawn_rift_vfx(
                    &mut commands,
                    rift_entity,
                    source_pos,
                    dest_pos,
                    cast_result.effective_radius,
                );
            } else if talent_params.teleport_up {
                // Gravitational Surge: only VFX at the source (no destination)
                super::vfx_systems::spawn_teleport_vfx(
                    &mut commands,
                    source_pos,
                    source_pos,
                    cast_result.effective_radius,
                );
            } else {
                super::vfx_systems::spawn_teleport_vfx(
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

/// Core Teleport casting logic — called by the local casting system.
///
/// Handles the two-phase state machine:
/// Phase 1: Click to start casting, release to lock destination position.
/// Phase 2: Click again to start source circle growth, cast completes on timer or early release.
///
/// With Emergency Recall talent: skips Phase 1, goes straight to source circle.
/// With Swap talent: captures units at both source and destination, then swaps them.
/// With Scatterport talent: scatters enemies to random locations instead of teleporting to dest.
#[allow(clippy::too_many_arguments)]
fn teleport_casting_logic(
    rng: &mut impl Rng,
    input: &WizardInput,
    time: &Time,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    _primed_spell: &PrimedSpell,
    caster: &mut TeleportCaster,
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    source_query: &Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
    talent_params: &TeleportTalentParams,
) -> TeleportCastResult {
    let mut result = TeleportCastResult {
        completed: false,
        first_phase_released: false,
        teleport_params: None,
        teleported_entities: Vec::new(),
        keep_destination: false,
        source_pos: None,
        dest_pos: None,
        effective_radius: 0.0,
    };

    let effective_circle_radius = CIRCLE_RADIUS * talent_params.radius_mult;
    let effective_cast_time = SECOND_CAST_TIME * talent_params.cast_time_mult;
    let effective_mana_cost = if talent_params.scatterport {
        MANA_COST * SCATTERPORT_MANA_MULT
    } else {
        MANA_COST
    };

    // Gravitational Surge: skip Phase 1, single-cast at cursor (no destination needed).
    // Sets a dummy destination so the state machine enters Phase 2 immediately.
    if talent_params.teleport_up && !caster.has_destination() {
        caster.destination_position = Some(Vec3::ZERO);
    }

    // Emergency Recall: skip Phase 1, always set destination to castle entrance.
    // Overrides any lingering gate destination — recall always goes home.
    if talent_params.emergency_recall && !caster.has_destination() {
        // Recall to King's spawn position (same formula as spawn_king)
        let angle = DEFENDER_GRID_CENTER_ANGLE;
        let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
        let king_spawn = Vec3::new(
            WIZARD_POSITION.x + radius * angle.cos(),
            0.0,
            WIZARD_POSITION.z + radius * angle.sin(),
        );
        caster.destination_position = Some(king_spawn);
    }

    // Handle release during first cast — finalize destination position
    if input.just_released
        && !caster.has_destination()
        && matches!(*casting_state, CastingState::Casting { .. })
    {
        if let Some(pos) = clamped_pos {
            caster.destination_position = Some(pos);
            casting_state.cancel(); // Return to resting for phase 2
            result.first_phase_released = true;
        }
        return result;
    }

    // Handle release during second cast — completes teleport early
    if input.just_released
        && caster.has_destination()
        && caster.source_circle.is_some()
        && let CastingState::Casting { elapsed } = *casting_state
    {
        if let Some(source_entity) = caster.source_circle
            && let Ok((transform, source_circle)) = source_query.get(source_entity)
        {
            let source_pos = transform.translation;
            let growth = (elapsed / effective_cast_time).min(1.0);
            let scale = source_circle.empowerment;
            let current_radius = effective_circle_radius * scale * growth;

            if mana.can_afford(effective_mana_cost) {
                mana.consume(effective_mana_cost);

                if let Some(dest_pos) = caster.destination_position {
                    // Origin bubble: fades in large and contracts to a point
                    vfx::systems::spawn_aura_bubble_contracting(
                        commands,
                        visual_assets,
                        visual_assets.teleport_aura_sphere.clone(),
                        source_pos,
                        current_radius,
                        1.0,
                    );
                    // Destination bubble: expands out
                    vfx::systems::spawn_aura_bubble(
                        commands,
                        visual_assets,
                        visual_assets.teleport_aura_sphere.clone(),
                        dest_pos,
                        current_radius,
                        1.5,
                    );
                    let entities = execute_teleport(
                        rng,
                        source_pos,
                        dest_pos,
                        current_radius,
                        units_query,
                        commands,
                        talent_params,
                    );
                    result.teleport_params = Some((
                        source_pos.x,
                        source_pos.z,
                        dest_pos.x,
                        dest_pos.z,
                        current_radius,
                    ));
                    result.teleported_entities = entities;
                    result.source_pos = Some(source_pos);
                    result.dest_pos = Some(dest_pos);
                    result.effective_radius = current_radius;
                }

                // Lingering Gate: keep destination for a second teleport
                if talent_params.lingering_gate && !caster.lingering_gate_active {
                    result.keep_destination = true;
                } else {
                    caster.destination_position = None;
                    caster.lingering_gate_active = false;
                }

                casting_state.cancel();
                result.completed = true;
            }
        }
        return result;
    }

    let Some(_clamped_pos) = clamped_pos else {
        return result;
    };

    // State machine based on whether destination exists
    if !caster.has_destination() {
        // PHASE 1: Placing destination
        match *casting_state {
            CastingState::Resting => {
                if input.just_pressed || input.pressed {
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { .. } => {
                // Position update handled by local wrapper
            }
            _ => {}
        }
    } else {
        // PHASE 2: Placing source circle and teleporting
        match *casting_state {
            CastingState::Resting => {
                if !mana.can_afford(effective_mana_cost) {
                    return result;
                }
                if input.just_pressed || input.pressed {
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { ref mut elapsed } => {
                *elapsed += time.delta_secs();

                // Check if cast complete
                if *elapsed >= effective_cast_time
                    && let Some(source_entity) = caster.source_circle
                    && let Ok((transform, source_circle)) = source_query.get(source_entity)
                {
                    let source_pos = transform.translation;
                    let radius = effective_circle_radius * source_circle.empowerment;

                    mana.consume(effective_mana_cost);

                    if let Some(dest_pos) = caster.destination_position {
                        vfx::systems::spawn_aura_bubble(
                            commands,
                            visual_assets,
                            visual_assets.teleport_aura_sphere.clone(),
                            source_pos,
                            radius,
                            1.0,
                        );
                        vfx::systems::spawn_aura_bubble(
                            commands,
                            visual_assets,
                            visual_assets.teleport_aura_sphere.clone(),
                            dest_pos,
                            radius,
                            1.5,
                        );
                        let entities = execute_teleport(
                            rng,
                            source_pos,
                            dest_pos,
                            radius,
                            units_query,
                            commands,
                            talent_params,
                        );
                        result.teleport_params =
                            Some((source_pos.x, source_pos.z, dest_pos.x, dest_pos.z, radius));
                        result.teleported_entities = entities;
                        result.source_pos = Some(source_pos);
                        result.dest_pos = Some(dest_pos);
                        result.effective_radius = radius;
                    }

                    // Lingering Gate: keep destination for a second teleport
                    if talent_params.lingering_gate && !caster.lingering_gate_active {
                        result.keep_destination = true;
                    } else {
                        caster.destination_position = None;
                        caster.lingering_gate_active = false;
                    }

                    casting_state.cancel();
                    result.completed = true;
                }
            }
            _ => {}
        }
    }

    result
}

/// Executes the actual teleportation, handling talent variants.
/// Returns the list of teleported entities for post-teleport effect application.
fn execute_teleport(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
    talent_params: &TeleportTalentParams,
) -> Vec<Entity> {
    if talent_params.teleport_up {
        teleport_units_up(source_center, radius, units_query, commands)
    } else if talent_params.scatterport {
        scatter_enemies(rng, source_center, radius, units_query, commands)
    } else if talent_params.swap_mode {
        swap_units(rng, source_center, dest_center, radius, units_query, commands)
    } else if talent_params.emergency_recall {
        recall_allies(rng, source_center, dest_center, radius, units_query, commands)
    } else {
        teleport_units_with_radius(rng, source_center, dest_center, radius, units_query, commands)
    }
}

/// Up: teleports all units within radius straight up into the air.
/// They fall back down and take fall damage on landing.
fn teleport_units_up(
    center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, center) <= radius {
            // Place the unit high in the air with zero velocity — it falls from there.
            // The airborne system handles visual Y offset via base_y + height.
            commands.entity(entity).insert(Airborne {
                vertical_velocity: 0.0,
                height: TELEPORT_UP_HEIGHT,
                base_y: transform.translation.y,
                gravity: TELEPORT_UP_GRAVITY,
                damage_type: DamageType::Force,
            });
            teleported.push(entity);
        }
    }

    teleported
}

/// Teleports all units within a specified radius of the source center to random positions
/// within the same radius of the destination center.
/// Returns the list of teleported entity IDs.
pub(crate) fn teleport_units_with_radius(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            let new_position =
                random_position_in_circle(rng, dest_center, radius, transform.translation.y);
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Scatterport talent: scatters all units to random locations in a large radius.
fn scatter_enemies(
    rng: &mut impl Rng,
    source_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            let new_position = random_position_in_circle(
                rng,
                source_center,
                SCATTERPORT_RADIUS,
                transform.translation.y,
            );
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Swap talent: swaps all units between two circles simultaneously.
fn swap_units(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    // Collect units in each circle first (can't modify while iterating)
    let mut source_units: Vec<(Entity, Vec3)> = Vec::new();
    let mut dest_units: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            source_units.push((entity, transform.translation));
        } else if xz_distance(transform.translation, dest_center) <= radius {
            dest_units.push((entity, transform.translation));
        }
    }

    // Move source units to destination
    for (entity, original_pos) in &source_units {
        let new_position = random_position_in_circle(rng, dest_center, radius, original_pos.y);
        commands
            .entity(*entity)
            .insert(Transform::from_translation(new_position));
        teleported.push(*entity);
    }

    // Move destination units to source
    for (entity, original_pos) in &dest_units {
        let new_position =
            random_position_in_circle(rng, source_center, radius, original_pos.y);
        commands
            .entity(*entity)
            .insert(Transform::from_translation(new_position));
        teleported.push(*entity);
    }

    teleported
}

/// Emergency Recall talent: teleports only allied (Defender) units to the King's spawn position.
fn recall_allies(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, team) in units_query.iter() {
        // Only recall defenders (skip entities without a team, like rocks)
        if team != Some(&Team::Defenders) {
            continue;
        }

        if xz_distance(transform.translation, source_center) <= radius {
            let new_position =
                random_position_in_circle(rng, dest_center, radius, transform.translation.y);
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Applies post-teleport talent effects (stun, haste, dimensional rift).
/// Effects are applied directly to the teleported entities rather than querying by position,
/// since teleport uses deferred commands (transforms haven't been applied yet).
/// Returns the rift entity if Dimensional Rift was spawned.
fn apply_post_teleport_effects(
    commands: &mut Commands,
    talent_params: &TeleportTalentParams,
    source_pos: Vec3,
    dest_pos: Vec3,
    teleported_entities: &[Entity],
) -> Option<Entity> {
    // Disorienting Arrival: stun AND haste all teleported units
    if talent_params.disorienting_arrival {
        for &entity in teleported_entities {
            commands
                .entity(entity)
                .insert(Stunned::new(DISORIENTING_STUN_DURATION));
            commands.entity(entity).insert(DisorientingHaste::new(
                DISORIENTING_ATTACK_SPEED,
                DISORIENTING_ATTACK_SPEED_DURATION,
            ));
        }
    }

    // Dimensional Rift: spawn persistent two-way portal
    if talent_params.dimensional_rift {
        let rift_entity = commands
            .spawn((
                DimensionalRift {
                    source_pos,
                    dest_pos,
                    walk_radius: DIMENSIONAL_RIFT_WALK_RADIUS,
                    time_remaining: DIMENSIONAL_RIFT_DURATION,
                    two_way: talent_params.swap_mode,
                },
                OnGameplayScreen,
            ))
            .id();
        return Some(rift_entity);
    }

    None
}

/// Ticks Dimensional Rift portals and teleports units that walk through them.
pub fn tick_dimensional_rift(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut rifts: Query<(Entity, &mut DimensionalRift)>,
    mut units: Query<
        (Entity, &mut Transform, Option<&RiftCooldown>),
        (With<Teleportable>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();

    for (rift_entity, mut rift) in rifts.iter_mut() {
        rift.time_remaining -= delta;
        if rift.time_remaining <= 0.0 {
            commands.entity(rift_entity).try_despawn();
            continue;
        }

        let rng = &mut game_rng.0;

        for (unit_entity, mut transform, cooldown) in units.iter_mut() {
            // Skip units on cooldown from recent rift teleport
            if cooldown.is_some() {
                continue;
            }

            let pos = transform.translation;

            if xz_distance(pos, rift.source_pos) <= rift.walk_radius {
                // Near source portal → teleport to destination
                let new_pos =
                    random_position_in_circle(rng, rift.dest_pos, rift.walk_radius, pos.y);
                transform.translation = new_pos;
                commands.entity(unit_entity).insert(RiftCooldown {
                    time_remaining: DIMENSIONAL_RIFT_UNIT_COOLDOWN,
                });
            } else if rift.two_way && xz_distance(pos, rift.dest_pos) <= rift.walk_radius {
                // Near destination portal → teleport to source (only with Swap talent)
                let new_pos =
                    random_position_in_circle(rng, rift.source_pos, rift.walk_radius, pos.y);
                transform.translation = new_pos;
                commands.entity(unit_entity).insert(RiftCooldown {
                    time_remaining: DIMENSIONAL_RIFT_UNIT_COOLDOWN,
                });
            }
        }
    }
}

/// Ticks Lingering Gate markers and removes expired ones.
pub fn tick_lingering_gate(
    mut commands: Commands,
    time: Res<Time>,
    mut gates: Query<(Entity, &mut LingeringGateMarker)>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
) {
    let delta = time.delta_secs();

    for (gate_entity, mut gate) in gates.iter_mut() {
        gate.time_remaining -= delta;
        if gate.time_remaining <= 0.0 {
            commands.entity(gate_entity).try_despawn();

            // Reset caster state when gate expires
            if let Ok(mut caster) = caster_query.single_mut()
                && caster.destination_circle == Some(gate_entity)
            {
                caster.destination_circle = None;
                caster.destination_position = None;
                caster.lingering_gate_active = false;
            }
        }
    }
}

/// Updates pulse animations for both destination and source circles.
pub fn update_circle_animations(
    time: Res<Time>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        Without<TeleportSourceCircle>,
    >,
    mut source_query: Query<(&mut Transform, &mut TeleportSourceCircle)>,
) {
    // Update destination circles
    for (mut transform, mut indicator) in &mut destination_query {
        indicator.time_alive += time.delta_secs();

        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= indicator.base_radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(indicator.base_radius * pulse);
        }
    }

    // Update source circles
    for (mut transform, mut indicator) in &mut source_query {
        indicator.time_alive += time.delta_secs();

        let radius = CIRCLE_RADIUS * indicator.empowerment;
        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(radius * pulse);
        }
    }
}

/// Generates a random position within a circle, clamped to battlefield bounds.
fn random_position_in_circle(rng: &mut impl Rng, center: Vec3, radius: f32, y: f32) -> Vec3 {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let random_radius = rng.gen_range(0.0..radius);

    let new_x = (center.x + angle.cos() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);
    let new_z = (center.z + angle.sin() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);

    Vec3::new(new_x, y, new_z)
}

/// Cleans up teleport circles and caster state when the player switches away
/// from the Teleport spell while a teleport is in progress.
pub fn cleanup_teleport_on_spell_switch(
    mut commands: Commands,
    mut wizard_query: Query<(&PrimedSpell, &mut CastingState), (With<LocalWizard>, Changed<PrimedSpell>)>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
) {
    let Ok((primed_spell, mut casting_state)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell == Spell::Teleport {
        return;
    }

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    // Only clean up if there's actually an in-progress teleport (circles exist).
    // TeleportCaster persists on the wizard, so we must check for active state.
    let has_circles =
        caster.destination_circle.is_some() || caster.source_circle.is_some();
    if !has_circles {
        return;
    }

    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).try_despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).try_despawn();
    }
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    caster.lingering_gate_active = false;
    if !matches!(*casting_state, CastingState::Resting) {
        casting_state.cancel();
    }
}
