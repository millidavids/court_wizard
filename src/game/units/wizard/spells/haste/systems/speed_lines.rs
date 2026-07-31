//! Speed-line streaks trailing behind hasted units while they move.
//!
//! Pale, short-lived quads spawned just behind a moving hasted unit, aligned
//! with its travel direction and shrinking away in a fraction of a second —
//! a faint motion cue that reads only when you look for it. Host-simulated
//! units carry the real `HasteModifier`; guest-rendered ghosts carry the
//! snapshot-mirrored `RemoteHasteEffect` (ghost velocities are host-
//! authoritative via the snapshot, so direction works there too). No
//! `GhostEntity` exclusion — visual systems run on ghosts by convention.

use bevy::prelude::*;

use crate::game::components::{OnGameplayScreen, Velocity};
use crate::game::units::components::{Corpse, HasteModifier, Hitbox, RemoteHasteEffect};
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::vfx::unit_motes::unit_effect_tick;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Seconds between streak spawns per hasted unit. Phase-offset per entity so
/// a hasted group funnelling through a chokepoint doesn't emit in lockstep,
/// and so the spawns spread across frames instead of bursting the whole army
/// onto one frame.
const SPEED_LINE_INTERVAL: f32 = 0.2;
/// Minimum horizontal speed (world units/second) before streaks appear —
/// stationary hasted units show nothing.
const SPEED_LINE_MIN_SPEED: f32 = 15.0;
/// Streak lifetime in seconds.
const SPEED_LINE_LIFETIME: f32 = 0.25;
/// Streak length along the travel direction (world units). Sized against
/// the 96-unit-wide infantry sprite so it reads at battle zoom.
const SPEED_LINE_LENGTH: f32 = 30.0;
/// Streak width across the travel direction (world units).
const SPEED_LINE_WIDTH: f32 = 3.5;

/// A single ground-level streak quad. Ticked by `update_haste_speed_lines`.
#[derive(Component)]
pub struct HasteSpeedLine {
    pub time_alive: f32,
    pub lifetime: f32,
    pub base_length: f32,
}

/// Spawns streaks behind moving hasted units on a fixed global tick.
#[allow(clippy::type_complexity)]
pub fn emit_haste_speed_line_vfx(
    mut commands: Commands,
    hasted_units: Query<
        (Entity, &Transform, &Hitbox, &Velocity),
        (
            Or<(With<HasteModifier>, With<RemoteHasteEffect>)>,
            Without<Corpse>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, transform, hitbox, velocity) in &hasted_units {
        if unit_effect_tick(entity, t, dt, SPEED_LINE_INTERVAL).is_none() {
            continue;
        }

        let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if speed < SPEED_LINE_MIN_SPEED {
            continue;
        }

        let dir_x = velocity.x / speed;
        let dir_z = velocity.z / speed;
        let pos = transform.translation;

        // Just behind the unit, low near the legs.
        let spawn_pos = Vec3::new(
            pos.x - dir_x * hitbox.radius * 0.9,
            pos.y - hitbox.height * 0.15,
            pos.z - dir_z * hitbox.radius * 0.9,
        );

        // Scale applies on the quad's local axes (X = length, Y = width),
        // UPWARD_ROTATION lays it flat, then the Y-rotation aligns local X
        // with the horizontal travel direction.
        let angle = (-dir_z).atan2(dir_x);

        commands.spawn((
            HasteSpeedLine {
                time_alive: 0.0,
                lifetime: SPEED_LINE_LIFETIME,
                base_length: SPEED_LINE_LENGTH,
            },
            Mesh3d(visual_assets.particle_quad.clone()),
            MeshMaterial3d(visual_assets.haste_speed_line.clone()),
            Transform::from_translation(spawn_pos)
                .with_rotation(Quat::from_rotation_y(angle) * UPWARD_ROTATION)
                .with_scale(Vec3::new(SPEED_LINE_LENGTH, SPEED_LINE_WIDTH, 1.0)),
            OnGameplayScreen,
        ));
    }
}

/// Shrinks streaks away over their lifetime and despawns them.
pub fn update_haste_speed_lines(
    mut commands: Commands,
    mut lines: Query<(Entity, &mut HasteSpeedLine, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut line, mut transform) in &mut lines {
        line.time_alive += dt;
        if line.time_alive >= line.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Shrink the length toward zero so the streak fades without any
        // per-entity material mutation.
        let remaining = 1.0 - line.time_alive / line.lifetime;
        transform.scale.x = line.base_length * remaining;
    }
}
