//! Per-unit "recently healed" visual.
//!
//! `RecentlyHealedVfx` is a short-lived, purely visual marker refreshed on
//! every successful Healing Plume heal tick. While it (or the MP-mirrored
//! `RemoteHealingEffect`) is present, sparse green motes rise around the
//! unit's body — a subtle cue that the unit is under active healing. It also
//! feeds the `UnitFlags::HEALING` snapshot bit so guests see the motes on
//! host-healed units.

use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{Corpse, Hitbox, RemoteHealingEffect, impl_timed_modifier};
use crate::game::units::wizard::spells::vfx::components::FloatingMote;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::vfx::unit_motes::{mote_scatter_offset, unit_effect_tick};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// How long the marker (and its motes) lingers after a heal tick. Covers the
/// zone's 0.5 s tick interval with buffer so continuous healing reads as one
/// unbroken effect instead of flickering between ticks.
const RECENTLY_HEALED_DURATION: f32 = 1.2;
/// Seconds between motes per healed unit. Has to compete with the zone's own
/// motes (2 every 0.15s across the whole plume), so it can't be too sparse.
const HEAL_MOTE_INTERVAL: f32 = 0.3;
/// Horizontal scatter radius of motes around the unit's center. Pushed past
/// the sprite's 48-unit half-width so motes ring the silhouette rather than
/// spawning inside it, where the billboard depth-occludes half of them.
const HEAL_MOTE_SCATTER_RADIUS: f32 = 52.0;
/// Mote quad size (world units).
const HEAL_MOTE_SIZE: f32 = 9.0;
/// Mote lifetime in seconds.
const HEAL_MOTE_LIFETIME: f32 = 1.5;
/// Upward drift speed of motes (world units/second).
const HEAL_MOTE_RISE_SPEED: f32 = 14.0;

/// Marker for a unit that was just healed by a Healing Plume tick.
///
/// Purely visual — carries no gameplay effect and expires on its own via the
/// generic `update_timed_modifier` system.
#[derive(Component)]
pub(crate) struct RecentlyHealedVfx {
    pub time_remaining: f32,
}

impl RecentlyHealedVfx {
    pub(crate) fn new() -> Self {
        Self {
            time_remaining: RECENTLY_HEALED_DURATION,
        }
    }

    /// Ticks the timer. Returns `true` when expired.
    pub(crate) fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}

impl_timed_modifier!(RecentlyHealedVfx);

/// Inserts or refreshes the marker on a unit that just received healing.
pub(crate) fn refresh_recently_healed_vfx(
    commands: &mut Commands,
    entity: Entity,
    existing: Option<Mut<RecentlyHealedVfx>>,
) {
    if let Some(mut vfx) = existing {
        vfx.time_remaining = RECENTLY_HEALED_DURATION;
    } else {
        commands.entity(entity).insert(RecentlyHealedVfx::new());
    }
}

/// Emits sparse green motes around units under active healing.
///
/// Host-simulated units carry the real `RecentlyHealedVfx`; guest-rendered
/// ghosts carry `RemoteHealingEffect` (mirrored from the host's snapshot).
/// Querying both lets the effect play identically on either peer, so there is
/// deliberately no `GhostEntity` exclusion. Cadence is per-entity via
/// phase-offset time-boundary crossing so a freshly healed army doesn't
/// sparkle in unison.
#[allow(clippy::type_complexity)]
pub(crate) fn emit_heal_regen_vfx(
    mut commands: Commands,
    healed_units: Query<
        (Entity, &Transform, &Hitbox),
        (
            Or<(With<RecentlyHealedVfx>, With<RemoteHealingEffect>)>,
            Without<Corpse>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, transform, hitbox) in &healed_units {
        let Some(seed) = unit_effect_tick(entity, t, dt, HEAL_MOTE_INTERVAL) else {
            continue;
        };

        let pos = transform.translation;
        let scatter = mote_scatter_offset(seed, HEAL_MOTE_SCATTER_RADIUS, 0.3);
        let y_jitter = ((seed * 7.3).sin() * 0.4) * hitbox.height;

        commands.spawn((
            FloatingMote {
                velocity: Vec3::new(0.0, HEAL_MOTE_RISE_SPEED, 0.0),
                time_alive: 0.0,
                lifetime: HEAL_MOTE_LIFETIME,
                base_size: HEAL_MOTE_SIZE,
                phase: seed,
            },
            Mesh3d(visual_assets.particle_quad.clone()),
            MeshMaterial3d(visual_assets.heal_regen_mote.clone()),
            Transform::from_translation(Vec3::new(
                pos.x + scatter.x,
                pos.y + y_jitter,
                pos.z + scatter.y,
            ))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(0.0)),
            OnGameplayScreen,
        ));
    }
}
