use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::HasteModifier;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::haste::components::{
    ChainHasteSource, FleetFeet, HasteSlowZone, HasteTalentParams, MomentumPending,
};
use crate::game::units::wizard::spells::haste::constants;
use crate::game::units::wizard::spells::messages::announce_area_cast;
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
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> HasteTalentParams {
    let mut params = HasteTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Haste, 0);
    let t2 = talents.get_selection(Spell::Haste, 1);
    let t3 = talents.get_selection(Spell::Haste, 2);

    match t1 {
        Some(0) => params.speed_mult = constants::ALACRITY_SPEED_MULT,
        Some(1) => params.duration_mult = constants::EXTENDED_RUSH_DURATION_MULT,
        Some(2) => params.cast_time_mult = constants::QUICK_CAST_CAST_TIME_MULT,
        _ => {}
    }

    match t2 {
        Some(0) => params.adrenaline_surge = true,
        Some(1) => params.momentum = true,
        Some(2) => params.fleet_feet = true,
        _ => {}
    }

    match t3 {
        Some(0) => params.time_warp = true,
        Some(1) => params.slow_zone = true,
        Some(2) => params.chain_haste = true,
        _ => {}
    }

    params
}

/// Local wizard haste casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_haste_casting(
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
        (Entity, &Transform, Option<&mut HasteModifier>),
        (
            Without<Wizard>,
            Without<GhostEntity>,
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
    if primed_spell.spell != Spell::Haste {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        constants::CIRCLE_RADIUS * primed_spell.empowerment,
    );

    // Handle release -- clean up indicator
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

    let mut completed = false;
    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.haste_indicator.clone(),
                    wizard_entity,
                    &mut casting_state,
                    &mana,
                    constants::MANA_COST,
                    clamped_cursor,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
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

            if casting_state.is_complete(cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                        && let Ok(indicator) = indicator_query.get(indicator_entity)
                    {
                        let radius = constants::CIRCLE_RADIUS * primed_spell.empowerment;
                        audio::play_sfx_synced(
                            &mut commands,
                            &mut pending_cast_events,
                            SpellSoundId::HasteCast,
                            indicator.position,
                            game_config,
                            sfx,
                        );
                        vfx::systems::spawn_aura_bubble_synced(
                            &mut commands,
                            &visual_assets,
                            &mut pending_cast_events,
                            visual_assets.haste_aura_sphere.clone(),
                            crate::networking::snapshot::AuraBubbleVariant::Haste,
                            indicator.position,
                            radius,
                            2.5,
                        );
                        announce_area_cast(
                            &mut commands,
                            Spell::Haste,
                            indicator.position,
                            radius,
                            primed_spell.empowerment,
                        );
                        let buffed_count = apply_haste_buff(
                            &mut commands,
                            indicator.position,
                            radius,
                            primed_spell.empowerment,
                            &talent_params,
                            &mut targets_query,
                        );

                        // Track talent progress
                        if buffed_count > 0
                            && let Some(progress) = talent_progress.as_deref_mut()
                        {
                            progress.increment(Spell::Haste, buffed_count);
                        }

                        // Spawn slow zone if talent is active
                        if talent_params.slow_zone {
                            commands.spawn((
                                HasteSlowZone {
                                    position: indicator.position,
                                    radius: constants::SLOW_ZONE_RADIUS * primed_spell.empowerment,
                                    time_remaining: constants::SLOW_ZONE_DURATION,
                                    slow_amount: constants::SLOW_ZONE_SLOW_AMOUNT,
                                },
                                OnGameplayScreen,
                            ));
                        }
                    }
                    cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
                    casting_state.cancel();
                    completed = true;
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

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Holy,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Applies haste buff to ALL units in radius (magic is indiscriminate).
/// Returns the number of units buffed.
#[allow(clippy::type_complexity)]
pub(crate) fn apply_haste_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &HasteTalentParams,
    targets: &mut Query<
        (Entity, &Transform, Option<&mut HasteModifier>),
        (
            Without<Wizard>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) -> u32 {
    let mut modifier = constants::HASTE_MODIFIER * empowerment * talent_params.speed_mult;
    let mut duration = constants::HASTE_DURATION * empowerment * talent_params.duration_mult;
    // Attack speed: granted by Adrenaline Surge or Time Warp (either one)
    let mut attack_speed = if talent_params.adrenaline_surge || talent_params.time_warp {
        constants::ADRENALINE_SURGE_ATTACK_SPEED
    } else {
        0.0
    };

    // Time Warp: double speed+attack bonuses, halve duration
    if talent_params.time_warp {
        modifier *= constants::TIME_WARP_BONUS_MULT;
        attack_speed *= constants::TIME_WARP_BONUS_MULT;
        duration *= constants::TIME_WARP_DURATION_MULT;
    }

    let mut buffed_count = 0u32;

    for (entity, transform, existing_haste) in targets.iter_mut() {
        let distance = crate::game::units::wizard::spells::utils::xz_distance(
            transform.translation,
            circle_pos,
        );
        if distance <= radius {
            if let Some(mut haste) = existing_haste {
                // Refresh duration if already hasted
                haste.refresh(duration);
                haste.modifier = modifier;
                haste.attack_speed = attack_speed;
            } else {
                commands
                    .entity(entity)
                    .insert(HasteModifier::with_attack_speed(
                        modifier,
                        duration,
                        attack_speed,
                    ));
            }

            // Insert talent-specific behavioral components
            let mut entity_cmds = commands.entity(entity);
            if talent_params.momentum {
                entity_cmds.insert(MomentumPending);
            }
            if talent_params.fleet_feet {
                entity_cmds.insert(FleetFeet::new(1));
            }
            if talent_params.chain_haste {
                entity_cmds.insert(ChainHasteSource {
                    hops_remaining: constants::CHAIN_HASTE_MAX_HOPS,
                    effectiveness: 1.0,
                    attack_speed,
                });
            }

            buffed_count += 1;
        }
    }

    buffed_count
}
