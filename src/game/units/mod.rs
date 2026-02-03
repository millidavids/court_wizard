//! Units plugin module.
//!
//! Contains all game unit types: wizard, infantry, and archers.

use bevy::prelude::*;

pub(crate) mod archer;
pub(crate) mod components;
pub(crate) mod constants;
pub(super) mod infantry;
pub(super) mod king;
mod movement;
mod systems;
pub(crate) mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;

/// System set for calculating unit movement (acceleration, velocity).
/// All unit-specific movement calculations should be in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementCalculationSet;

/// System set for applying transforms based on calculated movement.
/// This set runs after MovementCalculationSet.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyTransformsSet;
