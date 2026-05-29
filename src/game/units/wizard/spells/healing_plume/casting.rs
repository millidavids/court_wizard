//! Healing plume casting and heal application.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::aura::{spawn_healing_plume_zone, try_convert_field_medic};
use super::components::{
    CleansingPlumeZone, HealingPlumeTalentParams, HealingPlumeZone, OverflowZone, TriagePulseZone,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, MarkedForDeathModifier, RootedModifier, SlowMovementModifier, Team,
    TemporaryHitPoints,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::networking::snapshot::SpellSoundId;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> HealingPlumeTalentParams {
    let mut params = HealingPlumeTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::HealingPlume, 0);
    let t2 = talents.get_selection(Spell::HealingPlume, 1);
    let t3 = talents.get_selection(Spell::HealingPlume, 2);

    match t1 {
        Some(0) => params.heal_mult = constants::REJUVENATING_MISTS_HEAL_MULT,
        Some(1) => params.radius_mult = constants::VERDANT_BLOOM_RADIUS_MULT,
        Some(2) => params.duration_mult = constants::LASTING_REMEDY_DURATION_MULT,
        _ => {}
    }

    match t2 {
        Some(0) => params.cleansing_plume = true,
        Some(1) => params.overflow = true,
        Some(2) => params.triage_pulse = true,
        _ => {}
    }

    match t3 {
        Some(0) => params.font_of_life = true,
        Some(1) => params.healing_rain = true,
        Some(2) => params.field_medic = true,
        _ => {}
    }

    params
}

/// Local wizard healing plume casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_healing_plume_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    toggle_resources: (Option<Res<ActiveTalents>>, Option<Res<ActiveToggles>>),
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
    defenders_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &MeshMaterial3d<StandardMaterial>,
            Has<crate::game::units::infantry::components::Infantry>,
            Has<crate::game::units::archer::components::Archer>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::units::healer::components::Healer>,
        ),
    >,
) {
    let (active_talents, active_toggles) = toggle_resources;
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::HealingPlume {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let radius = constants::CIRCLE_RADIUS * primed_spell.empowerment * talent_params.radius_mult;

    let clamped_cursor = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        radius,
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
                    visual_assets.healing_plume_indicator.clone(),
                    pos,
                    radius,
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
        healing_plume_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Holy,
            time.elapsed_secs(),
        );
        // Spawn healing zone using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let zone_entity = spawn_healing_plume_zone(
                    &mut commands,
                    &visual_assets,
                    indicator.position,
                    radius,
                    primed_spell.empowerment,
                    &talent_params,
                    scorched_mult,
                );

                // Field Medic: convert nearest defender in zone to healer
                if talent_params.field_medic {
                    try_convert_field_medic(
                        &mut commands,
                        zone_entity,
                        indicator.position,
                        radius,
                        &defenders_query,
                        &mut materials,
                    );
                }

                audio::play_sfx_synced(
                    &mut commands,
                    pending_cast_events,
                    SpellSoundId::HealingPlumeCast,
                    indicator.position,
                    game_config,
                    sfx,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core healing plume casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn healing_plume_casting_logic(
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

/// Applies periodic healing to all non-corpse units within the healing plume zone.
/// Integrates Tier 2 talents: Overflow (temp HP) and Triage Pulse (double heal below threshold).
/// Drought synergy: healing is reduced on dry units.
pub fn apply_healing_plume_heal(
    time: Res<Time>,
    mut zones: Query<(
        &mut HealingPlumeZone,
        Has<OverflowZone>,
        Has<TriagePulseZone>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<crate::game::units::wizard::archetypes::meteorologist::components::DryModifier>,
        ),
        Without<Corpse>,
    >,
    mut commands: Commands,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    visual_assets: Res<SpellVisualAssets>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    use crate::game::units::wizard::archetypes::meteorologist::systems::apply_dry_healing_reduction;

    let delta = time.delta_secs();

    let mote_interval = vfx::constants::MOTE_SPAWN_INTERVAL;
    let mote_count = vfx::constants::MOTE_COUNT_PER_SPAWN;

    for (mut zone, has_overflow, has_triage) in &mut zones {
        let prev_time = zone.time_alive;
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if (zone.time_alive / mote_interval).floor() != (prev_time / mote_interval).floor() {
            vfx::systems::spawn_floating_motes_synced(
                &mut commands,
                &visual_assets,
                &mut pending_cast_events,
                &visual_assets.healing_mote,
                crate::networking::snapshot::MoteMaterial::Healing,
                zone.origin,
                zone.radius,
                mote_count,
                time.elapsed_secs(),
            );
        }

        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, is_dry) in &mut targets {
                let distance = xz_distance(zone.origin, transform.translation);

                if distance <= zone.radius {
                    let mut heal_amount = zone.heal_per_tick;

                    // Triage Pulse: double healing for low-HP allies
                    if has_triage
                        && health.max > 0.0
                        && (health.current / health.max) < constants::TRIAGE_PULSE_HP_THRESHOLD
                    {
                        heal_amount *= constants::TRIAGE_PULSE_HEAL_MULT;
                    }

                    heal_amount = apply_dry_healing_reduction(heal_amount, is_dry);

                    let hp_before = health.current;
                    health.heal(heal_amount);
                    let actual_healed = health.current - hp_before;

                    // Track talent progress (health restored)
                    if actual_healed > 0.0
                        && let Some(ref mut progress) = talent_progress
                    {
                        progress.increment(Spell::HealingPlume, actual_healed as u32);
                    }

                    // Overflow: excess healing becomes temp HP
                    if has_overflow {
                        let excess = heal_amount - actual_healed;
                        if excess > 0.0 {
                            if let Some(ref mut existing_temp_hp) = temp_hp {
                                let new_amount = (existing_temp_hp.amount + excess)
                                    .min(constants::OVERFLOW_MAX_TEMP_HP);
                                existing_temp_hp.amount = new_amount;
                                existing_temp_hp.time_remaining =
                                    constants::OVERFLOW_TEMP_HP_DURATION;
                            } else {
                                let amount = excess.min(constants::OVERFLOW_MAX_TEMP_HP);
                                commands.entity(entity).insert(TemporaryHitPoints::new(
                                    amount,
                                    constants::OVERFLOW_TEMP_HP_DURATION,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Tier 2: Cleansing Plume — periodically removes debuffs from all units inside the zone.
pub fn apply_cleansing_plume(
    time: Res<Time>,
    mut zones: Query<(&HealingPlumeZone, &mut CleansingPlumeZone)>,
    targets: Query<
        (
            Entity,
            &Transform,
            Has<SlowMovementModifier>,
            Has<RootedModifier>,
            Has<MarkedForDeathModifier>,
        ),
        Without<Corpse>,
    >,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (zone, mut cleansing) in &mut zones {
        cleansing.time_since_last_cleanse += delta;

        if cleansing.time_since_last_cleanse >= constants::CLEANSING_PLUME_INTERVAL {
            cleansing.time_since_last_cleanse = 0.0;

            for (entity, transform, has_slow, has_root, has_mark) in &targets {
                let distance = xz_distance(zone.origin, transform.translation);

                if distance <= zone.radius {
                    if has_slow {
                        commands.entity(entity).remove::<SlowMovementModifier>();
                    }
                    if has_root {
                        commands.entity(entity).remove::<RootedModifier>();
                    }
                    if has_mark {
                        commands.entity(entity).remove::<MarkedForDeathModifier>();
                    }
                }
            }
        }
    }
}
