//! Network entity identification and mapping.
//!
//! The host assigns a stable `NetworkEntityId` to every game entity at spawn time.
//! The guest maintains a mapping from these IDs to local entities for rendering.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Stable network identifier for an entity, assigned by the host.
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NetworkEntityId(pub u32);

/// Monotonically increasing counter for assigning network entity IDs.
#[derive(Resource, Default)]
pub struct EntityIdCounter(pub u32);

impl EntityIdCounter {
    /// Returns the next available ID and increments the counter.
    pub fn next(&mut self) -> NetworkEntityId {
        let id = self.0;
        self.0 += 1;
        NetworkEntityId(id)
    }
}

/// Maps remote network entity IDs to local Bevy entities (used by the guest).
#[derive(Resource, Default)]
pub struct NetworkEntityMap {
    /// Remote network ID → local entity.
    pub remote_to_local: HashMap<u32, Entity>,

    /// Local entity → remote network ID.
    pub local_to_remote: HashMap<Entity, u32>,
}

impl NetworkEntityMap {
    /// Inserts a bidirectional mapping.
    pub fn insert(&mut self, remote_id: u32, local_entity: Entity) {
        self.remote_to_local.insert(remote_id, local_entity);
        self.local_to_remote.insert(local_entity, remote_id);
    }

    /// Removes a mapping by remote ID, returning the local entity if it existed.
    pub fn remove_by_remote(&mut self, remote_id: u32) -> Option<Entity> {
        if let Some(entity) = self.remote_to_local.remove(&remote_id) {
            self.local_to_remote.remove(&entity);
            Some(entity)
        } else {
            None
        }
    }
}
