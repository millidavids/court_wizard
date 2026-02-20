//! Battlefield plugin module.
//!
//! Handles the battlefield ground plane, castle platform, and lighting.

pub(in crate::game) mod components;
mod plugin;
pub(in crate::game) mod styles;
pub(in crate::game) mod systems;

pub use plugin::BattlefieldPlugin;
