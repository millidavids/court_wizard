//! Arcane Crystal spell systems.

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::chain_lightning::constants as cl_constants;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::disintegrate::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam,
};
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::magic_missile::components::{MagicMissile, TargetTeams};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::meteor_fall::systems::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range, cleanup_spell_caster, handle_spell_release, spawn_circle_indicator,
    update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    disintegrate_constants, finger_of_death_constants, fireball_constants, magic_missile_constants,
    meteor_fall_constants,
};
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
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
fn scaled_count(base: usize, count_mult: f32) -> usize {
    (base as f32 * count_mult).ceil() as usize
}

/// Returns 2 if Spell Echo triggers (30% chance), 1 otherwise.
fn spell_echo_multiplier(rng: &mut impl Rng, spell_echo: bool) -> usize {
    if spell_echo && rng.random::<f32>() < SPELL_ECHO_CHANCE {
        return 2;
    }
    1
}

/// Increments resonance cascade counter if the component is present.
fn increment_resonance(resonance: &mut Option<Mut<ResonanceCascade>>) {
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
fn crystal_beam_geometry(origin: Vec3, target: Vec3, max_range: f32) -> (Vec3, f32) {
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
fn find_random_targets_in_range(
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
fn find_random_enemies_in_range(
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
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    existing_crystals: Query<(Entity, &ArcaneCrystal)>,
    target_assist: Res<TargetAssistWorldPos>,
) {
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
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

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

// ===== Fireball Absorption =====

/// Detects fireball explosions overlapping crystals and emits mini fireballs.
/// Triggers when the crystal is within a fireball's explosion radius, not just on direct hit.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_fireball_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(&mut ArcaneCrystal, Option<&mut ResonanceCascade>)>,
    explosions: Query<(Entity, &FireballExplosion), Without<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (explosion_entity, explosion) in &explosions {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            if crystal.explosions_processed.contains(&explosion_entity) {
                continue;
            }

            let distance = xz_distance(crystal.position, explosion.origin);

            if distance <= explosion.max_radius {
                crystal.explosions_processed.push(explosion_entity);
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::Fireball);
                crystal.auto_cast_timer = 0.0;

                // Spell Echo: chance to double the emission
                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);

                // Emit mini fireballs at random targets
                let count = scaled_count(MINI_FB_COUNT, crystal.count_mult) * echo_mult;
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                let mini_radius =
                    fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                for (_, target_pos) in &enemies {
                    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
                    let direction = (ground_target - crystal.position).normalize();
                    let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                    let velocity = direction * speed;

                    let entity = fireball_systems::spawn_fireball_entity(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        velocity,
                        fireball_constants::DAMAGE_PER_TICK * damage_scale,
                        fireball_constants::DAMAGE_TYPE,
                        fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                        explosion.empowerment * damage_scale,
                        mini_radius,
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);
            }
        }
    }
}

// ===== Disintegrate Beam Absorption =====

/// Detects disintegrate and finger of death beams hitting crystals.
///
/// Disintegrate: Maintains persistent beams that update each frame while channeling.
/// Finger of Death: One-shot burst of beams when the damage beam strikes.
/// All crystal beams are now real DisintegrateBeam entities with CrystalSpawn marker.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_beam_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(&mut ArcaneCrystal, Option<&mut ResonanceCascade>)>,
    disintegrate_beams: Query<&DisintegrateBeam, Without<CrystalSpawn>>,
    fod_beams: Query<(Entity, &FingerOfDeathBeam)>,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    active_talents: Option<Res<ActiveTalents>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let talent_cfg = disintegrate_systems::compute_talent_config(active_talents.as_deref());
    for (mut crystal, mut resonance) in &mut crystals {
        if crystal.permanent {
            continue;
        }
        // === Disintegrate: persistent beams while channeling ===
        let mut hit_by_disintegrate = false;
        for beam in &disintegrate_beams {
            if beam.contains_point(crystal.position) {
                hit_by_disintegrate = true;
                break;
            }
        }

        if hit_by_disintegrate {
            crystal.hit_by_disintegrate = true;
            crystal.mark_absorption();
            crystal.remembered_spell = Some(RememberedSpell::Disintegrate);
            crystal.auto_cast_timer = 0.0;

            // Clean up beam groups whose entities were despawned externally
            crystal.active_beams.retain(|(beam_entities, _)| {
                beam_entities.iter().any(|e| crystal_beams.get(*e).is_ok())
            });

            // Check each existing beam group's target — replace if dead or out of range
            let mut used_targets: Vec<Entity> = Vec::new();
            let mut groups_needing_new_target: Vec<usize> = Vec::new();

            for (i, (beam_entities, target_entity)) in crystal.active_beams.iter().enumerate() {
                if let Ok((_, target_transform)) = targets.get(*target_entity) {
                    let dist = xz_distance(crystal.position, target_transform.translation);
                    if dist <= crystal.range {
                        // Target still valid — update all beams in group to track it
                        let (base_direction, length) = crystal_beam_geometry(
                            crystal.position,
                            target_transform.translation,
                            crystal.range,
                        );
                        for beam_entity in beam_entities {
                            if let Ok(mut beam) =
                                crystal_beams.get_mut(*beam_entity).map(|(_, beam)| beam)
                            {
                                beam.origin = crystal.position;
                                beam.direction =
                                    Quat::from_axis_angle(Vec3::Y, beam.fan_offset_angle)
                                        * base_direction;
                                beam.length = length;
                            }
                        }
                        used_targets.push(*target_entity);
                        continue;
                    }
                }
                // Target dead or out of range
                groups_needing_new_target.push(i);
            }

            // Find replacement targets for beam groups that lost theirs
            if !groups_needing_new_target.is_empty() {
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        xz_distance(crystal.position, transform.translation) <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let rng = &mut game_rng.0;
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.random_range(0..=i);
                    candidates.swap(i, j);
                }

                for (idx, group_idx) in groups_needing_new_target.iter().enumerate() {
                    if let Some((new_target, new_pos)) = candidates.get(idx) {
                        // Despawn old group, spawn new group targeting new enemy
                        let (old_beams, _) = &crystal.active_beams[*group_idx];
                        for beam_entity in old_beams {
                            commands.entity(*beam_entity).try_despawn();
                        }
                        let new_beams = spawn_crystal_disintegrate_beam(
                            &mut commands,
                            &visual_assets,
                            crystal.position,
                            *new_pos,
                            crystal.range,
                            crystal.empowerment,
                            Some(&talent_cfg),
                        );
                        crystal.active_beams[*group_idx] = (new_beams, *new_target);
                        used_targets.push(*new_target);
                    } else {
                        // No replacement available — despawn the group
                        let (old_beams, _) = &crystal.active_beams[*group_idx];
                        for beam_entity in old_beams {
                            commands.entity(*beam_entity).try_despawn();
                        }
                    }
                }

                // Remove groups that had no replacement (iterate in reverse to keep indices valid)
                let candidate_count = candidates.len();
                for (idx, group_idx) in groups_needing_new_target.iter().enumerate().rev() {
                    if idx >= candidate_count {
                        crystal.active_beams.remove(*group_idx);
                    }
                }
            }

            // Spawn new beam groups if we have fewer than the scaled beam count
            let beam_target_count = scaled_count(BEAM_COUNT, crystal.count_mult);
            if crystal.active_beams.len() < beam_target_count {
                let needed = beam_target_count - crystal.active_beams.len();
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        xz_distance(crystal.position, transform.translation) <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let rng = &mut game_rng.0;
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.random_range(0..=i);
                    candidates.swap(i, j);
                }
                candidates.truncate(needed);

                for (target_entity, target_pos) in &candidates {
                    let beam_entities = spawn_crystal_disintegrate_beam(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        *target_pos,
                        crystal.range,
                        crystal.empowerment,
                        Some(&talent_cfg),
                    );
                    crystal.active_beams.push((beam_entities, *target_entity));
                }
            }
        } else if crystal.hit_by_disintegrate {
            // Disintegrate just stopped — despawn persistent beams
            crystal.hit_by_disintegrate = false;
            for (beam_entities, _) in crystal.active_beams.drain(..) {
                for beam_entity in beam_entities {
                    commands.entity(beam_entity).try_despawn();
                }
            }
        }

        // === Finger of Death: one-shot burst of beams ===
        for (fod_entity, fod_beam) in &fod_beams {
            if !fod_beam.has_fired || crystal.fod_beams_processed.contains(&fod_entity) {
                continue;
            }

            if fod_beam.contains_point(crystal.position, fod_beam.beam_width_fired()) {
                crystal.fod_beams_processed.push(fod_entity);
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::FingerOfDeath);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let fod_beam_count = scaled_count(BEAM_COUNT, crystal.count_mult) * echo_mult;
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    fod_beam_count,
                    &targets,
                );
                let damage_scale = BEAM_DAMAGE_SCALE * crystal.damage_mult;
                let fod_damage_per_tick = finger_of_death_constants::DAMAGE * damage_scale
                    / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
                let forked = talent_cfg.forked;
                let offsets: &[f32] = if forked {
                    &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
                } else {
                    &[0.0]
                };
                for (_, target_pos) in &enemies {
                    let (base_direction, length) =
                        crystal_beam_geometry(crystal.position, *target_pos, crystal.range);
                    for &offset in offsets {
                        let direction = if offset.abs() > 0.001 {
                            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
                        } else {
                            base_direction
                        };
                        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                            &mut commands,
                            &visual_assets,
                            crystal.position,
                            direction,
                            length,
                            crystal.empowerment,
                            fod_damage_per_tick,
                            Some(&talent_cfg),
                            damage_scale,
                            offset,
                        );
                        commands.entity(beam_entity).insert(CrystalSpawn {
                            origin: crystal.position,
                            max_range: crystal.range,
                            lifetime: Some(BEAM_DURATION),
                        });
                    }
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, fod_beam_count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);
            }
        }
    }
}

// ===== Meteor Absorption =====

/// Detects meteors hitting crystals and emits mini meteors.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_meteor_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(&mut ArcaneCrystal, Option<&mut ResonanceCascade>)>,
    meteors: Query<(Entity, &Transform, &MeteorProjectile)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (meteor_entity, meteor_transform, meteor) in &meteors {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = xz_distance(crystal.position, meteor_transform.translation);

            // Check if meteor is near the crystal's XZ position and falling through it
            if distance <= crystal.collision_radius
                && meteor_transform.translation.y <= crystal.position.y + CRYSTAL_HEIGHT
                && meteor_transform.translation.y >= 0.0
            {
                // Absorb the meteor
                commands.entity(meteor_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::Meteor);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(2, crystal.count_mult) * echo_mult;
                let damage_scale = DAMAGE_SCALE * crystal.damage_mult;

                // Emit mini meteors at random targets
                let enemies = find_random_targets_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &targets,
                );

                for (_, target_pos) in &enemies {
                    let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
                    let damage = meteor.damage * damage_scale;
                    let explosion_radius = meteor.explosion_radius * SIZE_SCALE;

                    let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                        &mut commands,
                        &visual_assets,
                        spawn_pos,
                        Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                        damage,
                        explosion_radius,
                        meteor.empowerment,
                        mini_radius,
                        MeteorProjectileTalentFlags::default(),
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                        lifetime: None,
                    });
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}

// ===== Magic Missile Absorption =====

/// Detects magic missiles hitting crystals and emits mini homing missiles.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_magic_missile_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(&mut ArcaneCrystal, Option<&mut ResonanceCascade>)>,
    missiles: Query<(Entity, &Transform, &MagicMissile), Without<CrystalSpawn>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (missile_entity, missile_transform, _missile) in &missiles {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = missile_transform.translation.distance(crystal.position);

            if distance <= crystal.collision_radius {
                // Absorb the missile
                commands.entity(missile_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::MagicMissile);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(MINI_MISSILE_COUNT, crystal.count_mult) * echo_mult;

                // Emit mini missiles at random enemy targets (not defenders)
                let targets = find_random_enemies_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &enemies,
                );

                let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

                for (target_entity, target_pos) in &targets {
                    let direction = (*target_pos - crystal.position).normalize();
                    let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                    let initial_velocity = direction * speed;

                    let rng = &mut game_rng.0;
                    let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

                    spawn_crystal_mini_missile(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        crystal.range,
                        initial_velocity,
                        wobble_offset,
                        Some(*target_entity),
                        mini_radius,
                        crystal.damage_mult,
                    );
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}

// ===== Chain Lightning Absorption =====

/// Detects chain lightning hitting crystals and emits lightning arcs.
/// This is called when chain lightning bounces to a crystal (crystal is added
/// as a valid bounce target in the chain lightning systems).
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_chain_lightning_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal, Option<&mut ResonanceCascade>)>,
    bolts: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningBolt,
    >,
    groups: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningGroup,
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    // Check if any bolt's last_hit_position matches a crystal position
    // (chain lightning system sets crystal as a bounce target, so the bolt
    // will have the crystal's position as last_hit_position after bouncing to it)
    for (crystal_entity, mut crystal, mut resonance) in &mut crystals {
        if crystal.permanent {
            continue;
        }
        for bolt in &bolts {
            // Check if this bolt just bounced to this crystal
            let dist = bolt.last_hit_position.distance(crystal.position);
            if dist > crystal.collision_radius {
                continue;
            }

            // Check if we're in the group's hit list (meaning we were targeted)
            let Ok(group) = groups.get(bolt.group_entity) else {
                continue;
            };
            if !group.hit_entities.contains(&crystal_entity) {
                continue;
            }

            crystal.mark_absorption();
            crystal.remembered_spell = Some(RememberedSpell::ChainLightning);
            crystal.auto_cast_timer = 0.0;

            let rng = &mut game_rng.0;
            let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
            let count = scaled_count(LIGHTNING_ARC_COUNT, crystal.count_mult) * echo_mult;

            // Emit arcs to random targets
            let enemies =
                find_random_targets_in_range(rng, crystal.position, crystal.range, count, &targets);

            let damage = bolt.current_damage * DAMAGE_SCALE * crystal.damage_mult;

            for (target_entity, target_pos) in &enemies {
                // Apply damage
                if let Ok((mut health, mut temp_hp, has_spell_shield)) =
                    health_query.get_mut(*target_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Electric,
                        has_spell_shield,
                    );
                }

                // Spawn arc visual using shared chain lightning helper
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    *target_pos,
                    0,
                    crystal.empowerment,
                );
            }

            // Track progress
            progress.increment(Spell::ArcaneCrystal, count as u32);

            // Resonance cascade
            increment_resonance(&mut resonance);

            break; // Only process once per crystal per frame
        }
    }
}

// ===== Auto-Cast System =====

/// Auto-casts the remembered spell on a timer.
///
/// This runs independently of spell absorption — the crystal fires spells
/// on its own based on whatever spell last hit it.
/// Disintegrate is special: it channels a single constant beam.
#[allow(clippy::too_many_arguments)]
pub(super) fn auto_cast_remembered_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal)>,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let talent_cfg = disintegrate_systems::compute_talent_config(active_talents.as_deref());
    let delta = time.delta_secs();

    // Collect crystal data to avoid borrow conflicts (skip permanent turrets).
    // Store Entity IDs for O(1) lookup via get_mut() instead of O(n) iter_mut().nth().
    let crystal_data: Vec<_> = crystals
        .iter()
        .filter(|(_, c)| !c.permanent)
        .map(|(e, c)| {
            (
                e,
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.auto_cast_timer,
                c.auto_disintegrate_beam.clone(),
                c.damage_mult,
                c.count_mult,
            )
        })
        .collect();

    for (
        entity,
        position,
        range,
        empowerment,
        remembered,
        timer,
        auto_beam,
        damage_mult,
        count_mult,
    ) in crystal_data.into_iter()
    {
        let Some(remembered) = remembered else {
            // No remembered spell — clean up any lingering auto-disintegrate beam
            if let Some((beam_entities, _)) = &auto_beam {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
                if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            continue;
        };

        // === Special case: Disintegrate = constant single beam ===
        if remembered == RememberedSpell::Disintegrate {
            handle_auto_disintegrate(
                &mut game_rng.0,
                entity,
                position,
                range,
                empowerment,
                auto_beam,
                &mut commands,
                &visual_assets,
                &mut crystal_beams,
                &targets,
                &mut crystals,
                &talent_cfg,
            );
            continue;
        }

        // === Timer-based auto-cast for all other spells ===

        // Clean up any lingering auto-disintegrate beam if spell changed
        if let Some((beam_entities, _)) = &auto_beam {
            for beam_entity in beam_entities {
                commands.entity(*beam_entity).try_despawn();
            }
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_disintegrate_beam = None;
            }
        }

        let new_timer = timer + delta;
        let interval = remembered.auto_cast_interval();

        if new_timer >= interval {
            // Reset timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = 0.0;
                crystal.trigger_pulse();
            }

            let autocast = CrystalAutocastParams {
                position,
                range,
                empowerment,
                damage_mult,
                count_mult,
            };

            match remembered {
                RememberedSpell::MagicMissile => {
                    auto_cast_magic_missiles(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &enemies,
                    );
                }
                RememberedSpell::Fireball => {
                    auto_cast_fireballs(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::ChainLightning => {
                    auto_cast_chain_lightning(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &mut health_query,
                    );
                }
                RememberedSpell::Meteor => {
                    auto_cast_meteors(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::FingerOfDeath => {
                    auto_cast_fod_beams(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &talent_cfg,
                    );
                }
                RememberedSpell::Disintegrate => unreachable!(),
            }
        } else {
            // Just advance timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = new_timer;
            }
        }
    }
}

/// Manages a persistent auto-disintegrate beam group.
///
/// Instead of despawning/respawning beams every frame (which resets time_alive
/// and breaks the growth animation + damage), we update beam fields in-place.
/// New beams are only spawned when the old target dies/leaves range.
#[allow(clippy::too_many_arguments)]
fn handle_auto_disintegrate(
    rng: &mut impl Rng,
    crystal_entity: Entity,
    position: Vec3,
    range: f32,
    empowerment: f32,
    auto_beam: Option<(Vec<Entity>, Entity)>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_beams: &mut Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    crystals: &mut Query<(Entity, &mut ArcaneCrystal)>,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    if let Some((beam_entities, target_entity)) = auto_beam {
        // Check if any beam entity still exists
        let any_alive = beam_entities.iter().any(|e| crystal_beams.get(*e).is_ok());
        if !any_alive {
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = None;
            }
            // Fall through to spawn a new beam below
        } else if let Ok((_, target_transform)) = targets.get(target_entity) {
            // Target alive — check range
            let dist = xz_distance(position, target_transform.translation);
            if dist <= range {
                // Target still valid — update all beams to track it in-place
                let (base_direction, length) =
                    crystal_beam_geometry(position, target_transform.translation, range);
                for beam_entity in &beam_entities {
                    if let Ok((_, mut beam)) = crystal_beams.get_mut(*beam_entity) {
                        beam.origin = position;
                        beam.direction =
                            Quat::from_axis_angle(Vec3::Y, beam.fan_offset_angle) * base_direction;
                        beam.length = length;
                    }
                }
                return;
            }
            // Target out of range — fall through to replace beam
        } else {
            // Target dead — fall through to replace beam
        }
        // Despawn old beams and find new target
        despawn_beam_group(commands, &beam_entities);
        let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
        if let Some((new_target, new_pos)) = new_targets.first() {
            let new_beams = spawn_crystal_disintegrate_beam(
                commands,
                assets,
                position,
                *new_pos,
                range,
                empowerment,
                Some(talent_cfg),
            );
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = Some((new_beams, *new_target));
            }
        } else if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = None;
        }
        return;
    }

    // No beam exists — try to spawn one
    let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
    if let Some((target_entity, target_pos)) = new_targets.first() {
        let beam_entities = spawn_crystal_disintegrate_beam(
            commands,
            assets,
            position,
            *target_pos,
            range,
            empowerment,
            Some(talent_cfg),
        );
        if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = Some((beam_entities, *target_entity));
        }
    }
}

/// Helper to despawn all beams in a group.
fn despawn_beam_group(commands: &mut Commands, beam_entities: &[Entity]) {
    for beam_entity in beam_entities {
        commands.entity(*beam_entity).try_despawn();
    }
}

/// Spawns crystal disintegrate beam(s) with damage scaling and CrystalSpawn marker.
/// When forked talent is active, spawns 3 beams in a fan pattern.
/// Returns all spawned beam entities.
fn spawn_crystal_disintegrate_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    max_range: f32,
    empowerment: f32,
    talent_cfg: Option<&disintegrate_systems::TalentConfig>,
) -> Vec<Entity> {
    let (base_direction, length) = crystal_beam_geometry(origin, target, max_range);
    let forked = talent_cfg.is_some_and(|cfg| cfg.forked);

    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };

    let mut entities = Vec::with_capacity(offsets.len());
    for &offset in offsets {
        let direction = if offset.abs() > 0.001 {
            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
        } else {
            base_direction
        };
        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
            commands,
            assets,
            origin,
            direction,
            length,
            empowerment,
            disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
            talent_cfg,
            BEAM_DAMAGE_SCALE,
            offset,
        );
        commands.entity(beam_entity).insert(CrystalSpawn {
            origin,
            max_range,
            lifetime: None,
        });
        entities.push(beam_entity);
    }
    entities
}

/// Shared parameters for crystal auto-cast helper functions.
struct CrystalAutocastParams {
    position: Vec3,
    range: f32,
    empowerment: f32,
    damage_mult: f32,
    count_mult: f32,
}

/// Auto-casts mini magic missiles at random enemies (not defenders).
fn auto_cast_magic_missiles(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    enemies: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    let targets = find_random_enemies_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_MISSILE_COUNT, params.count_mult),
        enemies,
    );
    let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

    for (target_entity, target_pos) in &targets {
        let direction = (*target_pos - params.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;

        let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            commands,
            assets,
            params.position,
            params.range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
            params.damage_mult,
        );
    }
}

/// Auto-casts mini fireballs at random enemies.
fn auto_cast_fireballs(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_FB_COUNT, params.count_mult),
        targets,
    );
    let mini_radius = fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;

    for (_, target_pos) in &enemies {
        let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let direction = (ground_target - params.position).normalize();
        let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
        let velocity = direction * speed;

        let entity = fireball_systems::spawn_fireball_entity(
            commands,
            assets,
            params.position,
            velocity,
            fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE * params.damage_mult,
            fireball_constants::DAMAGE_TYPE,
            fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
            params.empowerment * DAMAGE_SCALE * params.damage_mult,
            mini_radius,
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts chain lightning arcs at random enemies.
fn auto_cast_chain_lightning(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(LIGHTNING_ARC_COUNT, params.count_mult),
        targets,
    );
    let damage =
        cl_constants::INITIAL_DAMAGE * DAMAGE_SCALE * params.empowerment * params.damage_mult;

    for (target_entity, target_pos) in &enemies {
        if let Ok((mut health, mut temp_hp, has_spell_shield)) =
            health_query.get_mut(*target_entity)
        {
            apply_spell_damage(
                commands,
                *target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Electric,
                has_spell_shield,
            );
        }

        chain_lightning_systems::spawn_arc(
            commands,
            assets,
            params.position,
            *target_pos,
            0,
            params.empowerment,
        );
    }
}

/// Auto-casts mini meteors at random enemies.
fn auto_cast_meteors(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(2, params.count_mult),
        targets,
    );
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (_, target_pos) in &enemies {
        let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
        let damage = meteor_fall_constants::METEOR_DAMAGE * DAMAGE_SCALE * params.damage_mult;
        let explosion_radius = meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE;

        let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
            commands,
            assets,
            spawn_pos,
            Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
            damage,
            explosion_radius,
            params.empowerment,
            mini_radius,
            MeteorProjectileTalentFlags::default(),
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts Finger of Death beams at random enemies.
fn auto_cast_fod_beams(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(BEAM_COUNT, params.count_mult),
        targets,
    );
    let fod_damage_per_tick =
        finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE * params.damage_mult
            / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
    let forked = talent_cfg.forked;
    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };
    for (_, target_pos) in &enemies {
        let (base_direction, length) =
            crystal_beam_geometry(params.position, *target_pos, params.range);
        for &offset in offsets {
            let direction = if offset.abs() > 0.001 {
                Quat::from_axis_angle(Vec3::Y, offset) * base_direction
            } else {
                base_direction
            };
            let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                commands,
                assets,
                params.position,
                direction,
                length,
                params.empowerment,
                fod_damage_per_tick,
                Some(talent_cfg),
                BEAM_DAMAGE_SCALE,
                offset,
            );
            commands.entity(beam_entity).insert(CrystalSpawn {
                origin: params.position,
                max_range: params.range,
                lifetime: Some(BEAM_DURATION),
            });
        }
    }
}

/// Spawns a crystal mini magic missile with pre-advanced homing.
///
/// Shared helper for both absorption and auto-cast.
#[allow(clippy::too_many_arguments)]
fn spawn_crystal_mini_missile(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_position: Vec3,
    crystal_range: f32,
    initial_velocity: Vec3,
    wobble_offset: f32,
    target: Option<Entity>,
    visual_radius: f32,
    damage_mult: f32,
) {
    let mut mini_missile =
        crate::game::units::wizard::spells::magic_missile::components::MagicMissile::new(
            initial_velocity,
            wobble_offset,
            target,
            DAMAGE_SCALE * damage_mult,
            TargetTeams::AttackersAndUndead,
            crystal_range,
            crystal_position,
        );
    mini_missile.time_alive = MINI_MISSILE_HOMING_ADVANCE;

    commands.spawn((
        mini_missile,
        CrystalSpawn {
            origin: crystal_position,
            max_range: crystal_range,
            lifetime: None,
        },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.crystal_mini_missile.clone()),
        Transform::from_translation(crystal_position).with_scale(Vec3::splat(visual_radius)),
        OnGameplayScreen,
    ));
}

// ===== Resonance Cascade Burst =====

/// When a crystal's resonance counter reaches the threshold, emit a powerful
/// damage burst to all enemies in range and reset the counter.
#[allow(clippy::too_many_arguments)]
pub(super) fn resonance_cascade_burst(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(&mut ArcaneCrystal, &mut ResonanceCascade)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    for (mut crystal, mut resonance) in &mut crystals {
        if resonance.absorptions < RESONANCE_CASCADE_THRESHOLD {
            continue;
        }

        resonance.absorptions = 0;
        crystal.trigger_pulse();

        let hit_count = crystal_aoe_burst(
            &mut commands,
            &visual_assets,
            crystal.position,
            crystal.range,
            RESONANCE_CASCADE_DAMAGE * crystal.empowerment,
            RESONANCE_CASCADE_RADIUS,
            2.0,
            0.3,
            &targets,
            &mut health_query,
        );

        if hit_count > 0 {
            progress.increment(Spell::ArcaneCrystal, hit_count);
        }
    }
}

/// Shared AoE burst: damages all enemies in radius and spawns a visual ring.
/// Returns the number of enemies hit.
#[allow(clippy::too_many_arguments)]
fn crystal_aoe_burst(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    position: Vec3,
    crystal_range: f32,
    damage: f32,
    radius: f32,
    visual_height: f32,
    visual_lifetime: f32,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) -> u32 {
    let mut hit_count: u32 = 0;

    for (target_entity, target_transform) in targets {
        let dist = xz_distance(position, target_transform.translation);
        if dist > radius {
            continue;
        }

        if let Ok((mut health, mut temp_hp, has_spell_shield)) = health_query.get_mut(target_entity)
        {
            apply_spell_damage(
                commands,
                target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Force,
                has_spell_shield,
            );
            hit_count += 1;
        }
    }

    // Spawn burst visual — expanding ring
    let burst_pos = Vec3::new(position.x, visual_height, position.z);
    commands.spawn((
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(visual_assets.arcane_crystal_indicator.clone()),
        Transform::from_translation(burst_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        CrystalSpawn {
            origin: position,
            max_range: crystal_range,
            lifetime: Some(visual_lifetime),
        },
        OnGameplayScreen,
    ));

    hit_count
}

// ===== Auto-Crystal Firing =====

/// Fires homing magic missiles at nearby enemies on a timer.
#[allow(clippy::too_many_arguments)]
pub(super) fn auto_crystal_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(
        &ArcaneCrystal,
        &mut AutoCrystalTimer,
        Option<&mut ResonanceCascade>,
    )>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    let delta = time.delta_secs();

    for (crystal, mut timer, mut resonance) in &mut crystals {
        timer.timer += delta;

        // Overcharged Matrix speeds up the fire rate
        let interval = AUTO_CRYSTAL_INTERVAL / crystal.count_mult;
        if timer.timer < interval {
            continue;
        }
        timer.timer -= interval;

        // Find a random enemy in range
        let targets = find_random_enemies_in_range(
            &mut game_rng.0,
            crystal.position,
            crystal.range,
            1,
            &enemies,
        );

        let Some((target_entity, target_pos)) = targets.first() else {
            continue;
        };

        let direction = (*target_pos - crystal.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;
        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

        let rng = &mut game_rng.0;
        let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            &mut commands,
            &visual_assets,
            crystal.position,
            crystal.range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
            crystal.damage_mult,
        );

        // Increment resonance cascade counter on each turret shot
        increment_resonance(&mut resonance);

        progress.increment(Spell::ArcaneCrystal, 1);
    }
}

// ===== Crystal Network Chaining =====

/// When a crystal absorbs a spell, chain the absorption to nearby crystals
/// with CrystalNetwork marker. Each chain re-triggers the crystal's emission
/// at reduced damage.
#[allow(clippy::too_many_arguments)]
pub(super) fn crystal_network_chain(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal), With<CrystalNetwork>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
) {
    // Early return if no crystal absorbed this frame
    if !crystals.iter().any(|(_, c)| c.just_absorbed) {
        return;
    }

    // Collect crystal positions and remembered spells for chaining
    let crystal_data: Vec<(Entity, Vec3, f32, f32, Option<RememberedSpell>, bool)> = crystals
        .iter()
        .map(|(e, c)| {
            (
                e,
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.just_absorbed,
            )
        })
        .collect();

    // For each crystal that just pulsed, check if nearby crystals should chain
    for (source_entity, source_pos, _source_range, _source_emp, source_spell, source_pulsed) in
        &crystal_data
    {
        if !source_pulsed {
            continue;
        }
        let Some(remembered) = source_spell else {
            continue;
        };

        for (target_entity, target_pos, _target_range, _target_emp, _target_spell, target_pulsed) in
            &crystal_data
        {
            if source_entity == target_entity || *target_pulsed {
                continue;
            }

            let dist = xz_distance(*source_pos, *target_pos);

            if dist > CRYSTAL_NETWORK_CHAIN_RANGE {
                continue;
            }

            // Chain: trigger the target crystal to emit based on the source's remembered spell
            if let Ok((_, mut target_crystal)) = crystals.get_mut(*target_entity) {
                target_crystal.trigger_pulse();

                // Emit a reduced emission from the chained crystal
                let chain_damage_scale = DAMAGE_SCALE * target_crystal.damage_mult * 0.5;
                match remembered {
                    RememberedSpell::MagicMissile => {
                        let count =
                            scaled_count(MINI_MISSILE_COUNT / 2 + 1, target_crystal.count_mult);
                        let mini_targets = find_random_enemies_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &enemies,
                        );
                        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;
                        for (te, tp) in &mini_targets {
                            let direction = (*tp - target_crystal.position).normalize();
                            let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                            let rng = &mut game_rng.0;
                            let wobble = rng.random_range(0.0..std::f32::consts::TAU);
                            spawn_crystal_mini_missile(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                target_crystal.range,
                                direction * speed,
                                wobble,
                                Some(*te),
                                mini_radius,
                                target_crystal.damage_mult,
                            );
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Fireball => {
                        let count = scaled_count(MINI_FB_COUNT / 2 + 1, target_crystal.count_mult);
                        let fire_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius =
                            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                        for (_, tp) in &fire_targets {
                            let ground = Vec3::new(tp.x, 0.0, tp.z);
                            let direction = (ground - target_crystal.position).normalize();
                            let velocity =
                                direction * fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                            let entity = fireball_systems::spawn_fireball_entity(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                velocity,
                                fireball_constants::DAMAGE_PER_TICK * chain_damage_scale,
                                fireball_constants::DAMAGE_TYPE,
                                fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment * chain_damage_scale,
                                mini_radius,
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Meteor => {
                        let count = scaled_count(1, target_crystal.count_mult);
                        let meteor_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;
                        for (_, tp) in &meteor_targets {
                            let spawn_pos = Vec3::new(tp.x, MINI_METEOR_SPAWN_HEIGHT, tp.z);
                            let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                                &mut commands,
                                &visual_assets,
                                spawn_pos,
                                Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                                meteor_fall_constants::METEOR_DAMAGE * chain_damage_scale,
                                meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment,
                                mini_radius,
                                MeteorProjectileTalentFlags::default(),
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    _ => {
                        // Chain lightning, disintegrate, FoD — just pulse without extra emission
                        // (beams are too complex to duplicate in chain)
                    }
                }

                // Spawn visual arc between crystals
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    *source_pos,
                    target_crystal.position,
                    0,
                    target_crystal.empowerment,
                );
            }
        }
    }
}
