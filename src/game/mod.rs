//! Game module containing gameplay logic for the wizard tower defense game.
//!
//! This module implements the core gameplay, including:
//! - Battlefield and castle setup
//! - Wizard entity
//! - Defender and attacker unit spawning
//! - Unit movement and targeting
//! - Simple collision-based combat

mod battlefield;
pub mod components;
pub mod constants;
mod plugin;
mod shared_systems;
pub mod units;

pub use plugin::GamePlugin;
