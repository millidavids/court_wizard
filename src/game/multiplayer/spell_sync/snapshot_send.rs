use bevy::prelude::*;

use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    BeamSnapshot, CastEventKind, CastEventSnapshot, MagicMissileSnapshot, SpellSnapshotData,
    SpellVisualSnapshot, UNRELIABLE_SPELL_SNAPSHOT,
};

/// Resource holding the latest received remote spell visual snapshot.
#[derive(Resource, Default)]
pub struct LatestSpellSnapshot(pub Option<SpellVisualSnapshot>);

/// Outgoing one-shot cast VFX events for this tick.
///
/// Casting handlers push to this via the `_synced` VFX wrappers (or
/// `emit_cast_event` directly); `send_spell_visual_snapshot` drains it into
/// the outgoing `SpellVisualSnapshot.cast_events` vector once per send tick.
/// Receiver dispatches via `apply_cast_event`.
///
/// The `_synced` wrappers gate the push behind an `mp_active` flag so
/// single-player runs don't accumulate events that are never drained (the
/// drain system is `run_if(mp_running)`). Toggling happens via
/// `mark_pending_events_mp_active` / `_inactive` on MP state transitions.
#[derive(Resource, Default)]
pub struct PendingCastEvents {
    pub events: Vec<CastEventSnapshot>,
    pub mp_active: bool,
}

/// Pushes a one-shot cast VFX event so the remote peer reproduces it locally.
/// No-op in single-player (when `mp_active` is false). Used for visuals that
/// have no persistent component to snapshot (gun muzzle flashes, tracers, flame
/// particles, sword arcs).
pub(crate) fn emit_cast_event(
    pending: &mut PendingCastEvents,
    kind: CastEventKind,
    subkind: u8,
    pos: Vec3,
    extra: [f32; 4],
) {
    if pending.mp_active {
        pending.events.push(CastEventSnapshot {
            kind: kind as u8,
            subkind,
            x: pos.x,
            y: pos.y,
            z: pos.z,
            extra,
        });
    }
}

/// Marks `PendingCastEvents` as MP-active so `_synced` wrappers start pushing.
pub fn mark_pending_events_mp_active(mut pending: ResMut<PendingCastEvents>) {
    pending.mp_active = true;
}

/// Marks `PendingCastEvents` as MP-inactive and clears any straggler events.
pub fn mark_pending_events_mp_inactive(mut pending: ResMut<PendingCastEvents>) {
    pending.mp_active = false;
    pending.events.clear();
}

/// Collects magic missiles and beams, then serializes and sends the complete
/// spell visual snapshot over the unreliable channel.
pub fn send_spell_visual_snapshot(
    mut connection: ResMut<NetworkConnection>,
    mut spell_data: ResMut<SpellSnapshotData>,
    mut pending_cast_events: ResMut<PendingCastEvents>,
    missiles: Query<&Transform, With<MagicMissile>>,
    // `Has<CrystalSpawn>` rather than a `Without` filter: the peer still needs to SEE
    // a crystal's beams, it just must not dress them up as wizard casts. Filtering
    // them out here would have traded a wrong visual for a missing one.
    beams: Query<(
        &DisintegrateBeam,
        Has<crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn>,
    )>,
) {
    let snapshot = SpellVisualSnapshot {
        spell_effects: std::mem::take(&mut spell_data.spell_effects),
        spell_projectiles: std::mem::take(&mut spell_data.spell_projectiles),
        spell_arcs: std::mem::take(&mut spell_data.spell_arcs),
        magic_missiles: missiles
            .iter()
            .map(|t| MagicMissileSnapshot {
                x: t.translation.x,
                y: t.translation.y,
                z: t.translation.z,
            })
            .collect(),
        beams: beams
            .iter()
            .map(|(beam, from_crystal)| BeamSnapshot {
                ox: beam.origin.x,
                oy: beam.origin.y,
                oz: beam.origin.z,
                dx: beam.direction.x,
                dy: beam.direction.y,
                dz: beam.direction.z,
                length: beam.current_length(),
                width: beam.beam_width(),
                flags: if from_crystal {
                    crate::networking::snapshot::BEAM_FLAG_FROM_CRYSTAL
                } else {
                    0
                },
            })
            .collect(),
        // Drain one-shot cast events accumulated by casting handlers this
        // tick. Casting handlers run before `send_spell_visual_snapshot` in
        // Bevy's default `Update` schedule, so events emitted on cast
        // completion (e.g. `spawn_school_flare_synced` on fireball release)
        // ship on the same frame.
        cast_events: std::mem::take(&mut pending_cast_events.events),
    };

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(UNRELIABLE_SPELL_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}

/// Receives spell visual snapshots from the remote peer and stores the latest.
///
/// Filters incoming unreliable data by type prefix byte, taking only spell
/// snapshots and re-queuing any non-spell data (game snapshots) for other systems.
pub fn receive_spell_visual_snapshot(
    mut connection: ResMut<NetworkConnection>,
    mut latest: ResMut<LatestSpellSnapshot>,
) {
    // Separate spell snapshots from other unreliable data
    let all_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_spell_data: Option<&[u8]> = None;

    for data in &all_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_SPELL_SNAPSHOT => {
                latest_spell_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    // Re-queue non-spell data for other systems (game snapshots)
    connection.incoming_unreliable = other_data;

    // Deserialize the latest spell snapshot if a new one arrived this frame.
    // We do NOT reset `latest.0` on frames without new data — keeping the
    // previously-deserialized snapshot lets `apply_remote_spell_snapshot`
    // re-render ghost arcs every frame with fresh random jitter (matching
    // the local caster's per-frame `update_lightning_bolts` crackle).
    // Without persistence the bolts only redraw at the network snapshot
    // rate and visibly stutter between updates.
    //
    // One-shot `cast_events` (school flares, SFX, etc.) must NOT re-fire
    // each frame, so on stale frames we keep `latest.0` but empty its
    // `cast_events` list — leaving everything else (effects, projectiles,
    // arcs, missiles, beams) intact for per-frame re-rendering.
    if let Some(spell_bytes) = latest_spell_data {
        match bincode::deserialize::<SpellVisualSnapshot>(spell_bytes) {
            Ok(snapshot) => {
                latest.0 = Some(snapshot);
            }
            Err(_) => {
                warn!(
                    "Failed to deserialize spell visual snapshot ({} bytes)",
                    spell_bytes.len()
                );
            }
        }
    } else if let Some(s) = latest.0.as_mut() {
        s.cast_events.clear();
    }
}
