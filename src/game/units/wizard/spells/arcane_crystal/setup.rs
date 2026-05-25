//! Arcane crystal helpers, casting, and spawn.

use super::auto::crystal_aoe_burst;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::disintegrate::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam,
};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range, cleanup_spell_caster, handle_spell_release, spawn_circle_indicator,
    update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;

// ===== Talent Param Computation =====

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> ArcaneCrystalTalentParams {
    let mut params = ArcaneCrystalTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::ArcaneCrystal, 0) {
        Some(0) => params.damage_mult = REFINED_FACETS_DAMAGE_MULT,
        Some(1) => params.range_mult = WIDER_PRISM_RANGE_MULT,
        Some(2) => params.duration_mult = ENDURING_CRYSTAL_DURATION_MULT,
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::ArcaneCrystal, 1) {
        Some(0) => params.count_mult = OVERCHARGED_MATRIX_COUNT_MULT,
        Some(1) => params.resonance_cascade = true,
        Some(2) => params.spell_echo = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::ArcaneCrystal, 2) {
        Some(0) => params.crystal_network = true,
        Some(1) => params.prismatic_explosion = true,
        Some(2) => params.auto_crystal = true,
        _ => {}
    }

    params
}

/// Applies the count multiplier to a base count, rounding up.
pub(super) fn scaled_count(base: usize, count_mult: f32) -> usize {
    (base as f32 * count_mult).ceil() as usize
}

/// Returns 2 if Spell Echo triggers (30% chance), 1 otherwise.
pub(super) fn spell_echo_multiplier(rng: &mut impl Rng, spell_echo: bool) -> usize {
    if spell_echo && rng.random::<f32>() < SPELL_ECHO_CHANCE {
        return 2;
    }
    1
}

/// Increments resonance cascade counter if the component is present.
pub(super) fn increment_resonance(resonance: &mut Option<Mut<ResonanceCascade>>) {
    if let Some(res) = resonance {
        res.absorptions += 1;
    }
}

// ===== Frame Reset =====

/// Clears per-frame absorption flags on all crystals.
/// Guarded to avoid triggering Bevy change detection when already false.
pub(super) fn clear_absorption_flags(mut crystals: Query<&mut ArcaneCrystal>) {
    for mut crystal in &mut crystals {
        if crystal.just_absorbed {
            crystal.just_absorbed = false;
        }
    }
}

// ===== Helper Functions =====

/// Computes beam direction and length for a crystal beam.
/// The beam slopes from crystal height (origin.y) to Y=0 at max_range,
/// with its XZ direction aimed toward the target.
pub(super) fn crystal_beam_geometry(origin: Vec3, target: Vec3, max_range: f32) -> (Vec3, f32) {
    let origin_xz = Vec3::new(origin.x, 0.0, origin.z);
    let target_xz = Vec3::new(target.x, 0.0, target.z);
    let xz_dir = (target_xz - origin_xz).normalize_or(Vec3::X);
    let end_point = origin_xz + xz_dir * max_range; // Y=0 at range edge
    let direction = (end_point - origin).normalize();
    let length = origin.distance(end_point);
    (direction, length)
}

/// Finds random targets within range of a position.
/// Returns up to `count` random targets from any team (spells are indiscriminate).
pub(super) fn find_random_targets_in_range(
    rng: &mut impl Rng,
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) -> Vec<(Entity, Vec3)> {
    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, transform)| xz_distance(crystal_pos, transform.translation) <= range)
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();

    // Shuffle and take up to count
    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.random_range(0..=i);
        candidates.swap(i, j);
    }
    candidates.truncate(count);
    candidates
}

/// Finds random enemy targets (Attackers/Undead only) within range.
/// Used for magic missiles which should not target defenders.
pub(super) fn find_random_enemies_in_range(
    rng: &mut impl Rng,
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) -> Vec<(Entity, Vec3)> {
    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
        .filter(|(_, transform, _)| xz_distance(crystal_pos, transform.translation) <= range)
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.random_range(0..=i);
        candidates.swap(i, j);
    }
    candidates.truncate(count);
    candidates
}

// ===== Casting System =====

/// Local wizard Arcane Crystal casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_arcane_crystal_casting(
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
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    existing_crystals: Query<(Entity, &ArcaneCrystal)>,
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::ArcaneCrystal {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

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
                && mana.can_afford(MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.arcane_crystal_indicator.clone(),
                    pos,
                    CRYSTAL_RANGE * primed_spell.empowerment,
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
        arcane_crystal_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            local_origin.0,
            vfx::systems::SpellSchool::Arcane,
            time.elapsed_secs(),
        );
        let talent_params = compute_talent_params(active_talents.as_deref());

        if talent_params.auto_crystal {
            // Auto-Crystal (turret): only 1 non-permanent crystal allowed per level.
            // Count crystals placed this level (non-permanent ones).
            let placed_this_level = existing_crystals
                .iter()
                .filter(|(_, c)| !c.permanent)
                .count();
            if placed_this_level > 0 {
                // Already placed one this level — block
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
                // Refund mana since we blocked the cast
                mana.current = (mana.current + MANA_COST).min(mana.max);
                return;
            }
        } else if !talent_params.crystal_network {
            // Default: despawn existing crystals (non-permanent)
            for (crystal_entity, _) in &existing_crystals {
                commands.entity(crystal_entity).try_despawn();
            }
        } else {
            // Crystal Network: allow up to 3 crystals; despawn oldest if at limit
            let count = existing_crystals.iter().count();
            if count >= CRYSTAL_NETWORK_MAX_CRYSTALS
                && let Some((oldest, _)) = existing_crystals.iter().next()
            {
                commands.entity(oldest).try_despawn();
            }
        }

        // Spawn crystal using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                spawn_crystal(
                    &mut commands,
                    &visual_assets,
                    indicator.position,
                    primed_spell.empowerment,
                    &talent_params,
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.arcane_crystal_cast,
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

/// Core Arcane Crystal casting logic -- handles CastingState transitions and mana consumption.
///
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn arcane_crystal_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> bool {
    // Release is handled by the wrappers before calling this function
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
                if mana.consume(MANA_COST) {
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

/// Spawns the crystal entity with visual mesh and range indicator.
pub(crate) fn spawn_crystal(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    talent_params: &ArcaneCrystalTalentParams,
) {
    let range = CRYSTAL_RANGE * empowerment * talent_params.range_mult;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;
    let duration = CRYSTAL_DURATION * empowerment * talent_params.duration_mult;

    let is_turret = talent_params.auto_crystal;
    let mut crystal = ArcaneCrystal::new(
        Vec3::ZERO, // placeholder — set by spawn_crystal_entity
        range,
        duration,
        collision_radius,
        empowerment,
    );
    crystal.damage_mult = talent_params.damage_mult;
    crystal.count_mult = talent_params.count_mult;
    crystal.spell_echo = talent_params.spell_echo;

    if is_turret {
        crystal.permanent = true;
        crystal.duration = f32::MAX;
    }

    let mut entity_commands =
        spawn_crystal_entity(commands, assets, position, empowerment, crystal);

    if is_turret {
        entity_commands.insert(AutoCrystalTimer { timer: 0.0 });
    }
    // Resonance cascade applies to both turrets and normal crystals
    if talent_params.resonance_cascade {
        entity_commands.insert(ResonanceCascade { absorptions: 0 });
    }
    if !is_turret {
        if talent_params.prismatic_explosion {
            entity_commands.insert(PrismaticExplosion);
        }
        if talent_params.crystal_network {
            entity_commands.insert(CrystalNetwork);
        }
    }
}

/// Spawns a permanent crystal turret from saved data (loaded between levels).
pub(crate) fn spawn_permanent_crystal(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &crate::config::save_data::SavedCrystal,
    damage_mult: f32,
    count_mult: f32,
    resonance_cascade: bool,
) {
    let position = Vec3::new(saved.x, 0.0, saved.z);
    let empowerment = saved.empowerment;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;

    let mut crystal = ArcaneCrystal::new(
        Vec3::ZERO,
        saved.range,
        f32::MAX,
        collision_radius,
        empowerment,
    );
    crystal.permanent = true;
    crystal.damage_mult = damage_mult;
    crystal.count_mult = count_mult;

    let mut entity_commands =
        spawn_crystal_entity(commands, assets, position, empowerment, crystal);
    entity_commands.insert(AutoCrystalTimer { timer: 0.0 });
    if resonance_cascade {
        entity_commands.insert(ResonanceCascade { absorptions: 0 });
    }
}

/// Shared helper that spawns the crystal mesh entity and range indicator.
/// Returns the entity commands for the crystal so callers can insert additional components.
fn spawn_crystal_entity<'a>(
    commands: &'a mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    mut crystal: ArcaneCrystal,
) -> EntityCommands<'a> {
    let height = CRYSTAL_HEIGHT * empowerment;
    let crystal_pos = Vec3::new(position.x, height / 2.0, position.z);
    let sphere_radius = height / 3.0;
    let range = crystal.range;

    crystal.position = crystal_pos;

    let mut entity_commands = commands.spawn((
        crystal,
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.arcane_crystal.clone()),
        Transform::from_translation(crystal_pos).with_scale(Vec3::new(
            0.7 * sphere_radius,
            1.5 * sphere_radius,
            0.7 * sphere_radius,
        )),
        NetworkedSpellEffect {
            kind: SpellEffectKind::ArcaneCrystal,
        },
        OnGameplayScreen,
    ));

    let crystal_entity = entity_commands.id();

    // Spawn pink aura sphere as range indicator
    entity_commands.commands().spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(assets.crystal_aura_sphere.clone()),
        Transform::from_translation(Vec3::new(position.x, 0.0, position.z))
            .with_scale(Vec3::splat(range)),
        CrystalRangeIndicator { crystal_entity },
        OnGameplayScreen,
    ));

    entity_commands
}

// ===== Crystal Visuals & Lifetime =====

/// Updates crystal rotation, pulse animation, and lifetime.
pub(super) fn update_crystal_visuals(
    time: Res<Time>,
    mut crystals: Query<(&mut ArcaneCrystal, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (mut crystal, mut transform) in &mut crystals {
        if !crystal.permanent {
            crystal.time_alive += delta;
        }

        let sphere_radius = CRYSTAL_HEIGHT * crystal.empowerment / 3.0;

        // Rotation
        transform.rotate_y(ROTATION_SPEED * delta);

        // Pulse animation
        if crystal.pulse_timer > 0.0 {
            crystal.pulse_timer -= delta;
            let pulse_progress = crystal.pulse_timer / PULSE_DURATION;
            let scale_factor = 1.0 + (PULSE_SCALE - 1.0) * pulse_progress;
            transform.scale = Vec3::new(
                0.7 * sphere_radius * scale_factor,
                1.5 * sphere_radius * scale_factor,
                0.7 * sphere_radius * scale_factor,
            );
        } else {
            transform.scale = Vec3::new(
                0.7 * sphere_radius,
                1.5 * sphere_radius,
                0.7 * sphere_radius,
            );
        }
    }
}

/// Despawns expired crystals and their range indicators.
/// Triggers Prismatic Explosion if the talent is active.
#[allow(clippy::too_many_arguments)]
pub(super) fn cleanup_expired_crystals(
    mut commands: Commands,
    crystals: Query<(Entity, &ArcaneCrystal, Has<PrismaticExplosion>)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    visual_assets: Res<SpellVisualAssets>,
) {
    for (crystal_entity, crystal, has_prismatic) in &crystals {
        if crystal.permanent {
            continue;
        }
        if crystal.time_alive >= crystal.duration {
            // Prismatic Explosion: deal massive damage on expiry
            if has_prismatic {
                crystal_aoe_burst(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    crystal.range,
                    PRISMATIC_EXPLOSION_DAMAGE * crystal.empowerment,
                    PRISMATIC_EXPLOSION_RADIUS,
                    3.0,
                    0.5,
                    &targets,
                    &mut health_query,
                );
            }

            // Despawn active persistent beams
            for (beam_entities, _) in &crystal.active_beams {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
            }

            // Despawn auto-disintegrate beam if present
            if let Some((beam_entities, _)) = &crystal.auto_disintegrate_beam {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
            }

            commands.entity(crystal_entity).try_despawn();

            // Despawn associated range indicator
            for (indicator_entity, indicator) in &indicators {
                if indicator.crystal_entity == crystal_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
            }
        }
    }

    // Clean up orphaned indicators (e.g. crystal dispelled by enemy)
    for (indicator_entity, indicator) in &indicators {
        if crystals.get(indicator.crystal_entity).is_err() {
            commands.entity(indicator_entity).try_despawn();
        }
    }
}

// ===== Black Hole Interaction =====

/// Pulls crystals toward black holes and despawns them if they enter the sphere.
pub(super) fn crystal_black_hole_interaction(
    mut commands: Commands,
    time: Res<Time>,
    black_holes: Query<&BlackHole>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal, &mut Transform)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
) {
    use crate::game::units::wizard::spells::black_hole::constants::GRAVITY_RANGE;

    let delta = time.delta_secs();

    for (crystal_entity, mut crystal, mut transform) in &mut crystals {
        for black_hole in &black_holes {
            let to_bh = black_hole.position - crystal.position;
            let distance = to_bh.length();

            // Check if crystal is inside the black hole sphere
            if black_hole.contains_point(crystal.position) {
                commands.entity(crystal_entity).try_despawn();
                for (indicator_entity, indicator) in &indicators {
                    if indicator.crystal_entity == crystal_entity {
                        commands.entity(indicator_entity).try_despawn();
                    }
                }
                break;
            }

            // Apply gravitational pull
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                let gravity_strength = black_hole.gravitational_strength();
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(2500.0);
                let direction = to_bh.normalize();

                let displacement = direction * pull_strength * delta * 0.01; // Damped movement
                crystal.position += displacement;
                transform.translation = crystal.position;
            }
        }
    }
}

// ===== Range-Limited Despawn =====

/// Despawns crystal-spawned entities that exceed their max range from origin.
pub(super) fn despawn_out_of_range_crystal_spawns(
    mut commands: Commands,
    spawns: Query<(Entity, &Transform, &CrystalSpawn)>,
) {
    for (entity, transform, crystal_spawn) in &spawns {
        if xz_distance(crystal_spawn.origin, transform.translation) > crystal_spawn.max_range {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Despawns CrystalSpawn entities (non-beam) whose lifetime has expired.
/// This handles visual effects like resonance cascade burst rings and prismatic explosions.
pub(super) fn cleanup_expired_crystal_visuals(
    time: Res<Time>,
    mut commands: Commands,
    mut spawns: Query<(Entity, &mut CrystalSpawn), Without<DisintegrateBeam>>,
) {
    let delta = time.delta_secs();
    for (entity, mut spawn) in &mut spawns {
        if let Some(ref mut lifetime) = spawn.lifetime {
            *lifetime -= delta;
            if *lifetime <= 0.0 {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

/// Despawns crystal beams (and their visuals) that have exceeded their lifetime.
pub(super) fn cleanup_expired_crystal_beams(
    mut commands: Commands,
    beams: Query<(Entity, &DisintegrateBeam, &CrystalSpawn)>,
    glow_query: Query<(Entity, &BeamGlow)>,
    flare_query: Query<(Entity, &BeamOriginFlare)>,
    eclipse_query: Query<(Entity, &BeamEclipse)>,
) {
    let mut despawned = Vec::new();
    for (entity, beam, crystal_spawn) in &beams {
        if let Some(lifetime) = crystal_spawn.lifetime
            && beam.time_alive > lifetime
        {
            commands.entity(entity).try_despawn();
            despawned.push(entity);
        }
    }

    if despawned.is_empty() {
        return;
    }

    for (entity, glow) in &glow_query {
        if despawned.contains(&glow.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, flare) in &flare_query {
        if despawned.contains(&flare.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, eclipse) in &eclipse_query {
        if despawned.contains(&eclipse.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
}
