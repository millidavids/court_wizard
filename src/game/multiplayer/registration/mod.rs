//! Registration helpers for `MultiplayerGamePlugin`.
//!
//! `plugin.rs` stays registration-only; its `build()` calls these helpers in
//! source order (A→F). The split is purely organizational: each helper covers a
//! CONTIGUOUS span of the original `build()` body, so the order in which systems
//! are inserted into the schedule is byte-identical to before — preserving the
//! load-bearing `.after()`/`.before()`/`run_if`/system-set ordering and the
//! deterministic insertion order the spell-sync/snapshot comments rely on.

pub(super) mod coop;
pub(super) mod host_sync;
pub(super) mod net_relay;
pub(super) mod setup;
pub(super) mod spell_visuals;
pub(super) mod ui_flow;
