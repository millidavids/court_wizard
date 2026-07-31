//! Faint whitish ring at the feet of units holding temporary hit points.
//!
//! `TemporaryHitPoints` is granted by Guardian Circle, Battle Hymn's
//! Fortifying Hymn talent, and Healing Plume's Overflow talent — the ring is
//! source-agnostic: it means "this unit has a shield". Guest-rendered ghosts
//! carry the snapshot-mirrored `RemoteTempHpEffect` instead, so the ring
//! renders identically on both multiplayer peers (no `GhostEntity`
//! exclusion — visual systems run on ghosts by convention).
//!
//! Mirrors the Mark of Death indicator pattern: an independent tracking
//! entity follows its target each frame and despawns itself the moment the
//! target stops matching (buff expired, unit died, entity despawned).

use std::collections::HashSet;

use bevy::prelude::*;

use super::super::components::{Corpse, Hitbox, RemoteTempHpEffect, TemporaryHitPoints};
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Ring radius as a fraction of the target's hitbox radius. Tucked inside the
/// sprite's footprint: since the ring renders BEHIND units
/// (`depth_bias -100`), the sprite occludes its back and sides, leaving the
/// front arc visible at the unit's feet (the camera looks down at ~43°, so
/// that arc clears the sprite's bottom edge). Scaling off the hitbox keeps
/// the ring proportional on kings, brutes and bosses — all of which can be
/// shielded — instead of leaving a small disc floating inside a large sprite.
const RING_RADIUS_FRACTION: f32 = 0.91;
/// Height above the unit's feet, avoiding z-fighting with the battlefield.
const RING_GROUND_OFFSET: f32 = 1.0;
/// Sine pulse speed (radians/second) — slow, calm breathing (~5s cycle).
const RING_PULSE_SPEED: f32 = 1.2;
/// Pulse amplitude as a fraction of the base radius — barely perceptible.
const RING_PULSE_AMPLITUDE: f32 = 0.06;
/// Peak opacity of the fade. The ring breathes from fully hidden up to this
/// and back, in step with the scale pulse.
const RING_MAX_ALPHA: f32 = 0.5;

/// Tracking component on the ring entity, pointing at the shielded unit.
#[derive(Component)]
pub struct TempHpRingIndicator {
    pub target: Entity,
}

/// Computes the ring's pulsed scale factor.
fn ring_pulse_scale(elapsed_secs: f32) -> f32 {
    1.0 + (elapsed_secs * RING_PULSE_SPEED).sin() * RING_PULSE_AMPLITUDE
}

/// Computes the ring's faded opacity, sweeping 0 → `RING_MAX_ALPHA` → 0.
fn ring_pulse_alpha(elapsed_secs: f32) -> f32 {
    RING_MAX_ALPHA * ((elapsed_secs * RING_PULSE_SPEED).sin() * 0.5 + 0.5)
}

/// Computes the ring's world radius for a given target.
fn ring_radius(hitbox: &Hitbox) -> f32 {
    hitbox.radius * RING_RADIUS_FRACTION
}

/// Computes the ring's world position from its target's transform and hitbox
/// (unit origin is the sprite center; feet are half the hitbox height down).
fn ring_position(target_pos: Vec3, hitbox: &Hitbox) -> Vec3 {
    Vec3::new(
        target_pos.x,
        target_pos.y - hitbox.height * 0.5 + RING_GROUND_OFFSET,
        target_pos.z,
    )
}

/// Spawns a feet ring for newly shielded units that don't have one yet.
///
/// `Added`-driven so the system does no per-unit work while shields merely
/// persist (Guardian Circle shields whole armies for 20s+ at a time).
/// Re-inserting an existing `TemporaryHitPoints` (a buff refresh) doesn't
/// retrigger `Added`, so refreshes can't double-spawn.
///
/// **Ghosts are driven only by the host-mirrored marker.** Guardian Circle
/// and Entangle's Sanctuary talent insert the REAL `TemporaryHitPoints` onto
/// ghost entities (deliberately — `forward_status_effects_to_host` polls
/// `Added<TemporaryHitPoints>` on ghosts to relay the buff to the host), but
/// `update_timed_modifier::<TemporaryHitPoints>` is gated behind host-only
/// `is_gameplay_running`, so on the guest that local copy never expires.
/// Keying the ring off it would strand a ring on the ghost for the rest of
/// the match; `RemoteTempHpEffect` is host-authoritative and clears properly.
#[allow(clippy::type_complexity)]
pub fn spawn_temp_hp_rings(
    mut commands: Commands,
    newly_shielded: Query<
        (Entity, &Transform, &Hitbox),
        (
            Or<(
                (Added<TemporaryHitPoints>, Without<GhostEntity>),
                Added<RemoteTempHpEffect>,
            )>,
            Without<Corpse>,
        ),
    >,
    existing_rings: Query<&TempHpRingIndicator>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    // Only populated on frames where a shield was actually granted, so the
    // set build is off the steady-state path entirely.
    let tracked: HashSet<Entity> = existing_rings.iter().map(|r| r.target).collect();

    for (entity, transform, hitbox) in &newly_shielded {
        if tracked.contains(&entity) {
            continue;
        }

        commands.spawn((
            TempHpRingIndicator { target: entity },
            Mesh3d(visual_assets.unit_ring.clone()),
            MeshMaterial3d(visual_assets.temp_hp_ring.clone()),
            Transform::from_translation(ring_position(transform.translation, hitbox))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(
                    ring_radius(hitbox) * ring_pulse_scale(time.elapsed_secs()),
                )),
            OnGameplayScreen,
        ));
    }
}

/// Follows shielded units, breathes the shared ring material's opacity, and
/// despawns rings whose target lost its shield, died, or despawned.
///
/// Every ring shares one material handle, so the fade is written once per
/// frame (not per ring) and all shields breathe in unison — matching the
/// scale pulse, which is likewise driven off global elapsed time. Only the
/// alpha channel is touched, so the Excremage theme override's hue survives.
#[allow(clippy::type_complexity)]
pub fn update_temp_hp_rings(
    mut commands: Commands,
    mut rings: Query<(Entity, &TempHpRingIndicator, &mut Transform)>,
    shielded_units: Query<
        (&Transform, &Hitbox),
        (
            // Ghost arm is marker-only — see `spawn_temp_hp_rings`. A ghost's
            // real `TemporaryHitPoints` never expires on the guest, so a ring
            // keyed off it would never despawn.
            Or<(
                (With<TemporaryHitPoints>, Without<GhostEntity>),
                With<RemoteTempHpEffect>,
            )>,
            Without<Corpse>,
            Without<TempHpRingIndicator>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let pulse = ring_pulse_scale(time.elapsed_secs());

    if let Some(material) = materials.get_mut(&visual_assets.temp_hp_ring) {
        material.base_color = material
            .base_color
            .with_alpha(ring_pulse_alpha(time.elapsed_secs()));
    }

    for (ring_entity, ring, mut ring_transform) in &mut rings {
        if let Ok((target_transform, hitbox)) = shielded_units.get(ring.target) {
            ring_transform.translation = ring_position(target_transform.translation, hitbox);
            ring_transform.scale = Vec3::splat(ring_radius(hitbox) * pulse);
        } else {
            commands.entity(ring_entity).try_despawn();
        }
    }
}
