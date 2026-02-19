//! Arcane Crystal spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::wizard::components::{
    CastingState, Mana, PrimedSpell, SpellCaster, Wizard,
};
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::chain_lightning::constants as cl_constants;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::{Fireball, FireballExplosion};
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::{
    disintegrate_constants, finger_of_death_constants, fireball_constants, magic_missile_constants,
    meteor_fall_constants,
};

// ===== Helper Functions =====

/// Gets cursor position projected onto Y=0 plane.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    if ray.direction.y.abs() < 0.0001 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a position to be within the wizard's spell range.
fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();

    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}

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

/// Handles Arcane Crystal casting with left-click.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_arcane_crystal_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<Wizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster, With<Wizard>>,
    mut indicator_query: Query<&mut ArcaneCrystalCircleIndicator>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - cancel cast
    if mouse_left_released.read().next().is_some() {
        if let Ok(caster) = caster_query.single() {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    let Some(mut cursor_world_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
    };

    let wizard_pos = wizard_transform.translation;
    cursor_world_pos = clamp_to_spell_range(cursor_world_pos, wizard_pos, wizard.spell_range);

    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(MANA_COST) {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    cursor_world_pos,
                    primed_spell.empowerment,
                );

                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));

                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            if let Ok(caster) = caster_query.single()
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    if let Ok(caster) = caster_query.single()
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            spawn_crystal(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                indicator.position,
                                primed_spell.empowerment,
                            );
                        }
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
                } else {
                    if let Ok(caster) = caster_query.single()
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.single() {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }
}

/// Spawns the visual circle indicator during casting.
fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = CRYSTAL_RANGE * empowerment;

    commands
        .spawn((
            Mesh3d(meshes.add(Circle::new(radius))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: CIRCLE_COLOR,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(Vec3::new(position.x, CIRCLE_Y_POSITION, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ArcaneCrystalCircleIndicator::new(position),
            OnGameplayScreen,
        ))
        .id()
}

/// Updates circle indicator visuals during casting.
pub(super) fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut ArcaneCrystalCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();

        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);

        transform.translation.x = indicator.position.x;
        transform.translation.y = CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Spawns the crystal entity with visual mesh and range indicator.
fn spawn_crystal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) {
    let range = CRYSTAL_RANGE * empowerment;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;
    let duration = CRYSTAL_DURATION * empowerment;
    let height = CRYSTAL_HEIGHT * empowerment;

    let crystal_pos = Vec3::new(position.x, height / 2.0, position.z);

    // Spawn crystal mesh (vertically-stretched sphere to approximate crystal shape)
    let sphere = Sphere::new(height / 3.0);

    let crystal_entity = commands
        .spawn((
            ArcaneCrystal::new(crystal_pos, range, duration, collision_radius, empowerment),
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: CRYSTAL_COLOR,
                emissive: CRYSTAL_EMISSIVE.into(),
                unlit: false,
                ..default()
            })),
            Transform::from_translation(crystal_pos).with_scale(Vec3::new(0.7, 1.5, 0.7)), // Vertically stretched
            OnGameplayScreen,
        ))
        .id();

    // Spawn range indicator circle
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(range))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RANGE_INDICATOR_COLOR,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        Transform::from_translation(Vec3::new(position.x, RANGE_INDICATOR_Y, position.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
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

        // Rotation
        transform.rotate_y(ROTATION_SPEED * delta);

        // Pulse animation
        if crystal.pulse_timer > 0.0 {
            crystal.pulse_timer -= delta;
            let pulse_progress = crystal.pulse_timer / PULSE_DURATION;
            let scale_factor = 1.0 + (PULSE_SCALE - 1.0) * pulse_progress;
            transform.scale = Vec3::new(0.7 * scale_factor, 1.5 * scale_factor, 0.7 * scale_factor);
        } else {
            transform.scale = Vec3::new(0.7, 1.5, 0.7);
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

                for (_, target_pos) in &enemies {
                    // Aim at ground level (Y=0) at the target's XZ position
                    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
                    let direction = (ground_target - crystal.position).normalize();
                    let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                    let velocity = direction * speed;

                    let damage = fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE;
                    let explosion_radius = fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE;
                    let collision_radius =
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE;

                    let sphere = Sphere::new(
                        fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5,
                    );

                    commands.spawn((
                        Fireball::new(
                            velocity,
                            damage,
                            fireball_constants::DAMAGE_TYPE,
                            explosion_radius,
                            collision_radius,
                            explosion.empowerment * DAMAGE_SCALE,
                        ),
                        CrystalSpawn {
                            origin: crystal.position,
                            max_range: crystal.range,
                        },
                        Mesh3d(meshes.add(sphere)),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.5, 0.0),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(crystal.position),
                        OnGameplayScreen,
                    ));
                }
            }
        }
    }
}

// ===== Disintegrate Beam Absorption =====

/// Detects disintegrate and finger of death beams hitting crystals.
///
/// Disintegrate: Maintains 5 persistent beams that update each frame while channeling.
/// Finger of Death: One-shot burst of 5 beams when the damage beam strikes.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_beam_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: Query<&mut ArcaneCrystal>,
    disintegrate_beams: Query<&DisintegrateBeam>,
    fod_beams: Query<(Entity, &FingerOfDeathBeam)>,
    mut crystal_beams: Query<(Entity, &mut CrystalBeam)>,
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
                        // Target still valid — update beam direction to track it
                        if let Ok((_, mut beam)) = crystal_beams.get_mut(*beam_entity) {
                            beam.retarget(
                                crystal.position,
                                target_transform.translation,
                                crystal.range,
                            );
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
                        if let Ok((_, mut beam)) = crystal_beams.get_mut(beam_entity) {
                            beam.retarget(crystal.position, *new_pos, crystal.range);
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
                    let beam_entity = spawn_crystal_beam(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        crystal.position,
                        *target_pos,
                        crystal.range,
                        disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
                        disintegrate_constants::DAMAGE_INTERVAL,
                        disintegrate_constants::BEAM_WIDTH * SIZE_SCALE,
                        crystal.empowerment,
                        disintegrate_constants::BEAM_COLOR,
                        true,
                    );
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

        // === Finger of Death: one-shot burst of 5 beams ===
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
                for (_, target_pos) in &enemies {
                    spawn_crystal_beam(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        crystal.position,
                        *target_pos,
                        crystal.range,
                        finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE
                            / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL),
                        disintegrate_constants::DAMAGE_INTERVAL,
                        finger_of_death_constants::BEAM_WIDTH * SIZE_SCALE,
                        crystal.empowerment,
                        finger_of_death_constants::BEAM_COLOR_FIRED,
                        false,
                    );
                }
            }
        }
    }
}

/// Spawns a crystal beam entity toward a target position. Returns the entity ID.
#[allow(clippy::too_many_arguments)]
fn spawn_crystal_beam(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    target: Vec3,
    max_range: f32,
    damage_per_tick: f32,
    damage_interval: f32,
    beam_width: f32,
    empowerment: f32,
    color: Color,
    persistent: bool,
) -> Entity {
    let direction = (target - origin).normalize();
    let distance_to_target = origin.distance(target);
    let length = distance_to_target.min(max_range);

    let midpoint = origin + direction * (length / 2.0);
    let rectangle = Rectangle::new(beam_width, beam_width);

    commands
        .spawn((
            CrystalBeam {
                origin,
                direction,
                length,
                damage_per_tick,
                damage_interval,
                time_since_damage: 0.0,
                time_alive: 0.0,
                beam_duration: BEAM_DURATION,
                beam_width,
                empowerment,
                persistent,
            },
            Mesh3d(meshes.add(rectangle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id()
}

// ===== Crystal Beam Systems =====

/// Updates crystal beam visuals, damage, and lifetime.
pub(super) fn update_crystal_beams(
    time: Res<Time>,
    mut commands: Commands,
    mut beams: Query<(Entity, &mut CrystalBeam, &mut Transform)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<CrystalBeam>,
    >,
) {
    let delta = time.delta_secs();

    for (beam_entity, mut beam, mut transform) in &mut beams {
        beam.time_alive += delta;
        beam.time_since_damage += delta;

        // Despawn after duration (persistent beams are managed externally)
        if !beam.persistent && beam.time_alive >= beam.beam_duration {
            commands.entity(beam_entity).despawn();
            continue;
        }

        // Update visual
        let current_len = beam.current_length();
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        let scale_y = current_len / beam.beam_width;
        transform.scale = Vec3::new(1.0, scale_y, 1.0);

        // Apply damage
        if beam.time_since_damage >= beam.damage_interval {
            beam.time_since_damage = 0.0;

            for (entity, target_transform, mut health, mut temp_hp) in &mut targets {
                if beam.contains_point(target_transform.translation) {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        beam.damage_per_tick * beam.empowerment,
                        DamageType::Force,
                    );
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: Query<&mut ArcaneCrystal>,
    meteors: Query<(Entity, &Transform, &MeteorProjectile)>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
) {
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
                    // Launch a mini meteor from above the target
                    let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);

                    let damage = meteor.damage * DAMAGE_SCALE;
                    let explosion_radius = meteor.explosion_radius * SIZE_SCALE;

                    let sphere =
                        Sphere::new(meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE);

                    commands.spawn((
                        MeteorProjectile::new(
                            Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                            damage,
                            explosion_radius,
                            meteor.empowerment,
                        ),
                        CrystalSpawn {
                            origin: crystal.position,
                            max_range: crystal.range,
                        },
                        Mesh3d(meshes.add(sphere)),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: meteor_fall_constants::METEOR_COLOR,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(spawn_pos),
                        OnGameplayScreen,
                    ));
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

                for (target_entity, target_pos) in &targets {
                    let direction = (*target_pos - crystal.position).normalize();
                    let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                    let initial_velocity = direction * speed;

                    let mut rng = rand::thread_rng();
                    let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

                    let circle =
                        Circle::new(magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE);

                    let mut mini_missile = MagicMissile::new(
                        initial_velocity,
                        wobble_offset,
                        Some(*target_entity),
                        DAMAGE_SCALE,
                    );
                    mini_missile.time_alive = MINI_MISSILE_HOMING_ADVANCE;

                    commands.spawn((
                        mini_missile,
                        CrystalSpawn {
                            origin: crystal.position,
                            max_range: crystal.range,
                        },
                        Mesh3d(meshes.add(circle)),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.8, 0.3, 0.9),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(crystal.position),
                        OnGameplayScreen,
                    ));
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal)>,
    bolts: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningBolt,
    >,
    groups: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningGroup,
    >,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
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
                if let Ok((mut health, mut temp_hp)) = health_query.get_mut(*target_entity) {
                    apply_spell_damage(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Electric,
                    );
                }

                // Spawn arc visual
                spawn_crystal_arc(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    crystal.position,
                    *target_pos,
                );
            }

            break; // Only process once per crystal per frame
        }
    }
}

/// Spawns a visual lightning arc from crystal to target.
fn spawn_crystal_arc(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    start: Vec3,
    end: Vec3,
) {
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    let length = start.distance(end);
    let arc_width = cl_constants::MIN_ARC_WIDTH;

    let rectangle = Rectangle::new(arc_width, arc_width);

    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    commands.spawn((
        CrystalLightningArc {
            lifetime: cl_constants::ARC_LIFETIME,
            time_alive: 0.0,
        },
        Mesh3d(meshes.add(rectangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: MINI_LIGHTNING_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(1.0, length / arc_width, 1.0)),
        OnGameplayScreen,
    ));
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: Query<&mut ArcaneCrystal>,
    crystal_beams: Query<(Entity, &mut CrystalBeam)>,
    targets: Query<(Entity, &Transform), Without<Corpse>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
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
                &mut meshes,
                &mut materials,
                &crystal_beams,
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
                        &mut meshes,
                        &mut materials,
                        &enemies,
                    );
                }
                RememberedSpell::Fireball => {
                    auto_cast_fireballs(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &targets,
                    );
                }
                RememberedSpell::ChainLightning => {
                    auto_cast_chain_lightning(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
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
                        &mut meshes,
                        &mut materials,
                        &targets,
                    );
                }
                RememberedSpell::FingerOfDeath => {
                    auto_cast_fod_beams(
                        position,
                        range,
                        empowerment,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
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
#[allow(clippy::too_many_arguments)]
fn handle_auto_disintegrate(
    crystal_idx: usize,
    position: Vec3,
    range: f32,
    empowerment: f32,
    auto_beam: Option<(Entity, Entity)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    crystal_beams: &Query<(Entity, &mut CrystalBeam)>,
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
                // Target still valid — despawn old beam, respawn aimed at current position
                commands.entity(beam_entity).try_despawn();
                let new_beam = spawn_crystal_beam(
                    commands,
                    meshes,
                    materials,
                    position,
                    target_transform.translation,
                    range,
                    disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
                    disintegrate_constants::DAMAGE_INTERVAL,
                    disintegrate_constants::BEAM_WIDTH * SIZE_SCALE,
                    empowerment,
                    disintegrate_constants::BEAM_COLOR,
                    true,
                );
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = Some((new_beam, target_entity));
                }
                return;
            }
            // Target out of range — find new target
            commands.entity(beam_entity).try_despawn();
            let new_targets = find_random_targets_in_range(position, range, 1, targets);
            if let Some((new_target, new_pos)) = new_targets.first() {
                let new_beam = spawn_crystal_beam(
                    commands,
                    meshes,
                    materials,
                    position,
                    *new_pos,
                    range,
                    disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
                    disintegrate_constants::DAMAGE_INTERVAL,
                    disintegrate_constants::BEAM_WIDTH * SIZE_SCALE,
                    empowerment,
                    disintegrate_constants::BEAM_COLOR,
                    true,
                );
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = Some((new_beam, *new_target));
                }
            } else {
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            return;
        } else {
            // Target dead — find new target
            let new_targets = find_random_targets_in_range(position, range, 1, targets);
            if let Some((new_target, new_pos)) = new_targets.first() {
                commands.entity(beam_entity).try_despawn();
                let new_beam = spawn_crystal_beam(
                    commands,
                    meshes,
                    materials,
                    position,
                    *new_pos,
                    range,
                    disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
                    disintegrate_constants::DAMAGE_INTERVAL,
                    disintegrate_constants::BEAM_WIDTH * SIZE_SCALE,
                    empowerment,
                    disintegrate_constants::BEAM_COLOR,
                    true,
                );
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = Some((new_beam, *new_target));
                }
            } else {
                commands.entity(beam_entity).try_despawn();
                if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            return;
        }
    }

    // No beam exists — try to spawn one
    let new_targets = find_random_targets_in_range(position, range, 1, targets);
    if let Some((target_entity, target_pos)) = new_targets.first() {
        let beam_entity = spawn_crystal_beam(
            commands,
            meshes,
            materials,
            position,
            *target_pos,
            range,
            disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
            disintegrate_constants::DAMAGE_INTERVAL,
            disintegrate_constants::BEAM_WIDTH * SIZE_SCALE,
            empowerment,
            disintegrate_constants::BEAM_COLOR,
            true,
        );
        if let Some(mut crystal) = crystals.iter_mut().nth(crystal_idx) {
            crystal.auto_disintegrate_beam = Some((beam_entity, *target_entity));
        }
    }
}

/// Auto-casts mini magic missiles at random enemies (not defenders).
fn auto_cast_magic_missiles(
    position: Vec3,
    range: f32,
    _empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    enemies: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    let targets = find_random_enemies_in_range(position, range, MINI_MISSILE_COUNT, enemies);
    for (target_entity, target_pos) in &targets {
        let direction = (*target_pos - position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;

        let mut rng = rand::thread_rng();
        let wobble_offset = rng.gen_range(0.0..std::f32::consts::TAU);

        let circle = Circle::new(magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE);

        let mut mini_missile = MagicMissile::new(
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            DAMAGE_SCALE,
        );
        mini_missile.time_alive = MINI_MISSILE_HOMING_ADVANCE;

        commands.spawn((
            mini_missile,
            CrystalSpawn {
                origin: position,
                max_range: range,
            },
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.3, 0.9),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(position),
            OnGameplayScreen,
        ));
    }
}

/// Auto-casts mini fireballs at random enemies.
fn auto_cast_fireballs(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, MINI_FB_COUNT, targets);
    for (_, target_pos) in &enemies {
        let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let direction = (ground_target - position).normalize();
        let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
        let velocity = direction * speed;

        let damage = fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE;
        let explosion_radius = fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE;
        let collision_radius = fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE;

        let sphere =
            Sphere::new(fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5);

        commands.spawn((
            Fireball::new(
                velocity,
                damage,
                fireball_constants::DAMAGE_TYPE,
                explosion_radius,
                collision_radius,
                empowerment * DAMAGE_SCALE,
            ),
            CrystalSpawn {
                origin: position,
                max_range: range,
            },
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.5, 0.0),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(position),
            OnGameplayScreen,
        ));
    }
}

/// Auto-casts chain lightning arcs at random enemies.
#[allow(clippy::too_many_arguments)]
fn auto_cast_chain_lightning(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
    health_query: &mut Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
) {
    let enemies = find_random_targets_in_range(position, range, LIGHTNING_ARC_COUNT, targets);
    let damage = cl_constants::INITIAL_DAMAGE * DAMAGE_SCALE * empowerment;

    for (target_entity, target_pos) in &enemies {
        if let Ok((mut health, mut temp_hp)) = health_query.get_mut(*target_entity) {
            apply_spell_damage(
                commands,
                *target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Electric,
            );
        }

        spawn_crystal_arc(commands, meshes, materials, position, *target_pos);
    }
}

/// Auto-casts mini meteors at random enemies.
fn auto_cast_meteors(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, 2, targets);
    for (_, target_pos) in &enemies {
        let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
        let damage = meteor_fall_constants::METEOR_DAMAGE * DAMAGE_SCALE;
        let explosion_radius = meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE;

        let sphere = Sphere::new(meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE);

        commands.spawn((
            MeteorProjectile::new(
                Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                damage,
                explosion_radius,
                empowerment,
            ),
            CrystalSpawn {
                origin: position,
                max_range: range,
            },
            Mesh3d(meshes.add(sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: meteor_fall_constants::METEOR_COLOR,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(spawn_pos),
            OnGameplayScreen,
        ));
    }
}

/// Auto-casts Finger of Death beams at random enemies.
fn auto_cast_fod_beams(
    position: Vec3,
    range: f32,
    empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let enemies = find_random_targets_in_range(position, range, BEAM_COUNT, targets);
    for (_, target_pos) in &enemies {
        spawn_crystal_beam(
            commands,
            meshes,
            materials,
            position,
            *target_pos,
            range,
            finger_of_death_constants::DAMAGE * BEAM_DAMAGE_SCALE
                / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL),
            disintegrate_constants::DAMAGE_INTERVAL,
            finger_of_death_constants::BEAM_WIDTH * SIZE_SCALE,
            empowerment,
            finger_of_death_constants::BEAM_COLOR_FIRED,
            false,
        );
    }
}

/// Updates crystal lightning arc visuals and despawns expired arcs.
pub(super) fn update_crystal_lightning_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut arcs: Query<(
        Entity,
        &mut CrystalLightningArc,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut arc_materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut arc, material_handle) in &mut arcs {
        arc.time_alive += time.delta_secs();
        arc.lifetime -= time.delta_secs();

        if arc.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Pulsing effect
        if let Some(material) = arc_materials.get_mut(&material_handle.0) {
            let intensity = 0.7 + 0.3 * (arc.time_alive * 20.0).sin();
            let base = MINI_LIGHTNING_COLOR.to_srgba();
            material.base_color = Color::srgba(
                base.red * intensity,
                base.green * intensity,
                base.blue * intensity,
                base.alpha,
            );
        }
    }
}
