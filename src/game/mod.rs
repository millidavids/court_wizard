//! Game module containing gameplay logic for the wizard tower defense game.
//!
//! This module implements the core gameplay, including:
//! - Battlefield and castle setup
//! - Wizard entity
//! - Defender and attacker unit spawning
//! - Unit movement and targeting
//! - Simple collision-based combat

mod attacker;
mod battlefield;
mod components;
mod defender;
mod plugin;
mod shared_systems;
mod wizard;

pub use plugin::GamePlugin;
