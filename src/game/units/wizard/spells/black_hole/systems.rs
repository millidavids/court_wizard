//! Black Hole spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{BlackHole, UnitInBlackHole};
use super::constants::*;
use crate::game::components::{Acceleration, OnGameplayScreen};
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::components::{CastingState, Mana, PrimedSpell, Wizard};

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
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
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

/// Spawns a black hole entity with visual mesh.
fn spawn_black_hole(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) {
    let max_radius = MAX_RADIUS * empowerment;
    let spawn_pos = Vec3::new(position.x, BLACK_HOLE_HEIGHT, position.z);

    // Create sphere mesh
    let sphere = Sphere::new(max_radius);

    // Create dark material with purple emissive glow
    let material = StandardMaterial {
        base_color: BLACK_HOLE_COLOR,
        emissive: BLACK_HOLE_EMISSIVE.into(),
        unlit: false, // Let it interact with lighting for depth
        ..default()
    };

    commands.spawn((
        BlackHole::new(spawn_pos, max_radius, empowerment),
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
        OnGameplayScreen,
    ));
}

/// Handles Black Hole spell casting.
///
/// Left-click starts cast. After cast completes, spawns black hole at cursor position.
/// Black hole persists for LIFETIME seconds as a fixture.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_black_hole_casting(
    time: Res<Time>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            &Transform,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
        ),
        With<Wizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // State machine
    match *casting_state {
        CastingState::Resting => {
            // Check mana before starting cast
            if mana.can_afford(MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Advance cast time
            casting_state.advance(time.delta_secs());

            // Check if cast complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Get cursor position and spawn black hole
                if let Some(cursor_pos) = get_cursor_world_position(&camera_query, &window_query) {
                    let wizard_pos = wizard_transform.translation;
                    let clamped_pos =
                        clamp_to_spell_range(cursor_pos, wizard_pos, wizard.spell_range);

                    // Consume mana and spawn black hole
                    if mana.consume(MANA_COST) {
                        spawn_black_hole(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            clamped_pos,
                            primed_spell.empowerment,
                        );
                    }
                }

                // Return to resting state
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            // Should never be in channeling state for this spell
            casting_state.cancel();
        }
    }
}

/// Applies gravitational forces to all living units near black holes.
///
/// Gravitational pull increases over time (0-5 seconds) AND with proximity.
/// Uses inverse square law so units cannot escape when close enough.
/// Forces are applied to acceleration, which can override normal movement limits.
/// This system runs in MovementCalculationSet, after unit-specific movement calculations.
pub(super) fn apply_gravitational_forces(
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (&Transform, &mut Acceleration),
        (
            With<Team>,
            Without<Wizard>,
            Without<BlackHole>,
            Without<Corpse>,
        ),
    >,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for mut black_hole in black_holes.iter_mut() {
        // Update black hole timers
        black_hole.update_timers(delta);

        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (transform, mut acceleration) in units.iter_mut() {
            let unit_pos = transform.translation;
            let to_black_hole = bh_pos - unit_pos;
            let distance = to_black_hole.length();

            // Only apply forces within gravity range and avoid division by zero
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                // This will be integrated into velocity and applied to transform by apply_unit_movement
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies gravitational forces to corpses and despawns them if they touch the black hole.
///
/// Corpses are pulled by the same gravitational forces as living units.
/// When a corpse intersects the black hole sphere, it is despawned.
pub(super) fn apply_corpse_gravity_and_despawn(
    mut commands: Commands,
    mut black_holes: Query<&BlackHole>,
    mut corpses: Query<(Entity, &Transform, &mut Acceleration), With<Corpse>>,
) {
    for black_hole in black_holes.iter_mut() {
        let gravity_strength = black_hole.gravitational_strength();
        let bh_pos = black_hole.position;

        for (entity, transform, mut acceleration) in corpses.iter_mut() {
            let corpse_pos = transform.translation;
            let to_black_hole = bh_pos - corpse_pos;
            let distance = to_black_hole.length();

            // Check if corpse intersects the black hole sphere - if so, despawn it
            if black_hole.contains_point(corpse_pos) {
                commands.entity(entity).despawn();
                continue;
            }

            // Apply gravitational forces within range
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                // Use inverse square law for realistic gravity that grows stronger with proximity
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(MAX_FORCE_CLAMP);
                let direction = to_black_hole.normalize();

                // Apply gravitational force to acceleration
                let force = direction * pull_strength;
                acceleration.add_force(force);
            }
        }
    }
}

/// Applies damage to units touching the black hole sphere.
///
/// Damage increases over time for units that remain in contact.
pub(super) fn apply_black_hole_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Option<&mut UnitInBlackHole>,
        ),
        Without<Wizard>,
    >,
) {
    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.should_damage() {
            continue;
        }

        for (entity, transform, mut health, mut temp_hp, tracking) in units.iter_mut() {
            let unit_pos = transform.translation;

            if black_hole.contains_point(unit_pos) {
                // Track or update time inside
                let damage_multiplier = if let Some(mut tracker) = tracking {
                    tracker.time_inside += time.delta_secs();
                    tracker.damage_multiplier()
                } else {
                    commands.entity(entity).insert(UnitInBlackHole::new());
                    1.0 // First tick, no multiplier yet
                };

                // Apply scaled damage
                let total_damage = black_hole.damage_per_tick() * damage_multiplier;
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), total_damage);
            }
        }

        black_hole.reset_damage_timer();
    }
}

/// Removes tracking component when units leave the black hole.
pub(super) fn remove_units_from_black_hole(
    mut commands: Commands,
    black_holes: Query<&BlackHole>,
    units: Query<(Entity, &Transform), With<UnitInBlackHole>>,
) {
    for (entity, transform) in units.iter() {
        let unit_pos = transform.translation;
        let mut is_in_any_black_hole = false;

        for black_hole in black_holes.iter() {
            if black_hole.contains_point(unit_pos) {
                is_in_any_black_hole = true;
                break;
            }
        }

        if !is_in_any_black_hole {
            commands.entity(entity).remove::<UnitInBlackHole>();
        }
    }
}

/// Updates black hole visual scale to match growth animation and adds vibration effect.
pub(super) fn update_black_hole_visuals(mut black_holes: Query<(&BlackHole, &mut Transform)>) {
    for (black_hole, mut transform) in black_holes.iter_mut() {
        let growth_factor = (black_hole.time_alive / GROWTH_TIME).min(1.0);

        // Add vibration using sine waves on different axes
        let t = black_hole.time_alive * VIBRATION_FREQUENCY;
        let vibration = Vec3::new(
            (t * 1.0).sin() * VIBRATION_AMPLITUDE,
            (t * 1.7).sin() * VIBRATION_AMPLITUDE,
            (t * 2.3).sin() * VIBRATION_AMPLITUDE,
        );

        transform.scale = Vec3::splat(growth_factor);
        transform.translation = black_hole.position + vibration;
    }
}

/// Despawns black holes when they expire after LIFETIME seconds.
pub(super) fn despawn_expired_black_holes(
    mut commands: Commands,
    black_holes: Query<(Entity, &BlackHole)>,
) {
    for (entity, black_hole) in black_holes.iter() {
        if black_hole.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}
