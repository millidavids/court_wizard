//! Units plugin module.
//!
//! Contains all game unit types: wizard, infantry, and archers.

pub(crate) mod aerialist;
pub(crate) mod animation;
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
pub(crate) mod king;
pub(in crate::game) mod movement;
pub(in crate::game) mod ranged_bolt;
mod sets;
pub(crate) mod shielder;
mod spawning;
pub(crate) mod status_effects;
pub(in crate::game) mod systems;
pub(in crate::game) mod teleporter;
pub(crate) mod undead;
pub(in crate::game) mod unit_type;
pub(crate) mod wizard;

mod plugin;

pub use commander::Commander;
pub use components::UnitType;
pub use damage::DamageType;
pub use elite::EliteHealthBonus;
pub use plugin::UnitsPlugin;
pub use sets::{ApplyTransformsSet, MovementCalculationSet};
pub(crate) use spawning::random_position_in_cell;
