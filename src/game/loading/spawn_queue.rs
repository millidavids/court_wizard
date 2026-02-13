use bevy::prelude::*;

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
    Behemoth,
    Battlefield,
    #[allow(dead_code)]
    Castle,
    Wizard,
    LoadCauldronAssets,
    Cauldron,
    PathfindingGrid,
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
