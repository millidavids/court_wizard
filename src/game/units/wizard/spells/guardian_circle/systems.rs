use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::GuardianCircleShielded;
use super::constants;
use crate::config::GameConfig;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
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
use bevy::prelude::*;

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
    mut targets_query: Query<(Entity, &Transform, &Team), Without<Wizard>>,
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

                audio::play_sfx(
                    &mut commands,
                    &sfx.guardian_circle_cast,
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

/// Helper function to apply Guardian Circle buff to all units in radius.
///
/// Grants temporary HP to units with talent modifications applied.
/// Also inserts GuardianCircleShielded marker for Tier 2/3 talent effects.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, &Team), Without<Wizard>>,
    attacker_hit_msg: &mut MessageWriter<GuardianCircleHitAttackerMessage>,
    talent_progress: &mut Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    active_talents: Option<&ActiveTalents>,
) {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::GuardianCircle, 2));

    // Scale values by empowerment
    let scale = empowerment;
    let mut scaled_temp_hp = temp_hp_amount * scale;
    let mut scaled_duration = duration * scale;

    // Tier 1 modifications
    match t1 {
        Some(0) => scaled_temp_hp *= constants::REINFORCED_WARDS_MULT, // +40% temp HP
        Some(1) => scaled_duration *= constants::ENDURING_PROTECTION_MULT, // +60% duration
        Some(2) => scaled_temp_hp *= constants::EXPANSIVE_AEGIS_HP_MULT, // -15% temp HP
        _ => {}
    }

    // Build the GuardianCircleShielded component based on T2/T3 selections
    let has_talent_effects = t2.is_some() || t3.is_some();
    let shielded = if has_talent_effects {
        let mut s = GuardianCircleShielded::default();

        // Tier 2
        match t2 {
            Some(0) => {
                // Retaliating Wards
                s.retaliating_damage = constants::RETALIATING_WARDS_DAMAGE * scale;
                s.retaliating_radius = constants::RETALIATING_WARDS_RADIUS;
            }
            Some(1) => {
                // Fortified Resolve
                s.fortified_damage_bonus = constants::FORTIFIED_RESOLVE_DAMAGE_MULT;
            }
            // Rapid Deployment is handled in casting, not here
            _ => {}
        }

        // Tier 3
        match t3 {
            Some(0) => {
                // Sanctuary
                s.sanctuary_reduction = constants::SANCTUARY_DAMAGE_REDUCTION;
            }
            Some(1) => {
                // Martyrdom — store the granted temp HP as explosion damage
                s.martyrdom_damage = scaled_temp_hp;
                s.martyrdom_radius = constants::MARTYRDOM_DAMAGE_RADIUS;
            }
            Some(2) => {
                // Chain Ward
                s.chain_ward_hops = constants::CHAIN_WARD_MAX_HOPS;
                s.chain_ward_amount = scaled_temp_hp;
                s.chain_ward_duration = scaled_duration;
            }
            _ => {}
        }

        Some(s)
    } else {
        None
    };

    let mut buffed_count = 0u32;
    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(scaled_temp_hp, scaled_duration));

            // Insert talent marker if any T2/T3 talents are active
            if let Some(ref s) = shielded {
                commands.entity(entity).insert(s.clone());
            }

            // Protective Instincts: Guardian Circle hit an attacker or undead
            if *team == Team::Attackers || *team == Team::Undead {
                attacker_hit_msg.write(GuardianCircleHitAttackerMessage);
            }

            buffed_count += 1;
        }
    }

    if buffed_count > 0
        && let Some(progress) = talent_progress.as_deref_mut()
    {
        progress.increment(Spell::GuardianCircle, buffed_count);
    }
}

/// Cleanup system: remove GuardianCircleShielded when temp HP expires or is removed.
pub fn cleanup_guardian_circle_shielded(
    mut commands: Commands,
    query: Query<Entity, (With<GuardianCircleShielded>, Without<TemporaryHitPoints>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<GuardianCircleShielded>();
    }
}

/// Deals AoE force damage to enemies within radius of a position.
fn deal_aoe_force_damage(
    commands: &mut Commands,
    origin: Vec3,
    radius: f32,
    damage: f32,
    source_team: &Team,
    targets: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<GuardianCircleShielded>),
    >,
) {
    for (entity, transform, team, mut health, temp_hp) in targets.iter_mut() {
        if *team == *source_team {
            continue;
        }
        if transform.translation.distance(origin) <= radius {
            apply_spell_damage(
                commands,
                entity,
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                damage,
                DamageType::Force,
                false,
            );
        }
    }
}

/// Tier 2, Choice 0: Retaliating Wards.
///
/// When a unit's temp HP is fully broken (amount reaches 0 but component still exists),
/// deal AoE force damage to nearby enemies. Fires once then removes the marker.
pub fn retaliating_wards_check(
    mut commands: Commands,
    shielded_query: Query<(
        Entity,
        &GuardianCircleShielded,
        &TemporaryHitPoints,
        &Transform,
        &Team,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<GuardianCircleShielded>),
    >,
) {
    for (entity, shielded, temp_hp, transform, team) in &shielded_query {
        if shielded.retaliating_damage <= 0.0 || temp_hp.amount > 0.0 {
            continue;
        }

        deal_aoe_force_damage(
            &mut commands,
            transform.translation,
            shielded.retaliating_radius,
            shielded.retaliating_damage,
            team,
            &mut targets,
        );

        // One-shot: remove marker so retaliation doesn't fire again
        commands.entity(entity).remove::<GuardianCircleShielded>();
    }
}

/// Tier 3, Choice 1: Martyrdom.
///
/// When a shielded unit dies, the stored shield amount explodes as AoE damage.
/// Damage is the full temp HP granted at cast time (not what remained at death).
/// Fires once then removes the marker from the corpse.
pub fn martyrdom_on_death(
    mut commands: Commands,
    dead_query: Query<(Entity, &GuardianCircleShielded, &Transform, &Team), With<Corpse>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<GuardianCircleShielded>),
    >,
) {
    for (corpse_entity, shielded, transform, team) in &dead_query {
        if shielded.martyrdom_damage <= 0.0 {
            continue;
        }

        deal_aoe_force_damage(
            &mut commands,
            transform.translation,
            shielded.martyrdom_radius,
            shielded.martyrdom_damage,
            team,
            &mut targets,
        );

        // One-shot: remove marker so martyrdom doesn't fire again
        commands
            .entity(corpse_entity)
            .remove::<GuardianCircleShielded>();
    }
}

/// Tier 3, Choice 2: Chain Ward.
///
/// When a shielded unit dies, its temp HP jumps to the nearest unshielded ally.
/// Fires once then removes the marker from the corpse.
pub fn chain_ward_on_death(
    mut commands: Commands,
    dead_query: Query<(Entity, &GuardianCircleShielded, &Transform, &Team), With<Corpse>>,
    alive_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<GuardianCircleShielded>,
            Without<Wizard>,
        ),
    >,
) {
    for (corpse_entity, shielded, transform, shielded_team) in &dead_query {
        if shielded.chain_ward_hops == 0 || shielded.chain_ward_amount <= 0.0 {
            continue;
        }

        // Find the nearest allied unit without a shield
        let mut nearest: Option<(Entity, f32)> = None;
        for (candidate, candidate_transform, candidate_team) in &alive_query {
            if *candidate_team != *shielded_team {
                continue;
            }
            let dist = candidate_transform
                .translation
                .distance(transform.translation);
            if nearest.is_none_or(|(_, d)| dist < d) {
                nearest = Some((candidate, dist));
            }
        }

        if let Some((target_entity, _)) = nearest {
            commands
                .entity(target_entity)
                .insert(TemporaryHitPoints::new(
                    shielded.chain_ward_amount,
                    shielded.chain_ward_duration,
                ));

            // Clone and pass along with one fewer hop
            let mut chained = shielded.clone();
            chained.chain_ward_hops -= 1;
            commands.entity(target_entity).insert(chained);
        }

        // One-shot: remove marker so chain ward doesn't fire again
        commands
            .entity(corpse_entity)
            .remove::<GuardianCircleShielded>();
    }
}
