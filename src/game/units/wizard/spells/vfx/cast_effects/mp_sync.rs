//! MP-synced wrappers — spawn locally AND emit a cast-event for the remote peer.
//!
//! These wrappers replace the bare `spawn_*` helpers in spell casting handlers
//! that need cross-peer visual parity. The receiver dispatches each event back
//! to the matching `spawn_*` call via `apply_remote_cast_events`. Spell casting
//! handlers that already take a `ResMut<PendingCastEvents>` should call the
//! `_synced` variants; SP-only code paths can keep calling the bare helpers.

use bevy::prelude::*;

use super::cast_vfx::{
    SpellSchool, spawn_aura_bubble, spawn_aura_bubble_contracting, spawn_dust_smoke,
    spawn_floating_motes, spawn_school_flare, spawn_smoke_poof,
};
use crate::game::multiplayer::spell_sync::PendingCastEvents;
use crate::game::units::wizard::spells::visual_assets::{AuraSphereMaterial, SpellVisualAssets};
use crate::networking::snapshot::{
    AuraBubbleVariant, CastEventKind, CastEventSnapshot, MoteMaterial, PoofVariant, SparkMaterial,
    SpellSchoolWire,
};

/// Pushes a one-shot cast event into the outgoing MP snapshot.
///
/// `extra` is event-specific (radius, duration, count, etc.) — see
/// `CastEventKind` for per-kind semantics. No-op in single-player so the
/// `events` Vec doesn't grow unbounded (the drain system only runs in MP).
pub fn emit_cast_event(
    pending: &mut PendingCastEvents,
    kind: CastEventKind,
    subkind: u8,
    position: Vec3,
    extra: [f32; 4],
) {
    if !pending.mp_active {
        return;
    }
    pending.events.push(CastEventSnapshot {
        kind: kind as u8,
        subkind,
        x: position.x,
        y: position.y,
        z: position.z,
        extra,
    });
}

/// School-flare wrapper: spawns locally AND emits a `SchoolFlare` event.
pub fn spawn_school_flare_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    local_origin: Vec3,
    school: SpellSchool,
    time_secs: f32,
) {
    spawn_school_flare(commands, assets, local_origin, school, time_secs);
    // Encode through the wire enum's #[repr(u8)] discriminants so the ordinals
    // stay tied to SpellSchoolWire at compile time (the receiver decodes via
    // SpellSchoolWire::try_from). Don't hand-write magic numbers here.
    let subkind = match school {
        SpellSchool::Fire => SpellSchoolWire::Fire as u8,
        SpellSchool::Lightning => SpellSchoolWire::Lightning as u8,
        SpellSchool::Arcane => SpellSchoolWire::Arcane as u8,
        SpellSchool::Nature => SpellSchoolWire::Nature as u8,
        SpellSchool::Holy => SpellSchoolWire::Holy as u8,
        SpellSchool::Dark => SpellSchoolWire::Dark as u8,
        SpellSchool::Force => SpellSchoolWire::Force as u8,
        SpellSchool::Transmutation => SpellSchoolWire::Transmutation as u8,
    };
    emit_cast_event(
        pending,
        CastEventKind::SchoolFlare,
        subkind,
        local_origin,
        [0.0; 4],
    );
}

/// Aura-bubble wrapper: spawns locally AND emits an `AuraBubble` event.
#[allow(clippy::too_many_arguments)]
pub fn spawn_aura_bubble_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    material: Handle<AuraSphereMaterial>,
    variant: AuraBubbleVariant,
    position: Vec3,
    max_radius: f32,
    duration: f32,
) {
    spawn_aura_bubble(commands, assets, material, position, max_radius, duration);
    emit_cast_event(
        pending,
        CastEventKind::AuraBubble,
        variant as u8,
        position,
        [max_radius, duration, 0.0, 0.0],
    );
}

/// Contracting aura-bubble wrapper: spawns locally AND emits an event.
#[allow(clippy::too_many_arguments)]
pub fn spawn_aura_bubble_contracting_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    material: Handle<AuraSphereMaterial>,
    variant: AuraBubbleVariant,
    position: Vec3,
    max_radius: f32,
    duration: f32,
) {
    spawn_aura_bubble_contracting(commands, assets, material, position, max_radius, duration);
    emit_cast_event(
        pending,
        CastEventKind::AuraBubbleContract,
        variant as u8,
        position,
        [max_radius, duration, 0.0, 0.0],
    );
}

/// Smoke-poof wrapper: spawns locally AND emits a `SmokePoof` event.
/// `extra[0]` carries the puff count so the remote peer matches.
#[allow(clippy::too_many_arguments)]
pub fn spawn_smoke_poof_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    material: &Handle<StandardMaterial>,
    variant: PoofVariant,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    spawn_smoke_poof(commands, assets, material, position, count, time_secs);
    emit_cast_event(
        pending,
        CastEventKind::SmokePoof,
        variant as u8,
        position,
        [count as f32, 0.0, 0.0, 0.0],
    );
}

/// Floating-motes wrapper: spawns locally AND emits a `FloatingMotes` event.
#[allow(clippy::too_many_arguments)]
pub fn spawn_floating_motes_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    material: &Handle<StandardMaterial>,
    variant: MoteMaterial,
    center: Vec3,
    radius: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_floating_motes(commands, assets, material, center, radius, count, time_secs);
    emit_cast_event(
        pending,
        CastEventKind::FloatingMotes,
        variant as u8,
        center,
        [radius, count as f32, 0.0, 0.0],
    );
}

/// Sparks wrapper: spawns locally AND emits a `Sparks` event.
/// `extra[0]` carries the spark count so the remote peer matches.
#[allow(clippy::too_many_arguments)]
pub fn spawn_sparks_with_material_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    variant: SparkMaterial,
    material: Handle<StandardMaterial>,
    position: Vec3,
    count: usize,
    time_secs: f32,
) {
    super::super::fire_effects::spawn_sparks_with_material(
        commands, assets, position, count, time_secs, material,
    );
    emit_cast_event(
        pending,
        CastEventKind::Sparks,
        variant as u8,
        position,
        [count as f32, 0.0, 0.0, 0.0],
    );
}

/// Dust-smoke wrapper: spawns locally AND emits a `DustSmoke` event.
#[allow(clippy::too_many_arguments)]
pub fn spawn_dust_smoke_synced(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    pending: &mut PendingCastEvents,
    position: Vec3,
    half_width: f32,
    count: usize,
    time_secs: f32,
) {
    spawn_dust_smoke(commands, assets, position, half_width, count, time_secs);
    emit_cast_event(
        pending,
        CastEventKind::DustSmoke,
        0,
        position,
        [half_width, count as f32, 0.0, 0.0],
    );
}

/// Emits a `BanishmentLens` event. The local spawn is handled by the
/// existing `spawn_banishment_vfx` in `banishment/systems.rs`; this wrapper
/// only ships the network event.
pub fn emit_banishment_lens_event(
    pending: &mut PendingCastEvents,
    position: Vec3,
    radius: f32,
    duration: f32,
) {
    emit_cast_event(
        pending,
        CastEventKind::BanishmentLens,
        0,
        position,
        [radius, duration, 0.0, 0.0],
    );
}
