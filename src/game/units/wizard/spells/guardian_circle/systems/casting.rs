use super::super::constants;
use crate::config::GameConfig;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::Team;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

use super::buff::apply_guardian_circle_buff;

/// Local wizard Guardian Circle casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_guardian_circle_casting(
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
    mut targets_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut attacker_hit_msg: MessageWriter<GuardianCircleHitAttackerMessage>,
    audio_ctx: (Res<SpellSfxAssets>, Res<GameConfig>),
    talent_resources: (
        Option<ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>>,
        Option<Res<ActiveTalents>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (sfx, game_config) = &audio_ctx;
    let (mut talent_progress, active_talents, mut pending_cast_events) = talent_resources;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::GuardianCircle {
        return;
    }

    // Calculate talent modifications
    let talents = active_talents.as_deref();
    let t1 = talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 1));

    let radius_mult = match t1 {
        Some(2) => constants::EXPANSIVE_AEGIS_RADIUS_MULT, // Expansive Aegis
        _ => 1.0,
    };
    let cast_time_mult = match t2 {
        Some(2) => constants::RAPID_DEPLOYMENT_CAST_MULT, // Rapid Deployment
        _ => 1.0,
    };

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        constants::CIRCLE_RADIUS * primed_spell.empowerment * radius_mult,
    );

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
                && mana.can_afford(constants::MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.guardian_circle_indicator.clone(),
                    pos,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment * radius_mult,
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

    let completed = guardian_circle_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        clamped_cursor,
        cast_time_mult,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Holy,
            time.elapsed_secs(),
        );
        // Apply buff using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let radius = constants::CIRCLE_RADIUS * primed_spell.empowerment * radius_mult;

                audio::play_sfx_synced(
                    &mut commands,
                    &mut pending_cast_events,
                    SpellSoundId::GuardianCircleCast,
                    indicator.position,
                    game_config,
                    sfx,
                );
                vfx::systems::spawn_aura_bubble_synced(
                    &mut commands,
                    &visual_assets,
                    &mut pending_cast_events,
                    visual_assets.guardian_aura_sphere.clone(),
                    crate::networking::snapshot::AuraBubbleVariant::Guardian,
                    indicator.position,
                    radius,
                    2.5,
                );
                apply_guardian_circle_buff(
                    &mut commands,
                    indicator.position,
                    radius,
                    constants::TEMP_HP_AMOUNT,
                    constants::TEMP_HP_DURATION,
                    primed_spell.empowerment,
                    &mut targets_query,
                    &mut attacker_hit_msg,
                    &mut talent_progress,
                    talents,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core Guardian Circle casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn guardian_circle_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    _clamped_cursor: Option<Vec3>,
    cast_time_mult: f32,
) -> bool {
    // Release is handled by the wrappers before calling this function
    if input.just_released {
        return false;
    }

    let mut completed = false;
    let effective_cast_time = primed_spell.cast_time * cast_time_mult;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(constants::MANA_COST) {
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
