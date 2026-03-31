//! Units plugin module.
//!
//! Contains all game unit types: wizard, infantry, and archers.

use bevy::prelude::*;
use rand::Rng;

pub(crate) mod aerialist;
pub(crate) mod archer;
pub(crate) mod assassin;
pub(crate) mod boss;
pub(crate) mod brute;
pub(in crate::game) mod commander;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod damage;
pub(crate) mod dispeller;
pub(in crate::game) mod elite;
pub(crate) mod healer;
pub(in crate::game) mod infantry;
pub(crate) mod shielder;
pub(crate) mod king;
pub(crate) mod undead;
pub(in crate::game) mod movement;
pub(in crate::game) mod systems;
pub(crate) mod wizard;

mod plugin;

pub use components::UnitType;
pub use damage::DamageType;
pub use plugin::UnitsPlugin;

// Re-export commander types for future use
#[allow(unused_imports)]
pub use commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};

// Re-export elite types for future use
#[allow(unused_imports)]
pub use elite::{EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};

/// System set for calculating unit movement (acceleration, velocity).
/// All unit-specific movement calculations should be in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementCalculationSet;

/// System set for applying transforms based on calculated movement.
/// This set runs after MovementCalculationSet.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyTransformsSet;

/// Generates a random position within a tight spread around a cell center.
/// Used for spawning units randomly within their assigned grid cell.
/// Returns (x, z) coordinates offset from the center point.
pub(crate) fn random_position_in_cell(cell_x: f32, cell_z: f32) -> (f32, f32) {
    use crate::game::constants::GRID_ROW_DEPTH;
    let mut rng = rand::thread_rng();
    let spread = GRID_ROW_DEPTH / 8.0;
    let final_x = cell_x + rng.gen_range(-spread..spread);
    let final_z = cell_z + rng.gen_range(-spread..spread);
    (final_x, final_z)
}
