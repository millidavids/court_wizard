//! Arcane Crystal spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
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
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::chain_lightning::constants as cl_constants;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::magic_missile::components::{MagicMissile, TargetTeams};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    disintegrate_constants, finger_of_death_constants, fireball_constants, magic_missile_constants,
    meteor_fall_constants,
};
use crate::networking::snapshot::SpellEffectKind;
use crate::game::units::wizard::spells::utils::{clamp_to_spell_range, get_cursor_world_position, spawn_circle_indicator};

// ===== Helper Functions =====

/// Finds random targets within range of a position.
/// Returns up to `count` random targets from any team (spells are indiscriminate).
fn find_random_targets_in_range(
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform), Without<Corpse>>,
) -> Vec<(Entity, Vec3)> {
    let mut rng = rand::thread_rng();

    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, transform)| {
            let dist = Vec3::new(
                crystal_pos.x - transform.translation.x,
                0.0,
                crystal_pos.z - transform.translation.z,
            )
            .length();
            dist <= range
        })
        .map(|(entity, transform)| (entity, transform.translation))
        .collect();

    // Shuffle and take up to count
    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
        candidates.swap(i, j);
    }
    candidates.truncate(count);
    candidates
}

/// Finds random enemy targets (Attackers/Undead only) within range.
/// Used for magic missiles which should not target defenders.
fn find_random_enemies_in_range(
    crystal_pos: Vec3,
    range: f32,
    count: usize,
    units: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) -> Vec<(Entity, Vec3)> {
    let mut rng = rand::thread_rng();

    let mut candidates: Vec<(Entity, Vec3)> = units
        .iter()
        .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
        .filter(|(_, transform, _)| {
            let dist = Vec3::new(
                crystal_pos.x - transform.translation.x,
                0.0,
                crystal_pos.z - transform.translation.z,
            )
            .length();
            dist <= range
        })
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    let len = candidates.len();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
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
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut ArcaneCrystalCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
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
    if primed_spell.spell != Spell::ArcaneCrystal {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, SPELL_ORIGIN, wizard.spell_range));

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &visual_assets,
                    visual_assets.arcane_crystal_indicator.clone(),
                    pos,
                    CRYSTAL_RANGE * primed_spell.empowerment,
                    CIRCLE_Y_POSITION,
                )
                .insert(ArcaneCrystalCircleIndicator::new(pos, primed_spell.empowerment))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor
                && let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = pos;
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
    }

    let completed =
        arcane_crystal_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
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
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.arcane_crystal_cast,
                    indicator.position,
                    &game_config,
                );
            }
            commands.entity(indicator_entity).despawn();
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
) {
    let range = CRYSTAL_RANGE * empowerment;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;
    let duration = CRYSTAL_DURATION * empowerment;
    let height = CRYSTAL_HEIGHT * empowerment;

    let crystal_pos = Vec3::new(position.x, height / 2.0, position.z);

    // cross_plane_sphere has radius 1.0, scale to height/3.0 with vertical stretch
    let sphere_radius = height / 3.0;

    let crystal_entity = commands
        .spawn((
            ArcaneCrystal::new(crystal_pos, range, duration, collision_radius, empowerment),
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
        ))
        .id();

    // Spawn range indicator circle
    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.crystal_range_indicator.clone()),
        Transform::from_translation(Vec3::new(position.x, RANGE_INDICATOR_Y, position.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(range)),
        CrystalRangeIndicator { crystal_entity },
        OnGameplayScreen,
    ));
}

// ===== Crystal Visuals & Lifetime =====

/// Updates crystal rotation, pulse animation, and lifetime.
pub(super) fn update_crystal_visuals(
    time: Res<Time>,
    mut crystals: Query<(&mut ArcaneCrystal, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (mut crystal, mut transform) in &mut crystals {
        crystal.time_alive += delta;

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
pub(super) fn cleanup_expired_crystals(
    mut commands: Commands,
    crystals: Query<(Entity, &ArcaneCrystal)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
) {
    for (crystal_entity, crystal) in &crystals {
        if crystal.time_alive >= crystal.duration {
            // Despawn active persistent beams
            for (beam_entity, _) in &crystal.active_beams {
                commands.entity(*beam_entity).despawn();
            }

            // Despawn auto-disintegrate beam if present
            if let Some((beam_entity, _)) = crystal.auto_disintegrate_beam {
                commands.entity(beam_entity).try_despawn();
            }

            commands.entity(crystal_entity).despawn();

            // Despawn associated range indicator
            for (indicator_entity, indicator) in &indicators {
                if indicator.crystal_entity == crystal_entity {
                    commands.entity(indicator_entity).despawn();
                }
            }
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
                commands.entity(crystal_entity).despawn();
                for (indicator_entity, indicator) in &indicators {
                    if indicator.crystal_entity == crystal_entity {
                        commands.entity(indicator_entity).despawn();
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
        let distance = Vec3::new(
            crystal_spawn.origin.x - transform.translation.x,
            0.0,
            crystal_spawn.origin.z - transform.translation.z,
        )
        .length();

        if distance > crystal_spawn.max_range {
            commands.entity(entity).despawn();
        }
    }
}

// ===== Fireball Absorption =====

/// Detects fireball explosions overlapping crystals and emits mini fireballs.
/// Triggers when the crystal is within a fireball's explosion radius, not just on direct hit.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_fireball_hits(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<&mut ArcaneCrystal>,
    explosions: Query<(Entity, &FireballExplosion), Without<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
) {
    for (explosion_entity, explosion) in &explosions {
        for mut crystal in &mut crystals {
            if crystal.explosions_processed.contains(&explosion_entity) {
                continue;
            }

            let distance = Vec3::new(
                crystal.position.x - explosion.origin.x,
                0.0,
                crystal.position.z - explosion.origin.z,
            )
            .length();

            if distance <= explosion.max_radius {
                crystal.explosions_processed.push(explosion_entity);
                crystal.trigger_pulse();
                crystal.remembered_spell = Some(RememberedSpell::Fireball);
                crystal.auto_cast_timer = 0.0;

                // Emit mini fireballs at random targets
                let enemies = find_random_targets_in_range(
                    crystal.position,
                    crystal.range,
                    MINI_FB_COUNT,
                    &targets,
                );

                let mini_radius =
                    fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;

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
                        fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE,
                        fireball_constants::DAMAGE_TYPE,
                        fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                        explosion.empowerment * DAMAGE_SCALE,
                        mini_radius,
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                    });
                }
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
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<&mut ArcaneCrystal>,
    disintegrate_beams: Query<&DisintegrateBeam, Without<CrystalSpawn>>,
    fod_beams: Query<(Entity, &FingerOfDeathBeam)>,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
) {
    for mut crystal in &mut crystals {
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
            crystal.trigger_pulse();
            crystal.remembered_spell = Some(RememberedSpell::Disintegrate);
            crystal.auto_cast_timer = 0.0;

            // Clean up beams whose beam entity was despawned externally
            crystal
                .active_beams
                .retain(|(beam_e, _)| crystal_beams.get(*beam_e).is_ok());

            // Check each existing beam's target — replace if dead or out of range
            let mut used_targets: Vec<Entity> = Vec::new();
            let mut beams_needing_new_target: Vec<usize> = Vec::new();

            for (i, (beam_entity, target_entity)) in crystal.active_beams.iter().enumerate() {
                if let Ok((_, target_transform)) = targets.get(*target_entity) {
                    let dist = Vec3::new(
                        crystal.position.x - target_transform.translation.x,
                        0.0,
                        crystal.position.z - target_transform.translation.z,
                    )
                    .length();
                    if dist <= crystal.range {
                        // Target still valid — update beam to track it
                        if let Ok(mut beam) =
                            crystal_beams.get_mut(*beam_entity).map(|(_, beam)| beam)
                        {
                            let direction =
                                (target_transform.translation - crystal.position).normalize();
                            let length = crystal
                                .position
                                .distance(target_transform.translation)
                                .min(crystal.range);
                            beam.origin = crystal.position;
                            beam.direction = direction;
                            beam.length = length;
                        }
                        used_targets.push(*target_entity);
                        continue;
                    }
                }
                // Target dead or out of range
                beams_needing_new_target.push(i);
            }

            // Find replacement targets for beams that lost theirs
            if !beams_needing_new_target.is_empty() {
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        let dist = Vec3::new(
                            crystal.position.x - transform.translation.x,
                            0.0,
                            crystal.position.z - transform.translation.z,
                        )
                        .length();
                        dist <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let mut rng = rand::thread_rng();
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.gen_range(0..=i);
                    candidates.swap(i, j);
                }

                for (idx, beam_idx) in beams_needing_new_target.iter().enumerate() {
                    if let Some((new_target, new_pos)) = candidates.get(idx) {
                        let (beam_entity, _) = crystal.active_beams[*beam_idx];
                        crystal.active_beams[*beam_idx] = (beam_entity, *new_target);
                        if let Ok(mut beam) =
                            crystal_beams.get_mut(beam_entity).map(|(_, beam)| beam)
                        {
                            let direction = (*new_pos - crystal.position).normalize();
                            let length = crystal.position.distance(*new_pos).min(crystal.range);
                            beam.origin = crystal.position;
                            beam.direction = direction;
                            beam.length = length;
                        }
                        used_targets.push(*new_target);
                    } else {
                        // No replacement available — despawn the beam
                        let (beam_entity, _) = crystal.active_beams[*beam_idx];
                        commands.entity(beam_entity).despawn();
                    }
                }

                // Remove beams that had no replacement (iterate in reverse to keep indices valid)
                let candidate_count = candidates.len();
                for (idx, beam_idx) in beams_needing_new_target.iter().enumerate().rev() {
                    if idx >= candidate_count {
                        crystal.active_beams.remove(*beam_idx);
                    }
                }
            }

            // Spawn new beams if we have fewer than BEAM_COUNT
            if crystal.active_beams.len() < BEAM_COUNT {
                let needed = BEAM_COUNT - crystal.active_beams.len();
                let mut candidates: Vec<(Entity, Vec3)> = targets
                    .iter()
                    .filter(|(e, _)| !used_targets.contains(e))
                    .filter(|(_, transform)| {
                        let dist = Vec3::new(
                            crystal.position.x - transform.translation.x,
                            0.0,
                            crystal.position.z - transform.translation.z,
                        )
                        .length();
                        dist <= crystal.range
                    })
                    .map(|(entity, transform)| (entity, transform.translation))
                    .collect();

                let mut rng = rand::thread_rng();
                let len = candidates.len();
                for i in (1..len).rev() {
                    let j = rng.gen_range(0..=i);
                    candidates.swap(i, j);
                }
                candidates.truncate(needed);

                for (target_entity, target_pos) in &candidates {
                    let direction = (*target_pos - crystal.position).normalize();
                    let length = crystal.position.distance(*target_pos).min(crystal.range);
                    let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        direction,
                        length,
                        crystal.empowerment,
                        disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
                    );
                    commands.entity(beam_entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                    });
                    crystal.active_beams.push((beam_entity, *target_entity));
                }
            }
        } else if crystal.hit_by_disintegrate {
            // Disintegrate just stopped — despawn persistent beams
            crystal.hit_by_disintegrate = false;
            for (beam_entity, _) in crystal.active_beams.drain(..) {
                commands.entity(beam_entity).despawn();
            }
        }

        // === Finger of Death: one-shot burst of beams ===
        for (fod_entity, fod_beam) in &fod_beams {
            if !fod_beam.has_fired || crystal.fod_beams_processed.contains(&fod_entity) {
                continue;
            }

            if fod_beam.contains_point(crystal.position, fod_beam.beam_width_fired()) {
                crystal.fod_beams_processed.push(fod_entity);
                crystal.trigger_pulse();
                crystal.remembered_spell = Some(RememberedSpell::FingerOfDeath);
                crystal.auto_cast_timer = 0.0;

                let enemies = find_random_targets_in_range(
                    crystal.position,
                    crystal.range,
                    BEAM_COUNT,
                    &targets,
                );
                let fod_damage_per_tick = finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE
                    / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
                for (_, target_pos) in &enemies {
                    let direction = (*target_pos - crystal.position).normalize();
                    let length = crystal.position.distance(*target_pos).min(crystal.range);
                    let beam_entity = disintegrate_systems::spawn_beam_with_damage(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        direction,
                        length,
                        crystal.empowerment,
                        fod_damage_per_tick,
                    );
                    commands.entity(beam_entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                    });
                }
            }
        }
    }
}

// ===== Meteor Absorption =====

/// Detects meteors hitting crystals and emits mini meteors.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_meteor_hits(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<&mut ArcaneCrystal>,
    meteors: Query<(Entity, &Transform, &MeteorProjectile)>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
) {
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (meteor_entity, meteor_transform, meteor) in &meteors {
        for mut crystal in &mut crystals {
            let distance = Vec3::new(
                crystal.position.x - meteor_transform.translation.x,
                0.0,
                crystal.position.z - meteor_transform.translation.z,
            )
            .length();

            // Check if meteor is near the crystal's XZ position and falling through it
            if distance <= crystal.collision_radius
                && meteor_transform.translation.y <= crystal.position.y + CRYSTAL_HEIGHT
                && meteor_transform.translation.y >= 0.0
            {
                // Absorb the meteor
                commands.entity(meteor_entity).despawn();
                crystal.trigger_pulse();
                crystal.remembered_spell = Some(RememberedSpell::Meteor);
                crystal.auto_cast_timer = 0.0;

                // Emit mini meteors at random targets
                let enemies =
                    find_random_targets_in_range(crystal.position, crystal.range, 2, &targets);

                for (_, target_pos) in &enemies {
                    let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
                    let damage = meteor.damage * DAMAGE_SCALE;
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
                    );
                    commands.entity(entity).insert(CrystalSpawn {
                        origin: crystal.position,
                        max_range: crystal.range,
                    });
                }

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
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<&mut ArcaneCrystal>,
    missiles: Query<(Entity, &Transform, &MagicMissile), Without<CrystalSpawn>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    for (missile_entity, missile_transform, _missile) in &missiles {
        for mut crystal in &mut crystals {
            let distance = missile_transform.translation.distance(crystal.position);

            if distance <= crystal.collision_radius {
                // Absorb the missile
                commands.entity(missile_entity).despawn();
                crystal.trigger_pulse();
                crystal.remembered_spell = Some(RememberedSpell::MagicMissile);
                crystal.auto_cast_timer = 0.0;

                // Emit mini missiles at random enemy targets (not defenders)
                let targets = find_random_enemies_in_range(
                    crystal.position,
                    crystal.range,
                    MINI_MISSILE_COUNT,
                    &enemies,
                );

                let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

                for (target_entity, target_pos) in &targets {
                    let direction = (*target_pos - crystal.position).normalize();
                    let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                    let initial_velocity = direction * speed;

                    let mut rng = rand::thread_rng();
                    let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

                    spawn_crystal_mini_missile(
                        &mut commands,
                        &visual_assets,
                        crystal.position,
                        crystal.range,
                        initial_velocity,
                        wobble_offset,
                        Some(*target_entity),
                        mini_radius,
                    );
                }

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
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal)>,
    bolts: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningBolt,
    >,
    groups: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningGroup,
    >,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    // Check if any bolt's last_hit_position matches a crystal position
    // (chain lightning system sets crystal as a bounce target, so the bolt
    // will have the crystal's position as last_hit_position after bouncing to it)
    for (crystal_entity, mut crystal) in &mut crystals {
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

            crystal.trigger_pulse();
            crystal.remembered_spell = Some(RememberedSpell::ChainLightning);
            crystal.auto_cast_timer = 0.0;

            // Emit arcs to random targets
            let enemies = find_random_targets_in_range(
                crystal.position,
                crystal.range,
                LIGHTNING_ARC_COUNT,
                &targets,
            );

            let damage = bolt.current_damage * DAMAGE_SCALE;

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
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<&mut ArcaneCrystal>,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    // Collect crystal data to avoid borrow conflicts
    let crystal_data: Vec<_> = crystals
        .iter()
        .map(|c| {
            (
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.auto_cast_timer,
                c.auto_disintegrate_beam,
            )
        })
        .collect();

    for (idx, (position, range, empowerment, remembered, timer, auto_beam)) in
        crystal_data.into_iter().enumerate()
    {
        let Some(remembered) = remembered else {
            // No remembered spell — clean up any lingering auto-disintegrate beam
            if let Some((beam_entity, _)) = auto_beam {
                commands.entity(beam_entity).try_despawn();
                if let Some(mut crystal) = crystals.iter_mut().nth(idx) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            continue;
        };

        // === Special case: Disintegrate = constant single beam ===
        if remembered == RememberedSpell::Disintegrate {
            handle_auto_disintegrate(
                idx,
                position,
                range,
                empowerment,
                auto_beam,
                &mut commands,
                &visual_assets,
                &mut crystal_beams,
                &targets,
                &mut crystals,
            );
            continue;
        }

        // === Timer-based auto-cast for all other spells ===

        // Clean up any lingering auto-disintegrate beam if spell changed
        if let Some((beam_entity, _)) = auto_beam {
            commands.entity(beam_entity).try_despawn();
            if let Some(mut crystal) = crystals.iter_mut().nth(idx) {
                crystal.auto_disintegrate_beam = None;
            }
        }

        let new_timer = timer + delta;
        let interval = remembered.auto_cast_interval();

        if new_timer >= interval {
            // Reset timer
            if let Some(mut crystal) = crystals.iter_mut().nth(idx) {
                crystal.auto_cast_timer = 0.0;
                crystal.trigger_pulse();
            }

            match remembered {
                RememberedSpell::MagicMissile => {
                    auto_cast_magic_missiles(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &visual_assets,
                        &enemies,
                    );
                }
                RememberedSpell::Fireball => {
                    auto_cast_fireballs(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::ChainLightning => {
                    auto_cast_chain_lightning(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &mut health_query,
                    );
                }
                RememberedSpell::Meteor => {
                    auto_cast_meteors(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::FingerOfDeath => {
                    auto_cast_fod_beams(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::Disintegrate => unreachable!(),
            }
        } else {
            // Just advance timer
            if let Some(mut crystal) = crystals.iter_mut().nth(idx) {
                crystal.auto_cast_timer = new_timer;
            }
        }
    }
}

/// Manages a single persistent auto-disintegrate beam.
///
/// Instead of despawning/respawning the beam every frame (which resets time_alive
/// and breaks the growth animation + damage), we update the beam's fields in-place.
/// A new beam is only spawned when the old target dies/leaves range.
#[allow(clippy::too_many_arguments)]
fn handle_auto_disintegrate(
    crystal_idx: usize,
    position: Vec3,
    range: f32,
    empowerment: f32,
    auto_beam: Option<(Entity, Entity)>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_beams: &mut Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
    crystals: &mut Query<&mut ArcaneCrystal>,
) {
    if let Some((beam_entity, target_entity)) = auto_beam {
        // Check if beam entity still exists
        if crystal_beams.get(beam_entity).is_err() {
            if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                crystal.auto_disintegrate_beam = None;
            }
            // Fall through to spawn a new beam below
        } else if let Ok((_, target_transform)) = targets.get(target_entity) {
            // Target alive — check range
            let dist = Vec3::new(
                position.x - target_transform.translation.x,
                0.0,
                position.z - target_transform.translation.z,
            )
            .length();
            if dist <= range {
                // Target still valid — update beam to track it in-place
                if let Ok((_, mut beam)) = crystal_beams.get_mut(beam_entity) {
                    let direction = (target_transform.translation - position).normalize();
                    let length = position.distance(target_transform.translation).min(range);
                    beam.origin = position;
                    beam.direction = direction;
                    beam.length = length;
                }
                return;
            }
            // Target out of range — despawn old beam and find new target
            commands.entity(beam_entity).try_despawn();
            let new_targets = find_random_targets_in_range(position, range, 1, targets);
            if let Some((new_target, new_pos)) = new_targets.first() {
                let new_beam = spawn_crystal_disintegrate_beam(
                    commands,
                    assets,
                    position,
                    *new_pos,
                    range,
                    empowerment,
                );
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = Some((new_beam, *new_target));
                }
            } else if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                crystal.auto_disintegrate_beam = None;
            }
            return;
        } else {
            // Target dead — despawn old beam and find new target
            commands.entity(beam_entity).try_despawn();
            let new_targets = find_random_targets_in_range(position, range, 1, targets);
            if let Some((new_target, new_pos)) = new_targets.first() {
                let new_beam = spawn_crystal_disintegrate_beam(
                    commands,
                    assets,
                    position,
                    *new_pos,
                    range,
                    empowerment,
                );
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = Some((new_beam, *new_target));
                }
            } else if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                crystal.auto_disintegrate_beam = None;
            }
            return;
        }
    }

    // No beam exists — try to spawn one
    let new_targets = find_random_targets_in_range(position, range, 1, targets);
    if let Some((target_entity, target_pos)) = new_targets.first() {
        let beam_entity = spawn_crystal_disintegrate_beam(
            commands,
            assets,
            position,
            *target_pos,
            range,
            empowerment,
        );
        if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
            crystal.auto_disintegrate_beam = Some((beam_entity, *target_entity));
        }
    }
}

/// Spawns a DisintegrateBeam with crystal damage scaling and CrystalSpawn marker.
fn spawn_crystal_disintegrate_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    max_range: f32,
    empowerment: f32,
) -> Entity {
    let direction = (target - origin).normalize();
    let length = origin.distance(target).min(max_range);
    let beam_entity = disintegrate_systems::spawn_beam_with_damage(
        commands,
        assets,
        origin,
        direction,
        length,
        empowerment,
        disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
    );
    commands
        .entity(beam_entity)
        .insert(CrystalSpawn { origin, max_range });
    beam_entity
}

/// Auto-casts mini magic missiles at random enemies (not defenders).
fn auto_cast_magic_missiles(
    position: Vec3,
    range: f32,
    _empowerment: f32,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    enemies: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    let targets = find_random_enemies_in_range(position, range, MINI_MISSILE_COUNT, enemies);
    let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

    for (target_entity, target_pos) in &targets {
        let direction = (*target_pos - position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;

        let mut rng = rand::thread_rng();
        let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            commands,
            assets,
            position,
            range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
        );
    }
}

/// Auto-casts mini fireballs at random enemies.
fn auto_cast_fireballs(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, MINI_FB_COUNT, targets);
    let mini_radius = fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;

    for (_, target_pos) in &enemies {
        let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let direction = (ground_target - position).normalize();
        let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
        let velocity = direction * speed;

        let entity = fireball_systems::spawn_fireball_entity(
            commands,
            assets,
            position,
            velocity,
            fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE,
            fireball_constants::DAMAGE_TYPE,
            fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
            empowerment * DAMAGE_SCALE,
            mini_radius,
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: position,
            max_range: range,
        });
    }
}

/// Auto-casts chain lightning arcs at random enemies.
#[allow(clippy::too_many_arguments)]
fn auto_cast_chain_lightning(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    let enemies = find_random_targets_in_range(position, range, LIGHTNING_ARC_COUNT, targets);
    let damage = cl_constants::INITIAL_DAMAGE * DAMAGE_SCALE * empowerment;

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

        chain_lightning_systems::spawn_arc(commands, assets, position, *target_pos, 0, empowerment);
    }
}

/// Auto-casts mini meteors at random enemies.
fn auto_cast_meteors(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, 2, targets);
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (_, target_pos) in &enemies {
        let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
        let damage = meteor_fall_constants::METEOR_DAMAGE * DAMAGE_SCALE;
        let explosion_radius = meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE;

        let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
            commands,
            assets,
            spawn_pos,
            Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
            damage,
            explosion_radius,
            empowerment,
            mini_radius,
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: position,
            max_range: range,
        });
    }
}

/// Auto-casts Finger of Death beams at random enemies.
fn auto_cast_fod_beams(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, BEAM_COUNT, targets);
    let fod_damage_per_tick = finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE
        / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);
    for (_, target_pos) in &enemies {
        let direction = (*target_pos - position).normalize();
        let length = position.distance(*target_pos).min(range);
        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
            commands,
            assets,
            position,
            direction,
            length,
            empowerment,
            fod_damage_per_tick,
        );
        commands.entity(beam_entity).insert(CrystalSpawn {
            origin: position,
            max_range: range,
        });
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
) {
    let mut mini_missile =
        crate::game::units::wizard::spells::magic_missile::components::MagicMissile::new(
            initial_velocity,
            wobble_offset,
            target,
            DAMAGE_SCALE,
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
        },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.crystal_mini_missile.clone()),
        Transform::from_translation(crystal_position).with_scale(Vec3::splat(visual_radius)),
        OnGameplayScreen,
    ));
}
