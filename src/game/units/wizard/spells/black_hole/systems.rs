//! Black Hole spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{
    BlackHole, BlackHoleAccretionDisk, BlackHoleRing, BlackHoleSfx, UnitInBlackHole,
};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::{Acceleration, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{
    Corpse, Health, SpellDamaged, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

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

/// Spawns a black hole entity with billboard circle, torus ring, accretion disk, accretion ring, and looping sound.
pub(crate) fn spawn_black_hole(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
) {
    let max_radius = MAX_RADIUS * empowerment;
    let spawn_pos = Vec3::new(position.x, BLACK_HOLE_HEIGHT, position.z);

    // Main billboard circle (faces the camera each frame)
    let black_hole_entity = commands
        .spawn((
            BlackHole::new(spawn_pos, max_radius, empowerment),
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.black_hole_billboard.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
            NetworkedSpellEffect {
                kind: SpellEffectKind::BlackHole,
            },
            OnGameplayScreen,
        ))
        .id();

    // Billboard torus ring (white, pulsing)
    commands.spawn((
        BlackHoleRing {
            black_hole_entity,
            is_accretion: false,
        },
        Mesh3d(assets.black_hole_torus.clone()),
        MeshMaterial3d(assets.black_hole_ring.clone()),
        Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
        OnGameplayScreen,
    ));

    // Accretion disk (tilted flat circle, redshifted)
    commands.spawn((
        BlackHoleAccretionDisk { black_hole_entity },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.black_hole_accretion.clone()),
        Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
        OnGameplayScreen,
    ));

    // Accretion disk torus ring (warm-white, pulsing)
    commands.spawn((
        BlackHoleRing {
            black_hole_entity,
            is_accretion: true,
        },
        Mesh3d(assets.black_hole_torus.clone()),
        MeshMaterial3d(assets.black_hole_accretion_ring.clone()),
        Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
        OnGameplayScreen,
    ));

    // Looping sound effect attenuated by distance from wizard to black hole
    let sfx_entity = audio::play_looping_sfx_at(
        commands,
        &sfx.black_hole_persistent,
        spawn_pos,
        game_config,
    );
    commands.entity(sfx_entity).insert(BlackHoleSfx {
        black_hole_entity,
    });
}

/// Local wizard Black Hole casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_black_hole_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    visual_assets: Res<SpellVisualAssets>,
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

    let Ok((mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::BlackHole {
        return;
    }

    let cast_result = black_hole_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
    );

    if cast_result.completed {
        if let Some(pos) = cast_result.cursor_pos {
            spawn_black_hole(
                &mut commands,
                &visual_assets,
                pos,
                primed_spell.empowerment,
                &sfx,
                &game_config,
            );
        }
        mouse_state.left_consumed = true;
    }
}

/// Core Black Hole casting logic -- called by the local system.
///
/// Handles CastingState transitions, mana consumption, and cursor clamping.
/// Does NOT spawn the black hole or manage mouse_state -- those are the wrapper's job.
fn black_hole_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        cursor_pos: None,
    };

    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if let Some(cursor_pos) = input.cursor_pos {
                    let clamped_pos =
                        clamp_to_spell_range(cursor_pos, SPELL_ORIGIN, wizard.spell_range);

                    if mana.consume(MANA_COST) {
                        result.completed = true;
                        result.cursor_pos = Some(clamped_pos);
                    }
                }

                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    result
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
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
) {
    for mut black_hole in black_holes.iter_mut() {
        if !black_hole.should_damage() {
            continue;
        }

        for (entity, transform, mut health, mut temp_hp, tracking, has_spell_shield) in
            units.iter_mut()
        {
            let unit_pos = transform.translation;

            if black_hole.contains_point(unit_pos) {
                if has_spell_shield {
                    continue;
                }

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
                commands.entity(entity).insert(SpellDamaged);
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

/// Updates black hole visual scale to match growth animation, adds vibration effect,
/// billboards the circle to face the wizard, and applies pulsing.
pub(super) fn update_black_hole_visuals(
    time: Res<Time>,
    mut black_holes: Query<(&BlackHole, &mut Transform)>,
) {
    for (black_hole, mut transform) in black_holes.iter_mut() {
        let growth_factor = (black_hole.time_alive / GROWTH_TIME).min(1.0);

        // Add vibration using sine waves on different axes
        let t = black_hole.time_alive * VIBRATION_FREQUENCY;
        let vibration = Vec3::new(
            (t * 1.0).sin() * VIBRATION_AMPLITUDE,
            (t * 1.7).sin() * VIBRATION_AMPLITUDE,
            (t * 2.3).sin() * VIBRATION_AMPLITUDE,
        );

        // Pulsing scale in sync with torus rings
        let pulse =
            1.0 + (time.elapsed_secs() * RING_PULSE_FREQUENCY * std::f32::consts::TAU).sin()
                * RING_PULSE_AMPLITUDE;

        let position = black_hole.position + vibration;
        transform.scale = Vec3::splat(black_hole.max_radius * growth_factor * pulse);
        transform.translation = position;

        // Billboard: face the wizard (camera is near the wizard).
        // Circle mesh face normal is +Z. Rotate +Z to point toward the wizard.
        let toward_wizard = (SPELL_ORIGIN - position).normalize_or_zero();
        if toward_wizard.length_squared() > 0.001 {
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, toward_wizard);
        }
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

/// Updates billboard torus rings and accretion torus rings to follow their parent black holes.
pub(super) fn update_black_hole_rings(
    time: Res<Time>,
    mut rings: Query<(&BlackHoleRing, &mut Transform)>,
    black_holes: Query<(&BlackHole, &Transform), Without<BlackHoleRing>>,
) {
    for (ring, mut ring_transform) in rings.iter_mut() {
        let Ok((_black_hole, bh_transform)) = black_holes.get(ring.black_hole_entity) else {
            continue;
        };

        let bh_pos = bh_transform.translation;
        let bh_scale = bh_transform.scale.x; // uniform scale

        // Pulsing scale via sine wave
        let pulse =
            1.0 + (time.elapsed_secs() * RING_PULSE_FREQUENCY * std::f32::consts::TAU).sin()
                * RING_PULSE_AMPLITUDE;

        ring_transform.translation = bh_pos;

        if ring.is_accretion {
            // Accretion ring: already in XZ plane, just tilt slightly
            let scale = bh_scale * pulse;
            ring_transform.scale = Vec3::splat(scale);
            ring_transform.rotation = Quat::from_rotation_x(ACCRETION_TILT);
        } else {
            // Billboard ring: face the wizard, same scale as black hole.
            // Torus lies in XZ plane with axis along +Y, same as Circle.
            let scale = bh_scale * pulse;
            ring_transform.scale = Vec3::splat(scale);
            let toward_wizard = (SPELL_ORIGIN - bh_pos).normalize_or_zero();
            if toward_wizard.length_squared() > 0.001 {
                ring_transform.rotation = Quat::from_rotation_arc(Vec3::Y, toward_wizard);
            }
        }
    }
}

/// Updates the accretion disk to follow its parent black hole with a tilted rotation and pulsing.
pub(super) fn update_black_hole_accretion_disk(
    time: Res<Time>,
    mut disks: Query<(&BlackHoleAccretionDisk, &mut Transform)>,
    black_holes: Query<(&BlackHole, &Transform), Without<BlackHoleAccretionDisk>>,
) {
    for (disk, mut disk_transform) in disks.iter_mut() {
        let Ok((_black_hole, bh_transform)) = black_holes.get(disk.black_hole_entity) else {
            continue;
        };

        let bh_pos = bh_transform.translation;
        let bh_scale = bh_transform.scale.x; // uniform scale

        // Pulsing scale in sync with torus rings
        let pulse =
            1.0 + (time.elapsed_secs() * RING_PULSE_FREQUENCY * std::f32::consts::TAU).sin()
                * RING_PULSE_AMPLITUDE;

        disk_transform.translation = bh_pos;
        disk_transform.scale = Vec3::splat(bh_scale * pulse);
        // Circle face normal is +Z (vertical). Rotate -90° around X to lay flat,
        // then add the accretion tilt to match the accretion torus.
        disk_transform.rotation = Quat::from_rotation_x(
            -std::f32::consts::FRAC_PI_2 + ACCRETION_TILT,
        );
    }
}

/// Despawns orphaned black hole rings whose parent no longer exists.
pub(super) fn cleanup_black_hole_rings(
    mut commands: Commands,
    rings: Query<(Entity, &BlackHoleRing)>,
    black_holes: Query<&BlackHole>,
) {
    for (entity, ring) in rings.iter() {
        if black_holes.get(ring.black_hole_entity).is_err() {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns orphaned accretion disks whose parent no longer exists.
pub(super) fn cleanup_black_hole_accretion_disk(
    mut commands: Commands,
    disks: Query<(Entity, &BlackHoleAccretionDisk)>,
    black_holes: Query<&BlackHole>,
) {
    for (entity, disk) in disks.iter() {
        if black_holes.get(disk.black_hole_entity).is_err() {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns orphaned black hole sound effects whose parent no longer exists.
pub(super) fn cleanup_black_hole_sfx(
    mut commands: Commands,
    sfx_entities: Query<(Entity, &BlackHoleSfx)>,
    black_holes: Query<&BlackHole>,
) {
    for (entity, sfx) in sfx_entities.iter() {
        if black_holes.get(sfx.black_hole_entity).is_err() {
            commands.entity(entity).despawn();
        }
    }
}
