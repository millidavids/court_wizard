//! Multiplayer loading resources: sync state and config backup.

use bevy::prelude::*;

/// Tracks whether both players have finished loading.
#[derive(Resource, Default)]
pub struct MpLoadingSync {
    pub my_loaded: bool,
    pub peer_loaded: bool,
}

/// Snapshot of the `GameConfig` fields multiplayer overwrites during loading,
/// so MP exit can restore them and not pollute later single-player runs.
#[derive(Resource)]
pub struct MpConfigBackup {
    pub previous_seed: Option<u64>,
    pub previous_current_level: u32,
}
