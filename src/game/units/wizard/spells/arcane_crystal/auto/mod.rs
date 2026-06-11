//! Auto-cast and talent burst handlers.

mod auto_cast;
mod network;
mod spawn_helpers;
mod spell_variants;
mod talents;
mod turret;

// Re-export the public surface that the original auto.rs exposed.
// All items were `pub(super)` in auto.rs (visible to arcane_crystal/),
// so we re-export them at the same visibility level here.
pub(super) use auto_cast::auto_cast_remembered_spell;
pub(super) use network::crystal_network_chain;
pub(super) use spawn_helpers::{spawn_crystal_disintegrate_beam, spawn_crystal_mini_missile};
pub(super) use talents::{crystal_aoe_burst, resonance_cascade_burst};
pub(super) use turret::auto_crystal_fire;
