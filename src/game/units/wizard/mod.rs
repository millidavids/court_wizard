//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, and spells.

pub mod components;
mod constants;
mod plugin;
mod spell_range_indicator;
pub mod spells;
mod styles;
pub mod systems;

pub use plugin::WizardPlugin;
