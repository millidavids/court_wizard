//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, and spells.

pub(crate) mod components;
mod constants;
mod plugin;
mod spell_range_indicator;
pub(in crate::game) mod spells;
mod styles;
pub(super) mod systems;

pub use plugin::WizardPlugin;
