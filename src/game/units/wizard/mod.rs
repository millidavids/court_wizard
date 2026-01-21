//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, and spells.

pub mod components;
mod constants;
mod plugin;
pub mod spells;
mod styles;
mod systems;

pub use plugin::WizardPlugin;
