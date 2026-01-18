//! Game module containing gameplay logic for the wizard tower defense game.
//!
//! This module implements the core gameplay, including:
//! - Battlefield and castle setup
//! - Wizard entity
//! - Defender and attacker unit spawning
//! - Unit movement and targeting
//! - Simple collision-based combat

pub mod attacker;
pub mod battlefield;
mod components;
pub mod defender;
mod plugin;
mod shared_systems;
pub mod wizard;

pub use plugin::GamePlugin;
