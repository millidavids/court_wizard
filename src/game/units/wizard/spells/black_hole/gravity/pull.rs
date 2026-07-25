use super::super::components::BlackHole;
use super::super::constants::*;
use crate::game::components::Acceleration;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::Wizard;
use bevy::prelude::*;

pub(crate) fn apply_gravitational_forces(
    // No `Without<GhostSpellEffect>` filter — the host runs this system, and
    // its `Acceleration` writes only land on host-owned units. When the guest
    // casts a black hole, the host has only a `GhostSpellEffect`-tagged copy
    // of it; excluding ghosts here would mean nobody applies gravity for
    // guest-cast black holes (the guest's own `apply_gravitational_forces`
    // doesn't run — it lives in `MovementCalculationSet` which is host-only).
    mut black_holes: Query<&mut BlackHole>,
    mut units: Query<
        (&Transform, &mut Acceleration),
        (
            With<Team>,
            Without<Wizard>,
            Without<BlackHole>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
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
pub(crate) fn apply_corpse_gravity_and_despawn(
    mut commands: Commands,
    // See `apply_gravitational_forces`: no `GhostSpellEffect` filter so
    // guest-cast black holes (ghost copies on the host) still apply gravity
    // to host-side corpses.
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
                commands.entity(entity).try_despawn();
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
