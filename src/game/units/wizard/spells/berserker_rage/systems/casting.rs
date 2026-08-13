use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::super::components::{
    Bloodlust, ContagiousRage, FinalStand, Frenzy, FrenzyActive, UndyingFury, UndyingFuryActive,
};
use super::super::constants;
use super::buff_application::{apply_berserker_rage_buff, compute_talent_params};
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{BerserkerRageModifier, Corpse, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::messages::announce_area_cast;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

pub(crate) type BerserkerCleanupFilter = (
    Without<BerserkerRageModifier>,
    Or<(
        With<Bloodlust>,
        With<Frenzy>,
        With<FrenzyActive>,
        With<UndyingFury>,
        With<UndyingFuryActive>,
        With<ContagiousRage>,
        With<FinalStand>,
    )>,
    Without<Corpse>,
);

/// Local wizard berserker rage casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_berserker_rage_casting(
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
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut BerserkerRageModifier>,
        ),
        (
            Without<Wizard>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    audio_ctx: (Res<SpellSfxAssets>, Res<GameConfig>),
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (sfx, game_config) = &audio_ctx;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::BerserkerRage {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let base_radius = constants::CIRCLE_RADIUS * talent_params.radius_mult;

    let clamped_cursor = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        base_radius * primed_spell.empowerment,
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
                    visual_assets.berserker_rage_indicator.clone(),
                    pos,
                    base_radius * primed_spell.empowerment,
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
        berserker_rage_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Transmutation,
            time.elapsed_secs(),
        );
        // Apply buff using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let radius = base_radius * primed_spell.empowerment;
                vfx::systems::spawn_aura_bubble_synced(
                    &mut commands,
                    &visual_assets,
                    &mut pending_cast_events,
                    visual_assets.berserker_aura_sphere.clone(),
                    crate::networking::snapshot::AuraBubbleVariant::Berserker,
                    indicator.position,
                    radius,
                    2.5,
                );
                announce_area_cast(
                    &mut commands,
                    Spell::BerserkerRage,
                    indicator.position,
                    radius,
                    primed_spell.empowerment,
                );
                let buffed_count = apply_berserker_rage_buff(
                    &mut commands,
                    indicator.position,
                    radius,
                    primed_spell.empowerment,
                    &talent_params,
                    &mut targets_query,
                );
                audio::play_sfx_synced(
                    &mut commands,
                    &mut pending_cast_events,
                    SpellSoundId::BerserkerRageCast,
                    indicator.position,
                    game_config,
                    sfx,
                );
                // Track talent progress
                if buffed_count > 0
                    && let Some(ref mut progress) = talent_progress
                {
                    progress.increment(Spell::BerserkerRage, buffed_count);
                }
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core berserker rage casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn berserker_rage_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> bool {
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
