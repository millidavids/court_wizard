//! Battlefield plugin module.
//!
//! Handles the battlefield ground plane, castle platform, and lighting.

pub(in crate::game) mod components;
pub(in crate::game) mod constants;
pub(in crate::game) mod flora;
mod plugin;
pub(in crate::game) mod systems;
pub(in crate::game) mod trampling;

pub use plugin::BattlefieldPlugin;
