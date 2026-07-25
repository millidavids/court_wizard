use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use super::super::components::EntangleTalentParams;
use super::super::constants;
use super::super::vines::apply_entangle;
use crate::config::GameConfig;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::components::Corpse;
use crate::game::units::components::Team;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, handle_spell_release,
    try_start_cast_with_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> EntangleTalentParams {
    let mut params = EntangleTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Entangle, 0);
    let t2 = talents.get_selection(Spell::Entangle, 1);
    let t3 = talents.get_selection(Spell::Entangle, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Deep Roots
            params.duration_mult = constants::DEEP_ROOTS_DURATION_MULT;
        }
        Some(1) => {
            // Sprawling Thicket
            params.radius_mult = constants::SPRAWLING_THICKET_RADIUS_MULT;
            params.mana_mult = constants::SPRAWLING_THICKET_MANA_MULT;
        }
        Some(2) => {
            // Efficient Growth
            params.mana_mult = constants::EFFICIENT_GROWTH_MANA_MULT;
            params.cast_time_mult = constants::EFFICIENT_GROWTH_CAST_TIME_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.thorny_vines = true,
        Some(1) => params.clinging_roots = true,
        Some(2) => params.nourishing_roots = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.overgrowth = true,
        Some(1) => params.sanctuary = true,
        Some(2) => params.stranglehold = true,
        _ => {}
    }

    params
}

/// Local wizard entangle casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_entangle_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mesh_and_materials: (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
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
    targets_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Wizard>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    messages: (
        MessageWriter<EntangleHitDefenderMessage>,
        MessageWriter<ObstacleChanged>,
    ),
    config_and_talents: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        Option<Res<ActiveToggles>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (mut defender_hit_msg, mut obstacle_events) = messages;
    let (
        sfx,
        game_config,
        active_talents,
        mut talent_progress,
        active_toggles,
        mut pending_cast_events,
    ) = config_and_talents;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let (mut meshes, mut materials) = mesh_and_materials;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Entangle {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let effective_radius =
        constants::CIRCLE_RADIUS * primed_spell.empowerment * talent_params.radius_mult;
    let effective_mana_cost = constants::MANA_COST * talent_params.mana_mult;
    let effective_cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        effective_radius,
    );

    // Handle release — clean up indicator
    if handle_spell_release(
        &input,
        &mut commands,
        wizard_entity,
        &mut casting_state,
        &caster_query,
    ) {
        return;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.entangle_indicator.clone(),
                    wizard_entity,
                    &mut casting_state,
                    &mana,
                    effective_mana_cost,
                    clamped_cursor,
                    effective_radius,
                    &caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update indicator position
            update_indicator_position(
                wizard_entity,
                clamped_cursor,
                &caster_query,
                &mut indicator_query,
            );

            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(effective_mana_cost) {
                    vfx::systems::spawn_school_flare_synced(
                        &mut commands,
                        &visual_assets,
                        &mut pending_cast_events,
                        local_origin.0,
                        vfx::systems::SpellSchool::Nature,
                        time.elapsed_secs(),
                    );
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let cast_pos = indicator.position;
                            let root_duration = constants::ROOT_DURATION
                                * primed_spell.empowerment
                                * talent_params.duration_mult
                                * scorched_mult;
                            audio::play_sfx_synced(
                                &mut commands,
                                &mut pending_cast_events,
                                SpellSoundId::EntangleCast,
                                cast_pos,
                                &game_config,
                                &sfx,
                            );
                            let hit_count = apply_entangle(
                                &mut game_rng.0,
                                &mut commands,
                                &visual_assets,
                                &mut materials,
                                cast_pos,
                                effective_radius,
                                root_duration,
                                &targets_query,
                                &mut defender_hit_msg,
                                &mut obstacle_events,
                                &talent_params,
                            );
                            // Track talent progress
                            if hit_count > 0
                                && let Some(ref mut progress) = talent_progress
                            {
                                progress.increment(Spell::Entangle, hit_count);
                            }
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
                } else {
                    cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
            casting_state.cancel();
        }
    }
}
