use bevy::prelude::*;

use crate::config::save_data::{SavedCrystal, SavedWall};

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
    Brute,
    Ogre,
    Hags,
    Battlefield,
    #[allow(dead_code)]
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
}

/// Resource that holds the queue of spawn tasks.
#[derive(Resource)]
pub struct SpawnQueue {
    pub tasks: Vec<SpawnTask>,
}

impl SpawnQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Returns the next task to process (processes one task per frame).
    /// Always drains from the front since the vector shrinks as we process.
    pub fn pop_batch(&mut self, count: usize) -> Vec<SpawnTask> {
        let to_take = count.min(self.tasks.len());
        self.tasks.drain(0..to_take).collect()
    }

    /// Returns true if all tasks have been processed.
    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}
