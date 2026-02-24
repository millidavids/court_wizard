//! CRDT types for multiplayer state convergence.
//!
//! With exactly 2 peers, CRDTs are trivially simple: fixed-size `[f32; 2]` arrays
//! where each peer writes to their own slot and merge uses `max` per slot.
//!
//! - Peer 0 = host, Peer 1 = guest
//! - Merge = element-wise max (commutative, associative, idempotent)
//! - Both melee combat (host-only) and spells (both peers) write damage

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identifies which peer this client is: 0 = host, 1 = guest.
///
/// Used as the index into CRDT arrays when writing local damage/healing.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PeerId(pub usize);

impl PeerId {
    pub const HOST: usize = 0;
    pub const GUEST: usize = 1;
}

/// CRDT-based health tracking with per-peer damage and healing counters.
///
/// Each peer monotonically increments their own damage/healing slot.
/// Current HP is derived: `(max_hp + sum(healing) - sum(damage)).clamp(0, max_hp)`.
/// Merge takes the `max` of each slot — commutative, associative, idempotent.
///
/// ~20 bytes per unit when serialized.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CrdtHealth {
    /// Maximum HP for this unit.
    pub max_hp: f32,
    /// Monotonically increasing damage totals per peer.
    pub damage: [f32; 2],
    /// Monotonically increasing healing totals per peer.
    pub healing: [f32; 2],
}

impl CrdtHealth {
    /// Creates a new CrdtHealth with the given max HP and no damage/healing.
    pub fn new(max_hp: f32) -> Self {
        Self {
            max_hp,
            damage: [0.0; 2],
            healing: [0.0; 2],
        }
    }

    /// Returns the current HP derived from CRDT state.
    pub fn current_hp(&self) -> f32 {
        let total_damage: f32 = self.damage.iter().sum();
        let total_healing: f32 = self.healing.iter().sum();
        (self.max_hp + total_healing - total_damage).clamp(0.0, self.max_hp)
    }

    /// Returns true if the unit is dead (current HP <= 0).
    pub fn is_dead(&self) -> bool {
        self.current_hp() <= 0.0
    }

    /// Returns current HP as a 0-100 percentage.
    pub fn health_pct(&self) -> u8 {
        if self.max_hp > 0.0 {
            ((self.current_hp() / self.max_hp) * 100.0).clamp(0.0, 100.0) as u8
        } else {
            0
        }
    }

    /// Applies damage from the specified peer.
    ///
    /// Monotonically increases the damage counter for that peer.
    pub fn apply_damage(&mut self, peer: usize, amount: f32) {
        self.damage[peer] += amount;
    }

    /// Applies healing from the specified peer.
    ///
    /// Monotonically increases the healing counter for that peer.
    pub fn apply_healing(&mut self, peer: usize, amount: f32) {
        self.healing[peer] += amount;
    }

    /// Merges remote CRDT state into this one.
    ///
    /// Takes the element-wise max of each slot — the fundamental CRDT merge operation.
    pub fn merge(&mut self, remote: &CrdtHealth) {
        for i in 0..2 {
            self.damage[i] = self.damage[i].max(remote.damage[i]);
            self.healing[i] = self.healing[i].max(remote.healing[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_health() {
        let h = CrdtHealth::new(100.0);
        assert_eq!(h.current_hp(), 100.0);
        assert!(!h.is_dead());
        assert_eq!(h.health_pct(), 100);
    }

    #[test]
    fn test_damage_single_peer() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_damage(0, 30.0);
        assert_eq!(h.current_hp(), 70.0);
    }

    #[test]
    fn test_damage_both_peers() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_damage(0, 10.0);
        h.apply_damage(1, 15.0);
        assert_eq!(h.current_hp(), 75.0);
    }

    #[test]
    fn test_damage_clamp_to_zero() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_damage(0, 150.0);
        assert_eq!(h.current_hp(), 0.0);
        assert!(h.is_dead());
    }

    #[test]
    fn test_healing() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_damage(0, 50.0);
        h.apply_healing(1, 20.0);
        assert_eq!(h.current_hp(), 70.0);
    }

    #[test]
    fn test_healing_clamp_to_max() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_healing(0, 50.0);
        assert_eq!(h.current_hp(), 100.0); // Can't exceed max
    }

    #[test]
    fn test_merge_takes_max() {
        let mut local = CrdtHealth::new(100.0);
        local.apply_damage(0, 10.0);
        local.apply_damage(1, 5.0);

        let mut remote = CrdtHealth::new(100.0);
        remote.apply_damage(0, 8.0); // Less than local
        remote.apply_damage(1, 15.0); // More than local

        local.merge(&remote);
        assert_eq!(local.damage[0], 10.0); // Kept local (higher)
        assert_eq!(local.damage[1], 15.0); // Took remote (higher)
        assert_eq!(local.current_hp(), 75.0);
    }

    #[test]
    fn test_merge_idempotent() {
        let mut h = CrdtHealth::new(100.0);
        h.apply_damage(0, 10.0);
        let copy = h.clone();
        h.merge(&copy);
        assert_eq!(h.current_hp(), 90.0); // Unchanged
    }

    #[test]
    fn test_merge_commutative() {
        let mut a = CrdtHealth::new(100.0);
        a.apply_damage(0, 10.0);
        a.apply_damage(1, 5.0);

        let mut b = CrdtHealth::new(100.0);
        b.apply_damage(0, 8.0);
        b.apply_damage(1, 15.0);

        let mut a_then_b = a.clone();
        a_then_b.merge(&b);

        let mut b_then_a = b.clone();
        b_then_a.merge(&a);

        assert_eq!(a_then_b.current_hp(), b_then_a.current_hp());
    }
}
