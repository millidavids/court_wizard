//! Game module containing gameplay logic for the wizard tower defense game.
//!
//! This module implements the core gameplay, including:
//! - Battlefield and castle setup
//! - Wizard entity
//! - Defender and attacker unit spawning
//! - Unit movement and targeting
//! - Simple collision-based combat

mod battlefield;
pub(crate) mod cauldron;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod input;
mod loading;
pub(crate) mod pathfinding;
mod plugin;
pub(crate) mod resources;
pub(crate) mod run_conditions;
pub(crate) mod shared_systems;
mod systems;
pub(crate) mod units;
mod win_lose_systems;

pub use plugin::GamePlugin;
