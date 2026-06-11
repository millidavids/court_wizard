use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use super::super::components::SpikeGrowthTalentParams;
use super::super::constants;
use super::spawn::{spawn_minefield_zones, spawn_spike_growth_zone};
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    LocalSpellOrigin, SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist,
    build_wizard_input, clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster,
    handle_spell_release, try_start_cast_with_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> SpikeGrowthTalentParams {
    let mut params = SpikeGrowthTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::SpikeGrowth, 0);
    let t2 = talents.get_selection(Spell::SpikeGrowth, 1);
    let t3 = talents.get_selection(Spell::SpikeGrowth, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Wider Zone
            params.radius_mult = constants::WIDER_ZONE_RADIUS_MULT;
        }
        Some(1) => {
            // Sharper Spikes
            params.damage_mult = constants::SHARPER_SPIKES_DAMAGE_MULT;
        }
        Some(2) => {
            // Quick Bloom
            params.cast_time_mult = constants::QUICK_BLOOM_CAST_TIME_MULT;
            params.mana_mult = constants::QUICK_BLOOM_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.thorn_maze = true,
        Some(1) => params.poisoned_spikes = true,
        Some(2) => params.quicksand = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.death_garden = true,
        Some(1) => params.minefield = true,
        Some(2) => params.spike_storm = true,
        _ => {}
    }

    params
}

/// Local wizard spike growth casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_spike_growth_casting(
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
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        Option<Res<ActiveToggles>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let (active_talents, _talent_progress, active_toggles, mut pending_cast_events) =
        talent_resources;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::SpikeGrowth {
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
                    visual_assets.spike_growth_indicator.clone(),
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
                            audio::play_sfx_synced(
                                &mut commands,
                                &mut pending_cast_events,
                                SpellSoundId::SpikeGrowthCast,
                                indicator.position,
                                &game_config,
                                &sfx,
                            );

                            if talent_params.minefield {
                                spawn_minefield_zones(
                                    &mut commands,
                                    &visual_assets,
                                    indicator.position,
                                    effective_radius,
                                    primed_spell.empowerment,
                                    &mut obstacle_events,
                                    &talent_params,
                                    scorched_mult,
                                );
                            } else {
                                spawn_spike_growth_zone(
                                    &mut commands,
                                    &visual_assets,
                                    indicator.position,
                                    effective_radius,
                                    primed_spell.empowerment,
                                    &mut obstacle_events,
                                    &talent_params,
                                    scorched_mult,
                                );
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
