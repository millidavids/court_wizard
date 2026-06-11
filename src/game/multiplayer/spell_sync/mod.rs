//! Bidirectional spell visual synchronization.
//!
//! Both host and guest collect their local spell visuals into a
//! `SpellVisualSnapshot` and send it over the unreliable channel.
//! The receiving client renders ghost entities from the snapshot.
//!
//! This runs symmetrically on both clients — each sees the other's spells.

mod effect_collect;
mod ghost_spawn;
mod ghost_update;
mod projectile_collect;
mod snapshot_send;

// ── Public surface (reproduces the original spell_sync.rs exports) ──────────

pub use snapshot_send::{
    LatestSpellSnapshot, PendingCastEvents, mark_pending_events_mp_active,
    mark_pending_events_mp_inactive, receive_spell_visual_snapshot, send_spell_visual_snapshot,
};

pub(crate) use snapshot_send::emit_cast_event;

pub use effect_collect::collect_spell_effect_snapshots;
pub use projectile_collect::collect_spell_projectile_snapshots;

pub use ghost_spawn::apply_remote_spell_snapshot;

pub use ghost_update::{apply_remote_cast_events, spawn_ghost_beam_impact_vfx};
