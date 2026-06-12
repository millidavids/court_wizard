//! Multiplayer game plugin.
//!
//! Registers only multiplayer-specific systems. The host reuses all single-player
//! gameplay systems (movement, combat, spells, etc.) via `is_gameplay_running`,
//! so this plugin only adds networking, guest rendering, score screen, and
//! disconnect detection.

use bevy::prelude::*;

use super::registration;

/// Plugin that manages multiplayer gameplay.
///
/// The host reuses all single-player gameplay systems (registered by `GamePlugin`,
/// `UnitsPlugin`, `InfantryPlugin`, `ArcherPlugin`, `KingPlugin`, etc.) via the
/// `is_gameplay_running` run condition. This plugin only adds:
/// - Loading and camera setup
/// - Resource init/cleanup
/// - Host networking (ID assignment, snapshots, king death check)
/// - Guest snapshot rendering and game-over handling
/// - Score screen UI and rematch flow
/// - Disconnect detection
///
/// `build()` delegates to `registration::*` helpers, each covering a CONTIGUOUS
/// span of the wiring in source order (A→F). The split is organizational only:
/// systems are inserted in the exact same order as before, so every
/// `.after()`/`.before()`/`run_if`/system-set constraint — and the deterministic
/// insertion order the spell-sync/snapshot comments rely on — is preserved.
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, app: &mut App) {
        registration::setup::register(app); // Span A — loading, camera, resource init/cleanup
        registration::coop::register(app); // Span B — co-op host + sync pause + cast cancel
        registration::host_sync::register(app); // Span C — CRDT sync, tally, king death, host snapshots
        registration::spell_visuals::register(app); // Span D — bidirectional spell-visual sync
        registration::net_relay::register(app); // Span E — guest snapshot + host/guest message relay
        registration::ui_flow::register(app); // Span F — escape, pause, disconnect, score screen
    }
}
