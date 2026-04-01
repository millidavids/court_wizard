use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{
    CleansingPlumeZone, FieldMedicConverted, FieldMedicOriginalType, FontOfLifePending,
    FontOfLifeZone, HealingPlumeTalentParams, HealingPlumeZone, HealingRainZone, OverflowZone,
    TriagePulseZone,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{
    Corpse, Health, MarkedForDeathModifier, RootedModifier, SlowMovementModifier, Team,
    TemporaryHitPoints,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, build_wizard_input, clamp_cursor_to_spell_range, cleanup_spell_caster,
    get_cursor_world_position, handle_spell_release, spawn_circle_indicator,
    update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
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
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
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
    let input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);

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

    let clamped_cursor = clamp_cursor_to_spell_range(input.cursor_pos, wizard.spell_range, radius);

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
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
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
                    &mut materials,
                    indicator.position,
                    radius,
                    primed_spell.empowerment,
                    &talent_params,
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

                audio::play_sfx(
                    &mut commands,
                    &sfx.healing_plume_cast,
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
            vfx::systems::spawn_floating_motes(
                &mut commands,
                &visual_assets,
                &visual_assets.healing_mote,
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

/// Tier 3: Font of Life — detects deaths inside the zone and schedules resurrection.
pub fn font_of_life_detect_deaths(
    mut commands: Commands,
    mut zones: Query<(Entity, &HealingPlumeZone, &mut FontOfLifeZone)>,
    new_corpses: Query<(Entity, &Transform), (With<Corpse>, Without<FontOfLifePending>)>,
) {
    for (_zone_entity, zone, mut font) in &mut zones {
        for (corpse_entity, transform) in &new_corpses {
            // Skip already-resurrected entities
            if font.resurrected.contains(&corpse_entity) {
                continue;
            }

            let distance = xz_distance(zone.origin, transform.translation);

            if distance <= zone.radius {
                font.resurrected.insert(corpse_entity);
                commands.entity(corpse_entity).insert(FontOfLifePending {
                    time_remaining: constants::FONT_OF_LIFE_RESURRECT_DELAY,
                });
            }
        }
    }
}

/// Tier 3: Font of Life — processes pending resurrections.
#[allow(clippy::too_many_arguments)]
pub fn font_of_life_resurrect(
    time: Res<Time>,
    mut commands: Commands,
    mut pending: Query<(Entity, &Transform, &mut FontOfLifePending), With<Corpse>>,
    infantry_assets: Res<crate::game::units::infantry::resources::InfantryAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, transform, mut pending_res) in &mut pending {
        pending_res.time_remaining -= delta;

        if pending_res.time_remaining <= 0.0 {
            let max_hp = crate::game::constants::UNIT_HEALTH;
            let resurrect_hp = max_hp * constants::FONT_OF_LIFE_RESURRECT_HP_PERCENT;

            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                entity,
                transform.translation,
                Team::Defenders,
                resurrect_hp,
                constants::FONT_OF_LIFE_RESURRECT_SPEED,
                Color::srgba(0.3, 0.9, 0.3, 1.0), // Green tint for resurrected
                infantry_assets.sprite_texture.clone(),
                infantry_assets.sprite_mesh.clone(),
                &mut materials,
            );

            commands.entity(entity).remove::<FontOfLifePending>();
        }
    }
}

/// Tier 3: Healing Rain — moves the zone toward the wizard's cursor each frame.
pub fn move_healing_rain_zones(
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut zones: Query<(&mut HealingPlumeZone, &mut Transform), With<HealingRainZone>>,
) {
    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        return;
    };
    let target = Vec3::new(cursor_pos.x, 0.0, cursor_pos.z);
    let delta = time.delta_secs();

    for (mut zone, mut transform) in &mut zones {
        let direction = target - zone.origin;
        let dist = direction.length();
        if dist < 1.0 {
            continue;
        }

        let move_amount = (constants::HEALING_RAIN_MOVE_SPEED * delta).min(dist);
        let offset = direction.normalize() * move_amount;
        zone.origin += offset;
        transform.translation.x = zone.origin.x;
        transform.translation.z = zone.origin.z;
    }
}

/// Tier 3: Field Medic — attempts to convert the nearest defender in the zone to a healer.
fn try_convert_field_medic(
    commands: &mut Commands,
    zone_entity: Entity,
    position: Vec3,
    radius: f32,
    defenders_query: &Query<
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
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut best: Option<(
        Entity,
        f32,
        FieldMedicOriginalType,
        Handle<StandardMaterial>,
    )> = None;

    for (entity, transform, team, material_handle, is_infantry, is_archer) in defenders_query.iter()
    {
        if *team != Team::Defenders {
            continue;
        }

        let original_type = if is_archer {
            FieldMedicOriginalType::Archer
        } else if is_infantry {
            FieldMedicOriginalType::Infantry
        } else {
            continue;
        };

        let distance = xz_distance(position, transform.translation);

        if distance <= radius && best.as_ref().is_none_or(|(_, d, _, _)| distance < *d) {
            best = Some((entity, distance, original_type, material_handle.0.clone()));
        }
    }

    if let Some((entity, _, original_type, original_material)) = best {
        // Remove original unit marker immediately
        match original_type {
            FieldMedicOriginalType::Infantry => {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::infantry::components::Infantry>();
            }
            FieldMedicOriginalType::Archer => {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::archer::components::Archer>();
            }
        }

        // Create green-tinted material for the converted healer
        let (r, g, b, a) = constants::FIELD_MEDIC_COLOR;
        let green_material = materials.add(StandardMaterial {
            base_color: Color::srgba(r, g, b, a),
            ..Default::default()
        });

        commands.entity(entity).insert((
            crate::game::units::healer::components::Healer,
            crate::game::units::healer::components::HealerAttackTimer::new(),
            MeshMaterial3d(green_material),
            FieldMedicConverted {
                zone_entity,
                original_type,
                original_material,
            },
        ));
    }
}

/// Tier 3: Field Medic — reverts converted healers when the zone expires.
pub fn field_medic_cleanup(
    mut commands: Commands,
    zones: Query<Entity, With<HealingPlumeZone>>,
    converted_units: Query<(Entity, &FieldMedicConverted)>,
) {
    for (entity, converted) in &converted_units {
        // If the zone no longer exists, revert the conversion
        if zones.get(converted.zone_entity).is_err() {
            commands
                .entity(entity)
                .remove::<crate::game::units::healer::components::Healer>()
                .remove::<crate::game::units::healer::components::HealerAttackTimer>()
                .remove::<FieldMedicConverted>()
                .insert(MeshMaterial3d(converted.original_material.clone()));

            // Restore original unit type marker
            match converted.original_type {
                FieldMedicOriginalType::Infantry => {
                    commands
                        .entity(entity)
                        .insert(crate::game::units::infantry::components::Infantry);
                }
                FieldMedicOriginalType::Archer => {
                    commands
                        .entity(entity)
                        .insert(crate::game::units::archer::components::Archer);
                }
            }
        }
    }
}

/// Fades healing plume zone visual over the last few seconds.
pub fn fade_healing_plume_zone(
    zones: Query<(&HealingPlumeZone, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (zone, material_handle) in &zones {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        material.base_color = Color::srgba(0.1, 0.7, 0.2, 0.4 * fade);
    }
}

/// Despawns expired healing plume zones.
pub fn cleanup_healing_plume_zone(
    mut commands: Commands,
    zones: Query<(Entity, &HealingPlumeZone)>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_healing_plume_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &HealingPlumeTalentParams,
) -> Entity {
    let duration = constants::ZONE_DURATION * empowerment * talent_params.duration_mult;
    let mut heal = constants::HEAL_PER_TICK * empowerment * talent_params.heal_mult;
    if talent_params.healing_rain {
        heal *= constants::HEALING_RAIN_HEAL_MULT;
    }

    let base_mat = materials
        .get(&assets.healing_plume_zone)
        .cloned()
        .unwrap_or_default();
    let instance_material = materials.add(base_mat);

    let zone_entity = commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(instance_material),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
            HealingPlumeZone::new(
                Vec3::new(position.x, 0.0, position.z),
                radius,
                heal,
                constants::TICK_INTERVAL,
                duration,
            ),
            NetworkedSpellEffect {
                kind: SpellEffectKind::HealingPlumeZone,
            },
            OnGameplayScreen,
        ))
        .id();

    // Add talent-specific zone components
    if talent_params.cleansing_plume {
        commands
            .entity(zone_entity)
            .insert(CleansingPlumeZone::new());
    }
    if talent_params.overflow {
        commands.entity(zone_entity).insert(OverflowZone);
    }
    if talent_params.triage_pulse {
        commands.entity(zone_entity).insert(TriagePulseZone);
    }
    if talent_params.font_of_life {
        commands.entity(zone_entity).insert(FontOfLifeZone::new());
    }
    if talent_params.healing_rain {
        commands.entity(zone_entity).insert(HealingRainZone);
    }

    zone_entity
}
