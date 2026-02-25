use bevy::prelude::*;

/// Unit type enum for upgrade tasks.
#[derive(Clone, Copy, Debug)]
pub enum UnitType {
    Infantry,
    Archer,
}

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
    Behemoth,
    Boss,
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
    UpgradeToElite {
        entity: Entity,
        unit_type: UnitType,
    },
    UpgradeToCommander {
        entity: Entity,
        unit_type: UnitType,
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
