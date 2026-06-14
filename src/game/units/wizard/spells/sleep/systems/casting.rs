use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::super::components::SleepTalentParams;
use super::super::constants;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

use super::effects::apply_sleep;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> SleepTalentParams {
    let mut params = SleepTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Sleep, 0);
    let t2 = talents.get_selection(Spell::Sleep, 1);
    let t3 = talents.get_selection(Spell::Sleep, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Deep Slumber: +40% duration
            params.duration_mult = constants::DEEP_SLUMBER_DURATION_MULT;
        }
        Some(1) => {
            // Lullaby: +40% radius
            params.radius_mult = constants::LULLABY_RADIUS_MULT;
        }
        Some(2) => {
            // Nightmare Fuel: +50% wake-up bonus damage
            params.bonus_damage_mult = constants::NIGHTMARE_FUEL_BONUS_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.narcoleptic_wave = true,
        Some(1) => params.night_terrors = true,
        Some(2) => {
            // Drowsy: halved cast time, -25% mana
            params.cast_time_mult = constants::DROWSY_CAST_TIME_MULT;
            params.mana_mult = constants::DROWSY_MANA_MULT;
        }
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.comatose = true,
        Some(1) => params.dreamwalker = true,
        Some(2) => params.eternal_slumber = true,
        _ => {}
    }

    params
}

/// Local wizard sleep casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_sleep_casting(
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
    targets_query: Query<(Entity, &Transform, &Health, &Team), (Without<Corpse>, Without<Wizard>)>,
    sfx_ctx: (Res<SpellSfxAssets>, Res<GameConfig>),
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let (sfx, game_config) = &sfx_ctx;
    let (active_talents, mut talent_progress, mut pending_cast_events) = talent_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Sleep {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = sleep_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &targets_query,
        &mut commands,
        &visual_assets,
        &mut meshes,
        sfx,
        game_config,
        &talent_params,
        &mut talent_progress,
        local_origin.0,
        &mut pending_cast_events,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core sleep casting logic.
#[allow(clippy::too_many_arguments)]
fn sleep_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    targets_query: &Query<(Entity, &Transform, &Health, &Team), (Without<Corpse>, Without<Wizard>)>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &SleepTalentParams,
    talent_progress: &mut Option<ResMut<BattleTalentProgress>>,
    local_origin: Vec3,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> bool {
    // Check for release event
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = local_origin;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let effective_radius = constants::CIRCLE_RADIUS * scale * talent_params.radius_mult;
    let effective_mana_cost = constants::MANA_COST * talent_params.mana_mult;
    let effective_cast_time = primed_spell.cast_time * talent_params.cast_time_mult;
    let max_center_distance = (max_ground_radius - effective_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.sleep_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    effective_mana_cost,
                    cursor_world_pos,
                    effective_radius,
                    caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );
            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(effective_mana_cost) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                        && let Ok(indicator) = indicator_query.get(indicator_entity)
                    {
                        audio::play_sfx_synced(
                            commands,
                            pending,
                            SpellSoundId::SleepCast,
                            indicator.position,
                            game_config,
                            sfx,
                        );
                        vfx::systems::spawn_aura_bubble_synced(
                            commands,
                            assets,
                            pending,
                            assets.sleep_aura_sphere.clone(),
                            crate::networking::snapshot::AuraBubbleVariant::Sleep,
                            indicator.position,
                            effective_radius,
                            2.5,
                        );
                        let hit_count = apply_sleep(
                            commands,
                            indicator.position,
                            effective_radius,
                            primed_spell.empowerment,
                            targets_query,
                            talent_params,
                        );
                        // Track talent progress
                        if hit_count > 0
                            && let Some(progress) = talent_progress
                        {
                            progress.increment(Spell::Sleep, hit_count);
                        }
                    }
                    completed = true;
                }
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}
