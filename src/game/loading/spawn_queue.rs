use std::collections::VecDeque;

use bevy::prelude::*;

use crate::config::save_data::{
    SavedBoulder, SavedBush, SavedCrystal, SavedFlora, SavedPond, SavedTree, SavedWall,
};

/// A task that spawns a single entity.
/// Each plugin can enqueue tasks for the entities it needs to spawn.
pub enum SpawnTask {
    DefenderInfantry {
        unit_index: u32,
    },
    AttackerInfantry {
        unit_index: u32,
        level: u32,
    },
    DefenderArcher {
        unit_index: u32,
    },
    AttackerArcher {
        unit_index: u32,
        level: u32,
    },
    King,
    KingsGuard {
        guard_index: u32,
    },
    UpgradeToDispeller {
        entity: Entity,
    },
    UpgradeToHealer {
        entity: Entity,
    },
    UpgradeToShielder {
        entity: Entity,
    },
    AttackerAssassin {
        unit_index: u32,
        level: u32,
    },
    AttackerAerialist {
        unit_index: u32,
        level: u32,
    },
    Brute,
    Ogre,
    Hags,
    DarkMage,
    Battlefield,
    Castle,
    Wizard,
    LoadWizardAssets,
    LoadCauldronAssets,
    Cauldron,
    PathfindingGrid,
    // Upgrade tasks
    SelectInfantryUpgrades,
    SelectArcherUpgrades,
    SelectDispellerUpgrades,
    SelectHealerUpgrades,
    SelectShielderUpgrades,
    SelectEliteUpgrades,
    UpgradeToElite {
        entity: Entity,
    },
    UpgradeToCommander {
        entity: Entity,
    },
    PermanentWall {
        wall: SavedWall,
    },
    PermanentCrystal {
        crystal: SavedCrystal,
        damage_mult: f32,
        count_mult: f32,
        resonance_cascade: bool,
    },
    Flora {
        flora: SavedFlora,
    },
    TramplingOverlay,
    TerrainTree {
        tree: SavedTree,
    },
    TerrainPond {
        pond: SavedPond,
    },
    TerrainBush {
        bush: SavedBush,
    },
    TerrainBoulder {
        boulder: SavedBoulder,
    },
}

impl SpawnTask {
    /// Returns true if this task reads entities or resources from the World
    /// that may have been created by deferred commands earlier in the queue.
    /// When true, a command flush (frame boundary) is needed before this task
    /// if any spawn tasks ran in the current frame.
    pub fn needs_command_flush(&self) -> bool {
        matches!(
            self,
            SpawnTask::Wizard
                | SpawnTask::Cauldron
                | SpawnTask::SelectInfantryUpgrades
                | SpawnTask::SelectArcherUpgrades
                | SpawnTask::SelectDispellerUpgrades
                | SpawnTask::SelectHealerUpgrades
                | SpawnTask::SelectShielderUpgrades
                | SpawnTask::SelectEliteUpgrades
                | SpawnTask::UpgradeToElite { .. }
                | SpawnTask::UpgradeToCommander { .. }
        )
    }

    /// Returns true if this task creates new entities or inserts resources
    /// via deferred commands (i.e., it produces World state that later tasks
    /// might need to read).
    pub fn creates_deferred_state(&self) -> bool {
        !matches!(
            self,
            SpawnTask::SelectInfantryUpgrades
                | SpawnTask::SelectArcherUpgrades
                | SpawnTask::SelectDispellerUpgrades
                | SpawnTask::SelectHealerUpgrades
                | SpawnTask::SelectShielderUpgrades
                | SpawnTask::SelectEliteUpgrades
                | SpawnTask::UpgradeToElite { .. }
                | SpawnTask::UpgradeToCommander { .. }
                | SpawnTask::UpgradeToDispeller { .. }
                | SpawnTask::UpgradeToHealer { .. }
                | SpawnTask::UpgradeToShielder { .. }
        )
    }
}

/// Resource that holds the queue of spawn tasks.
///
/// Uses `VecDeque` for O(1) front removal during batch processing.
#[derive(Resource)]
pub struct SpawnQueue {
    pub tasks: VecDeque<SpawnTask>,
}

impl SpawnQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    /// Returns true if all tasks have been processed.
    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}
