//! Black hole casting system and core casting logic.

use super::super::constants::*;
use super::spawn::spawn_black_hole;
use super::talents::compute_talent_params;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range, cleanup_spell_caster, spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use bevy::prelude::*;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

/// Local wizard Black Hole casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_black_hole_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::BlackHole {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let indicator_radius = MAX_RADIUS * primed_spell.empowerment * talent_params.radius_mult;

    // Check Twin Stars talent for mana cost multiplier
    let t3 = active_talents
        .as_deref()
        .and_then(|t| t.get_selection(Spell::BlackHole, 2));
    let mana_mult = if t3 == Some(1) {
        TWIN_STARS_MANA_MULT
    } else {
        1.0
    };

    let total_mana_cost = MANA_COST * mana_mult;

    // Clamp cursor to spell range for indicator positioning
    let clamped_cursor = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(total_mana_cost)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.black_hole_indicator.clone(),
                    pos,
                    indicator_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor {
                update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
        }
    }

    let cast_result = black_hole_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        mana_mult,
        local_origin.0,
    );

    if cast_result.completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Arcane,
            time.elapsed_secs(),
        );
        // Clean up indicator and SpellCaster
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);

        if let Some(pos) = cast_result.cursor_pos {
            if t3 == Some(1) {
                // Twin Stars: spawn 2 black holes offset from the target position
                let offset = Vec3::new(TWIN_STARS_OFFSET / 2.0, 0.0, 0.0);
                let emp = primed_spell.empowerment * TWIN_STARS_EFFECTIVENESS;
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos - offset,
                    emp,
                    sfx,
                    game_config,
                    talent_params,
                );
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos + offset,
                    emp,
                    sfx,
                    game_config,
                    talent_params,
                );
            } else {
                spawn_black_hole(
                    &mut commands,
                    &visual_assets,
                    pos,
                    primed_spell.empowerment,
                    sfx,
                    game_config,
                    talent_params,
                );
            }
        }
        mouse_state.left_consumed = true;
    }
}

/// Core Black Hole casting logic -- called by the local system.
///
/// Handles CastingState transitions, mana consumption, and cursor clamping.
/// Does NOT spawn the black hole or manage mouse_state -- those are the wrapper's job.
#[allow(clippy::too_many_arguments)]
fn black_hole_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    mana_mult: f32,
    local_origin: Vec3,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        cursor_pos: None,
    };

    let total_mana_cost = MANA_COST * mana_mult;

    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(total_mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if let Some(cursor_pos) = input.cursor_pos {
                    let clamped_pos =
                        clamp_to_spell_range(cursor_pos, local_origin, wizard.spell_range);

                    if mana.consume(total_mana_cost) {
                        result.completed = true;
                        result.cursor_pos = Some(clamped_pos);
                    }
                }

                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    result
}
