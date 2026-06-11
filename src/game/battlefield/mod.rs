//! Battlefield plugin module.
//!
//! Handles the battlefield ground plane, castle platform, and lighting.

pub(in crate::game) mod components;
pub(in crate::game) mod constants;
pub(crate) mod ground_material;
mod plugin;
pub(in crate::game) mod systems;
pub(in crate::game) mod trampling;

pub use plugin::BattlefieldPlugin;
pub(crate) use systems::load_battlefield_assets;
