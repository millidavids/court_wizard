//! Multiplayer replication for Warglock gun visuals.
//!
//! Gun *damage* already reaches the opponent through the normal CRDT health
//! pipeline (hitscan / flame collisions call `apply_spell_damage` on ghost
//! units, which forwards to the host). What the opponent could NOT see was the
//! *visuals* — muzzle flashes, bullet tracers, flame particles, the burning
//! ground patch — and the *sound*. This module ships those:
//!
//! - One-shot visuals (muzzle flash, tracer, flame particle) ride the existing
//!   `CastEventSnapshot` pipeline: we watch for locally-spawned visual entities
//!   (`Added<…>`) and push a matching cast event, which the opponent reproduces
//!   in `apply_remote_cast_events` via the `spawn_ghost_*` helpers below.
//! - The persistent flamethrower ground fire is tagged `NetworkedSpellEffect`
//!   so the standard spell-effect snapshot ships it as a ghost spell effect.

use bevy::prelude::*;

use super::components::{
    BulletTracer, FlameParticle, GhostBulletTracer, GhostFlameParticle, GhostMuzzleFlash, GunType,
    MuzzleFlash,
};
use super::constants;
use super::fire::{FlameGroundFire, spawn_muzzle_flash, spawn_tracer};
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::multiplayer::spell_sync::{PendingCastEvents, emit_cast_event};
use crate::game::units::wizard::spells::audio::emit_sfx_event;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::{CastEventKind, SpellEffectKind, SpellSoundId};

/// Maps the active gun to its one-shot shot sound for cross-peer playback.
fn gun_shot_sound(gun: GunType) -> SpellSoundId {
    match gun {
        GunType::MachineGun => SpellSoundId::MachineGunShot,
        GunType::Magnum => SpellSoundId::MagnumShot,
        GunType::Shotgun => SpellSoundId::ShotgunShot,
        GunType::RocketLauncher => SpellSoundId::RocketShot,
        GunType::Flamethrower => SpellSoundId::FlamethrowerBurst,
    }
}

/// Ships each locally-spawned muzzle flash (and the matching shot sound) to the
/// opponent. `Added<MuzzleFlash>` fires once per shot fired by a hitscan/rocket
/// gun, so this also serves as the gunshot SFX trigger. The shot sound is read
/// from the flash's own `gun` field (not the live `GunState`) so a rapid
/// fire-then-switch ships the gun that actually fired.
pub(super) fn replicate_gun_muzzle_flashes(
    new_flashes: Query<(&Transform, &MuzzleFlash), (Added<MuzzleFlash>, Without<GhostMuzzleFlash>)>,
    mut pending: ResMut<PendingCastEvents>,
) {
    for (transform, flash) in &new_flashes {
        let pos = transform.translation;
        emit_cast_event(
            &mut pending,
            CastEventKind::GunMuzzleFlash,
            0,
            pos,
            [0.0; 4],
        );
        emit_sfx_event(&mut pending, gun_shot_sound(flash.gun), pos);
    }
}

/// Ships each locally-spawned bullet tracer to the opponent (visual only).
pub(super) fn replicate_gun_tracers(
    new_tracers: Query<&BulletTracer, (Added<BulletTracer>, Without<GhostBulletTracer>)>,
    mut pending: ResMut<PendingCastEvents>,
) {
    for tracer in &new_tracers {
        emit_cast_event(
            &mut pending,
            CastEventKind::GunBulletTracer,
            0,
            tracer.origin,
            [
                tracer.velocity.x,
                tracer.velocity.y,
                tracer.velocity.z,
                tracer.max_range,
            ],
        );
    }
}

/// Ships each locally-spawned flame particle to the opponent (visual only).
/// Real (non-ghost) flames only — never re-ship a ghost we received.
pub(super) fn replicate_gun_flames(
    new_flames: Query<
        (&Transform, &FlameParticle),
        (Added<FlameParticle>, Without<GhostFlameParticle>),
    >,
    mut pending: ResMut<PendingCastEvents>,
) {
    for (transform, flame) in &new_flames {
        emit_cast_event(
            &mut pending,
            CastEventKind::GunFlameParticle,
            0,
            transform.translation,
            [
                flame.velocity.x,
                flame.velocity.y,
                flame.velocity.z,
                flame.lifetime,
            ],
        );
    }
}

/// Tags newly-spawned flamethrower ground fire for replication so the standard
/// spell-effect snapshot ships it to the opponent as a ghost spell effect.
/// Runs on the local Warglock peer (real flames only — ghost flames never spawn
/// ground fire, see `despawn_expired_flames`).
pub(super) fn tag_flame_ground_fire_for_replication(
    mut commands: Commands,
    new_patches: Query<Entity, Added<FlameGroundFire>>,
) {
    for entity in &new_patches {
        commands.entity(entity).insert(NetworkedSpellEffect {
            kind: SpellEffectKind::FlameGroundFire,
        });
    }
}

// ── Receiver-side ghost spawns (called from `apply_remote_cast_events`) ──────

/// Spawns a visual-only ghost muzzle flash for the opponent's shot. Tagged
/// `GhostMuzzleFlash` so a mirror-Warglock peer never re-ships it.
pub(crate) fn spawn_ghost_muzzle_flash(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pos: Vec3,
) {
    // The firing gun isn't carried for the visual (the shot SOUND crosses as a
    // separate SfxOneShot event). The `gun` field is never read on ghost flashes
    // — they're excluded from `replicate_gun_muzzle_flashes` — so any value works.
    let entity = spawn_muzzle_flash(commands, assets, pos, GunType::MachineGun);
    commands.entity(entity).insert(GhostMuzzleFlash);
}

/// Spawns a visual-only ghost bullet tracer for the opponent's shot. Tagged
/// `GhostBulletTracer` so a mirror-Warglock peer never re-ships it.
pub(crate) fn spawn_ghost_tracer(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    velocity: Vec3,
    max_range: f32,
) {
    let entity = spawn_tracer(commands, assets, origin, velocity, max_range);
    commands.entity(entity).insert(GhostBulletTracer);
}

/// Spawns a visual-only ghost flame particle for the opponent's flamethrower.
/// Tagged `GhostFlameParticle` so it deals no damage and leaves no ground fire.
pub(crate) fn spawn_ghost_flame(commands: &mut Commands, pos: Vec3, velocity: Vec3, lifetime: f32) {
    commands.spawn((
        Transform::from_translation(pos),
        FlameParticle {
            velocity,
            damage: 0.0,
            lifetime,
            time_alive: 0.0,
            radius: constants::FLAME_PARTICLE_SIZE,
        },
        GhostFlameParticle,
        OnGameplayScreen,
    ));
}
