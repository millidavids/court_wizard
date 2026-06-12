//! CRDT health snapshot types sent from guest to host.

use serde::{Deserialize, Serialize};

/// Compact CRDT health state sent from guest to host.
///
/// The guest sends only its local CRDT counters (damage/healing from peer 1)
/// so the host can merge them. Much smaller than a full GameSnapshot since
/// we only need the network ID + CRDT arrays per unit.
#[derive(Serialize, Deserialize)]
pub struct CrdtSnapshot {
    /// Per-unit CRDT health data.
    pub units: Vec<CrdtUnitUpdate>,
}

/// Per-unit CRDT health update (~21 bytes).
#[derive(Serialize, Deserialize)]
pub struct CrdtUnitUpdate {
    /// Network entity ID.
    pub id: u32,
    /// CRDT damage counters per peer.
    pub damage: [f32; 2],
    /// CRDT healing counters per peer.
    pub healing: [f32; 2],
    /// Status effect flags (FIRE_EFFECT, FROST_EFFECT, ELECTRIC_EFFECT bits from UnitFlags).
    pub effects: u8,
}
