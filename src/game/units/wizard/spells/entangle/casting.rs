//! Entangle casting and zone setup.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard,
};
use super::components::{EntangleGroundEffect, EntangleRooted, EntangleTalentParams, ThornyVines};
use super::constants;
use super::vines::apply_entangle;
use crate::config::GameConfig;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{
    Corpse, Health, RootedModifier, SlowMovementModifier, Team, TemporaryHitPoints,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::networking::snapshot::SpellSoundId;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin, cleanup_spell_caster, handle_spell_release,
    try_start_cast_with_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
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
    targets_query: Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
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

/// Ticks entangle ground effect timer and handles Overgrowth expansion.
pub fn tick_entangle_ground_effect(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut effects: Query<&mut EntangleGroundEffect>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let delta = time.delta_secs();
    let mote_interval = vfx::constants::MOTE_SPAWN_INTERVAL;
    let mote_count = vfx::constants::MOTE_COUNT_PER_SPAWN;

    for mut effect in &mut effects {
        let prev_remaining = effect.time_remaining;
        effect.time_remaining -= delta;

        let prev_elapsed = effect.duration - prev_remaining;
        let curr_elapsed = effect.duration - effect.time_remaining;
        if effect.time_remaining > 0.0
            && (curr_elapsed / mote_interval).floor() != (prev_elapsed / mote_interval).floor()
        {
            vfx::systems::spawn_floating_motes_synced(
                &mut commands,
                &visual_assets,
                &mut pending_cast_events,
                &visual_assets.nature_mote,
                crate::networking::snapshot::MoteMaterial::Nature,
                effect.center,
                effect.current_radius,
                mote_count,
                time.elapsed_secs(),
            );
        }

        // Overgrowth: expand zone over its lifetime
        if effect.talent_params.overgrowth {
            let progress = (effect.time_remaining / effect.duration).max(0.0);
            let elapsed_fraction = 1.0 - progress;
            let growth =
                effect.base_radius * constants::OVERGROWTH_GROWTH_FRACTION * elapsed_fraction;
            effect.current_radius = effect.base_radius + growth;
        }
    }
}

/// Overgrowth: periodically root new units entering the expanding zone.
pub fn overgrowth_root_new_units(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<&mut EntangleGroundEffect>,
    targets: Query<
        (Entity, &Transform, &Team, Option<&RootedModifier>),
        (Without<Wizard>, Without<Corpse>),
    >,
    mut defender_hit_msg: MessageWriter<EntangleHitDefenderMessage>,
) {
    let delta = time.delta_secs();
    for mut effect in &mut effects {
        if !effect.talent_params.overgrowth {
            continue;
        }
        effect.overgrowth_check_timer += delta;
        if effect.overgrowth_check_timer < constants::OVERGROWTH_CHECK_INTERVAL {
            continue;
        }
        effect.overgrowth_check_timer -= constants::OVERGROWTH_CHECK_INTERVAL;

        let remaining_duration = effect.time_remaining;
        if remaining_duration <= 0.0 {
            continue;
        }

        let talent_params = effect.talent_params;
        let center = effect.center;
        let radius = effect.current_radius;

        for (entity, transform, team, rooted) in &targets {
            if rooted.is_some() {
                continue;
            }
            let distance = transform.translation.distance(center);
            if distance <= radius {
                apply_entangle_to_unit(
                    &mut commands,
                    entity,
                    team,
                    remaining_duration,
                    &talent_params,
                    &mut defender_hit_msg,
                );
            }
        }
    }
}

/// Despawns expired entangle ground effects.
pub fn cleanup_entangle_ground_effect(
    mut commands: Commands,
    effects: Query<(Entity, &EntangleGroundEffect)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, effect) in &effects {
        if effect.time_remaining <= 0.0 {
            // Notify pathfinding that this zone is removed
            let buffered_radius = effect.current_radius + OBSTACLE_BUFFER;
            let origin_2d = Vec2::new(effect.center.x, effect.center.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}

/// Thorny Vines: deals periodic damage to rooted enemies (not defenders).
pub fn thorny_vines_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut rooted_units: Query<(
        Entity,
        &mut ThornyVines,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut thorny, mut health, mut temp_hp) in &mut rooted_units {
        thorny.tick_timer += delta;
        if thorny.tick_timer >= constants::THORNY_VINES_TICK_INTERVAL {
            thorny.tick_timer -= constants::THORNY_VINES_TICK_INTERVAL;
            let damage = constants::THORNY_VINES_DPS * constants::THORNY_VINES_TICK_INTERVAL;
            crate::game::units::components::apply_damage_to_unit(
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
            );
            if health.current <= 0.0 {
                commands.entity(entity).insert(Corpse);
            }
        }
    }
}

/// Handles effects when EntangleRooted units lose their RootedModifier (root expired).
/// Applies Clinging Roots slow and Stranglehold burst damage.
pub fn handle_entangle_root_expire(
    mut commands: Commands,
    mut rooted_units: Query<
        (
            Entity,
            &EntangleRooted,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<RootedModifier>,
    >,
) {
    for (entity, entangle, mut health, mut temp_hp) in &mut rooted_units {
        // Clinging Roots: slow enemies after root expires
        if entangle.talent_params.clinging_roots && !entangle.is_defender {
            commands.entity(entity).insert(SlowMovementModifier::new(
                constants::CLINGING_ROOTS_SLOW,
                constants::CLINGING_ROOTS_SLOW_DURATION,
            ));
        }

        // Stranglehold: burst damage if rooted long enough
        if entangle.talent_params.stranglehold
            && !entangle.is_defender
            && entangle.total_root_duration >= constants::STRANGLEHOLD_THRESHOLD
        {
            crate::game::units::components::apply_damage_to_unit(
                &mut health,
                temp_hp.as_deref_mut(),
                constants::STRANGLEHOLD_DAMAGE,
            );
            if health.current <= 0.0 {
                // Stranglehold kills don't leave corpses — despawn entirely
                commands.entity(entity).try_despawn();
                continue;
            }
        }

        commands
            .entity(entity)
            .remove::<(EntangleRooted, ThornyVines)>();
    }
}

/// Nourishing Roots: regenerates wizard mana based on number of rooted enemies.
pub fn nourishing_roots_mana_regen(
    time: Res<Time>,
    mut wizard_query: Query<&mut Mana, With<LocalWizard>>,
    rooted_units: Query<&EntangleRooted>,
) {
    let Ok(mut mana) = wizard_query.single_mut() else {
        return;
    };

    let enemy_count = rooted_units
        .iter()
        .filter(|e| e.talent_params.nourishing_roots && !e.is_defender)
        .count();

    if enemy_count > 0 {
        let regen =
            constants::NOURISHING_ROOTS_MANA_PER_SEC * enemy_count as f32 * time.delta_secs();
        mana.regenerate(regen);
    }
}

/// Applies entangle root/sanctuary to a single unit based on talent params.
/// Returns true if the unit is an enemy (for hit counting).
pub(super) fn apply_entangle_to_unit(
    commands: &mut Commands,
    entity: Entity,
    team: &Team,
    duration: f32,
    talent_params: &EntangleTalentParams,
    defender_hit_msg: &mut MessageWriter<EntangleHitDefenderMessage>,
) -> bool {
    let is_defender = *team == Team::Defenders;

    // Nature's Sanctuary: defenders get temp HP instead of root
    if talent_params.sanctuary && is_defender {
        commands.entity(entity).insert(TemporaryHitPoints::new(
            constants::SANCTUARY_TEMP_HP,
            duration,
        ));
    } else {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            RootedModifier::new(duration),
            EntangleRooted {
                total_root_duration: duration,
                is_defender,
                talent_params: *talent_params,
            },
        ));
        // Thorny Vines: only apply to non-defenders
        if talent_params.thorny_vines && !is_defender {
            entity_commands.insert(ThornyVines { tick_timer: 0.0 });
        }
    }

    if is_defender {
        defender_hit_msg.write(EntangleHitDefenderMessage);
    }

    !is_defender
}
