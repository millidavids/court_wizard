//! Core two-phase state machine for the Teleport spell.

use bevy::prelude::*;
use rand::Rng;

use super::super::arrival::execute_teleport;
use super::super::components::{
    TeleportCaster, TeleportDestinationCircle, TeleportSourceCircle, TeleportTalentParams,
};
use super::super::constants::*;
use crate::game::units::components::{Corpse, Team, Teleportable};
use crate::game::units::wizard::components::{CastingState, Mana, WizardInput};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Result from teleport casting logic, used to communicate state back to the wrapper.
pub(super) struct TeleportCastResult {
    /// Whether the spell completed (teleport executed).
    pub(super) completed: bool,
    /// Whether the first phase was finalized (destination locked in on release).
    pub(super) first_phase_released: bool,
    /// Teleport parameters for network sync: (source_x, source_z, dest_x, dest_z, radius).
    pub(super) teleport_params: Option<(f32, f32, f32, f32, f32)>,
    /// Entities that were teleported (for talent effects and progress tracking).
    pub(super) teleported_entities: Vec<Entity>,
    /// Whether to keep the destination (Lingering Gate talent).
    pub(super) keep_destination: bool,
    /// Source and dest positions for post-teleport effects.
    pub(super) source_pos: Option<Vec3>,
    pub(super) dest_pos: Option<Vec3>,
    /// The effective radius used for this teleport.
    pub(super) effective_radius: f32,
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
pub(super) fn teleport_casting_logic(
    rng: &mut impl Rng,
    input: &WizardInput,
    time: &Time,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
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
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> TeleportCastResult {
    use crate::game::constants::{
        DEFENDER_GRID_CENTER_ANGLE, DEFENDER_GRID_GROUND_RANGE, WIZARD_POSITION,
    };

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
                    vfx::systems::spawn_aura_bubble_contracting_synced(
                        commands,
                        visual_assets,
                        pending,
                        visual_assets.teleport_aura_sphere.clone(),
                        crate::networking::snapshot::AuraBubbleVariant::Teleport,
                        source_pos,
                        current_radius,
                        1.0,
                    );
                    // Destination bubble: expands out
                    vfx::systems::spawn_aura_bubble_synced(
                        commands,
                        visual_assets,
                        pending,
                        visual_assets.teleport_aura_sphere.clone(),
                        crate::networking::snapshot::AuraBubbleVariant::Teleport,
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

                    // Re-check affordability at completion: mana may have been
                    // spent elsewhere during the windup. If we can't afford it,
                    // the cast still ends (below) but no teleport happens.
                    let can_pay = mana.can_afford(effective_mana_cost);
                    if can_pay {
                        mana.consume(effective_mana_cost);
                    }

                    if can_pay && let Some(dest_pos) = caster.destination_position {
                        vfx::systems::spawn_aura_bubble_synced(
                            commands,
                            visual_assets,
                            pending,
                            visual_assets.teleport_aura_sphere.clone(),
                            crate::networking::snapshot::AuraBubbleVariant::Teleport,
                            source_pos,
                            radius,
                            1.0,
                        );
                        vfx::systems::spawn_aura_bubble_synced(
                            commands,
                            visual_assets,
                            pending,
                            visual_assets.teleport_aura_sphere.clone(),
                            crate::networking::snapshot::AuraBubbleVariant::Teleport,
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
