//! Multiplayer spawn-queue types: task enum and queue resource.

use crate::config::save_data::{SavedBoulder, SavedBush, SavedFlora, SavedPond, SavedTree};

/// Multiplayer-specific spawn tasks.
pub enum MpSpawnTask {
    /// Build the full single-player battlefield (tiled ground, castle #1,
    /// left/right wall backdrops, wall-floor, stone/sand underlays, lava
    /// pool, water-ripple mesh).
    Battlefield,
    /// Spawn the second castle (Castle 1 is spawned by `setup_battlefield`).
    Castle2,
    /// Initialize the pathfinding grid.
    PathfindingGrid,
    /// Load wizard sprite sheet assets.
    LoadWizardAssets,
    HostWizard,
    GuestWizard,
    HostInfantry {
        unit_index: u32,
    },
    HostArcher {
        unit_index: u32,
    },
    GuestInfantry {
        unit_index: u32,
    },
    GuestArcher {
        unit_index: u32,
    },
    HostKing,
    HostKingsGuard {
        guard_index: u32,
    },
    GuestKing,
    GuestKingsGuard {
        guard_index: u32,
    },
    /// A single flora decoration (visual only).
    Flora {
        flora: SavedFlora,
    },
    /// A single boulder (pathfinding obstacle).
    TerrainBoulder {
        boulder: SavedBoulder,
    },
    /// A single tree (pathfinding obstacle).
    TerrainTree {
        tree: SavedTree,
    },
    /// A single pond (slow terrain).
    TerrainPond {
        pond: SavedPond,
    },
    /// A single bush (slow terrain).
    TerrainBush {
        bush: SavedBush,
    },
    /// Load the cauldron sprite-sheet asset (must precede `Cauldron`).
    LoadCauldronAssets,
    /// Spawn the cauldron entity so brewing works in multiplayer.
    Cauldron,
}

impl MpSpawnTask {
    /// Tasks that read a resource inserted by a prior task via
    /// `commands.insert_resource(...)`. The processor must flush the
    /// command queue (end the frame) before running these — Bevy doesn't
    /// apply deferred resource inserts until the next sync point.
    pub(crate) fn needs_command_flush(&self) -> bool {
        matches!(
            self,
            MpSpawnTask::HostWizard | MpSpawnTask::GuestWizard | MpSpawnTask::Cauldron,
        )
    }

    /// Tasks that schedule a deferred `commands.insert_resource(...)` that
    /// a subsequent task will need to read.
    pub(crate) fn creates_deferred_state(&self) -> bool {
        matches!(
            self,
            MpSpawnTask::LoadWizardAssets | MpSpawnTask::LoadCauldronAssets,
        )
    }
}

/// Resource that holds the multiplayer spawn queue.
#[derive(bevy::prelude::Resource)]
pub struct MpSpawnQueue {
    pub tasks: Vec<MpSpawnTask>,
}

impl MpSpawnQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn pop_next(&mut self) -> Option<MpSpawnTask> {
        if self.tasks.is_empty() {
            None
        } else {
            Some(self.tasks.remove(0))
        }
    }

    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}
