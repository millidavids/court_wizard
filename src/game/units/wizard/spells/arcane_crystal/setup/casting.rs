use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::helpers::compute_talent_params;
use super::spawn::spawn_crystal;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range, cleanup_spell_caster, handle_spell_release, spawn_circle_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;

// ===== Casting System =====

/// Local wizard Arcane Crystal casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_arcane_crystal_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    existing_crystals: Query<(Entity, &ArcaneCrystal)>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::ArcaneCrystal {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    // Handle release -- clean up indicator and SpellCaster
    if handle_spell_release(
        &input,
        &mut commands,
        wizard_entity,
        &mut casting_state,
        &caster_query,
    ) {
        return;
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.arcane_crystal_indicator.clone(),
                    pos,
                    CRYSTAL_RANGE * primed_spell.empowerment,
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

    let completed =
        arcane_crystal_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Arcane,
            time.elapsed_secs(),
        );
        let talent_params = compute_talent_params(active_talents.as_deref());

        if talent_params.auto_crystal {
            // Auto-Crystal (turret): only 1 permanent turret crystal allowed per level.
            // Count permanent crystals already placed this level.
            let placed_this_level = existing_crystals
                .iter()
                .filter(|(_, c)| !c.permanent)
                .count();
            if placed_this_level > 0 {
                // Already placed one this level — block
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
                // Refund mana since we blocked the cast
                mana.current = (mana.current + MANA_COST).min(mana.max);
                return;
            }
        } else if !talent_params.crystal_network {
            // Default: despawn existing crystals (non-permanent)
            for (crystal_entity, _) in &existing_crystals {
                commands.entity(crystal_entity).try_despawn();
            }
        } else {
            // Crystal Network: allow up to 3 crystals; despawn an arbitrary one if at limit
            let count = existing_crystals.iter().count();
            if count >= CRYSTAL_NETWORK_MAX_CRYSTALS
                && let Some((oldest, _)) = existing_crystals.iter().next()
            {
                commands.entity(oldest).try_despawn();
            }
        }

        // Spawn crystal using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                spawn_crystal(
                    &mut commands,
                    &visual_assets,
                    indicator.position,
                    primed_spell.empowerment,
                    &talent_params,
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.arcane_crystal_cast,
                    indicator.position,
                    &game_config,
                    &sfx,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core Arcane Crystal casting logic -- handles CastingState transitions and mana consumption.
///
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn arcane_crystal_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> bool {
    // Release is handled by the wrappers before calling this function
    if input.just_released {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                casting_state.start_cast();
            }
        }
    }

    completed
}
