use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{EntangleGroundEffect, EntangleRooted, EntangleTalentParams, EntangleVine};
use super::constants;
use crate::config::GameConfig;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{
    Corpse, Health, RootedModifier, SlowMovementModifier, Team, TemporaryHitPoints,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    self, SpellCircleIndicator, clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;

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
    visual_assets: Res<SpellVisualAssets>,
    mesh_and_materials: (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    targets_query: Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    mut defender_hit_msg: MessageWriter<EntangleHitDefenderMessage>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talent_resources: (Option<Res<ActiveTalents>>, Option<ResMut<BattleTalentProgress>>),
) {
    let (active_talents, mut talent_progress) = talent_resources;
    let (mut meshes, mut materials) = mesh_and_materials;
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Entangle {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let effective_radius = constants::CIRCLE_RADIUS * primed_spell.empowerment * talent_params.radius_mult;
    let effective_mana_cost = constants::MANA_COST * talent_params.mana_mult;
    let effective_cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range(
        input.cursor_pos,
        wizard.spell_range,
        effective_radius,
    );

    // Handle release — clean up indicator
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(effective_mana_cost)
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.entangle_indicator.clone(),
                    clamped_cursor,
                    effective_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update indicator position
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = clamped_cursor;
            }

            if casting_state.is_complete(effective_cast_time) {
                if mana.consume(effective_mana_cost) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let cast_pos = indicator.position;
                            let root_duration =
                                constants::ROOT_DURATION * primed_spell.empowerment * talent_params.duration_mult;
                            audio::play_sfx(
                                &mut commands,
                                &sfx.entangle_cast,
                                cast_pos,
                                &game_config,
                                &sfx,
                            );
                            let hit_count = apply_entangle(
                                &mut commands,
                                &visual_assets,
                                &mut materials,
                                cast_pos,
                                effective_radius,
                                root_duration,
                                &targets_query,
                                &mut defender_hit_msg,
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
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }
}

/// Ticks entangle ground effect timer and handles Overgrowth expansion.
pub fn tick_entangle_ground_effect(
    time: Res<Time>,
    mut effects: Query<&mut EntangleGroundEffect>,
) {
    let delta = time.delta_secs();
    for mut effect in &mut effects {
        effect.time_remaining -= delta;

        // Overgrowth: expand zone over its lifetime
        if effect.talent_params.overgrowth {
            let progress = (effect.time_remaining / effect.duration).max(0.0);
            let elapsed_fraction = 1.0 - progress;
            let growth = effect.base_radius * constants::OVERGROWTH_GROWTH_FRACTION * elapsed_fraction;
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
) {
    for (entity, effect) in &effects {
        if effect.time_remaining <= 0.0 {
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
        &mut EntangleRooted,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
) {
    let delta = time.delta_secs();
    for (entity, mut entangle, mut health, mut temp_hp) in &mut rooted_units {
        if !entangle.talent_params.thorny_vines {
            continue;
        }
        entangle.thorny_tick_timer += delta;
        if entangle.thorny_tick_timer >= constants::THORNY_VINES_TICK_INTERVAL {
            entangle.thorny_tick_timer -= constants::THORNY_VINES_TICK_INTERVAL;
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
        (Entity, &EntangleRooted, &mut Health, Option<&mut TemporaryHitPoints>),
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

        commands.entity(entity).remove::<EntangleRooted>();
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
        let regen = constants::NOURISHING_ROOTS_MANA_PER_SEC * enemy_count as f32 * time.delta_secs();
        mana.regenerate(regen);
    }
}

/// Applies entangle root/sanctuary to a single unit based on talent params.
/// Returns true if the unit is an enemy (for hit counting).
fn apply_entangle_to_unit(
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
        commands
            .entity(entity)
            .insert(TemporaryHitPoints::new(
                constants::SANCTUARY_TEMP_HP,
                duration,
            ));
    } else {
        commands.entity(entity).insert((
            RootedModifier::new(duration),
            EntangleRooted {
                total_root_duration: duration,
                is_defender,
                thorny_tick_timer: 0.0,
                talent_params: *talent_params,
            },
        ));
    }

    if is_defender {
        defender_hit_msg.write(EntangleHitDefenderMessage);
    }

    !is_defender
}

/// Applies root to ALL units in radius (magic is indiscriminate).
/// Returns the number of enemies hit (for talent progress tracking).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_entangle(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    circle_pos: Vec3,
    radius: f32,
    root_duration: f32,
    targets: &Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    defender_hit_msg: &mut MessageWriter<EntangleHitDefenderMessage>,
    talent_params: &EntangleTalentParams,
) -> u32 {
    let mut hit_count = 0u32;

    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            if apply_entangle_to_unit(commands, entity, team, root_duration, talent_params, defender_hit_msg) {
                hit_count += 1;
            }
        }
    }

    // Spawn invisible ground effect entity (tracks zone for overgrowth/fade)
    commands.spawn((
        Transform::from_translation(Vec3::new(
            circle_pos.x,
            constants::CIRCLE_Y_POSITION,
            circle_pos.z,
        )),
        Visibility::Hidden,
        EntangleGroundEffect::new(root_duration, circle_pos, radius, *talent_params),
        NetworkedSpellEffect {
            kind: SpellEffectKind::EntangleGround,
        },
        OnGameplayScreen,
    ));

    // Spawn vine toruses rising from the ground
    spawn_vine_toruses(commands, assets, materials, circle_pos, radius, root_duration);

    hit_count
}

/// Spawns random flat vine rings within the entangle circle.
fn spawn_vine_toruses(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    center: Vec3,
    radius: f32,
    duration: f32,
) {
    for _ in 0..constants::VINE_COUNT {
        // Random position within circle (uniform distribution via rejection-free polar)
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        let dist = radius * rand::random::<f32>().sqrt() * 0.85; // 0.85 keeps vines slightly inward
        let x = center.x + angle.cos() * dist;
        let z = center.z + angle.sin() * dist;

        // Random scale
        let scale = constants::VINE_MIN_SCALE
            + rand::random::<f32>() * (constants::VINE_MAX_SCALE - constants::VINE_MIN_SCALE);

        // Random orientation — tilt the ring so it looks like a vine arching out of the ground
        // Annulus lies in XZ plane by default, so we tilt it partially upright
        let yaw = rand::random::<f32>() * std::f32::consts::TAU;
        let tilt = 0.4 + rand::random::<f32>() * 0.8; // 0.4..1.2 radians tilt from horizontal
        let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(tilt);

        // Position so at most 75% of the ring is above ground (y=0).
        // The ring's visible height above ground depends on tilt and scale.
        // We set final_y so the center is near or below ground level,
        // leaving only the top arc poking through.
        let max_above = constants::VINE_MAX_ABOVE_GROUND * (0.3 + rand::random::<f32>() * 0.7);
        // Center of ring sits below ground so only the top arch is visible
        let final_y = max_above - scale * tilt.sin() * 0.5;

        // Each vine gets its own material instance for independent alpha fading
        let base_mat = materials
            .get(&assets.entangle_vine)
            .cloned()
            .unwrap_or_default();
        let vine_material = materials.add(base_mat);

        commands.spawn((
            Mesh3d(assets.entangle_vine_ring.clone()),
            MeshMaterial3d(vine_material),
            Transform::from_translation(Vec3::new(x, constants::VINE_START_OFFSET, z))
                .with_rotation(rotation)
                .with_scale(Vec3::splat(scale)),
            EntangleVine {
                final_y,
                rise_elapsed: 0.0,
                rise_duration: constants::VINE_RISE_DURATION
                    * (0.7 + rand::random::<f32>() * 0.6), // Stagger rise timing
                duration,
                time_remaining: duration,
            },
            OnGameplayScreen,
        ));
    }
}

/// Animates vine toruses: rise from ground, then fade out at end of life.
pub fn animate_entangle_vines(
    time: Res<Time>,
    mut commands: Commands,
    mut vines: Query<(
        Entity,
        &mut EntangleVine,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();
    for (entity, mut vine, mut transform, material_handle) in &mut vines {
        vine.time_remaining -= delta;

        // Rise animation
        if vine.rise_elapsed < vine.rise_duration {
            vine.rise_elapsed += delta;
            let progress = (vine.rise_elapsed / vine.rise_duration).clamp(0.0, 1.0);
            // Ease-out: fast start, slow finish
            let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
            transform.translation.y =
                constants::VINE_START_OFFSET + (vine.final_y - constants::VINE_START_OFFSET) * eased;
        }

        // Fade out in the last 25% of lifetime
        let life_fraction = (vine.time_remaining / vine.duration).clamp(0.0, 1.0);
        if life_fraction < 0.25 {
            let fade = life_fraction / 0.25; // 1.0 → 0.0
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = Color::srgba(0.05, 0.3, 0.05, 0.75 * fade);
            }
        }

        // Despawn when expired
        if vine.time_remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns animated vine ring particles from active entangle zones.
pub fn emit_animated_vine_rings(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut effects: Query<&mut EntangleGroundEffect>,
) {
    let delta = time.delta_secs();
    for mut effect in &mut effects {
        if effect.time_remaining <= 0.0 {
            continue;
        }

        effect.animated_vine_timer += delta;
        if effect.animated_vine_timer < utils::RING_SPAWN_INTERVAL {
            continue;
        }
        effect.animated_vine_timer -= utils::RING_SPAWN_INTERVAL;

        utils::spawn_ring_particle(
            &mut commands,
            visual_assets.entangle_vine_ring.clone(),
            visual_assets.entangle_vine.clone(),
            effect.center,
            effect.current_radius,
        );
    }
}
