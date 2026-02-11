//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, spells, and archetypes.

pub(crate) mod archetypes;
pub(crate) mod components;
mod constants;
pub(crate) mod messages;
mod plugin;
mod spell_range_indicator;
pub(in crate::game) mod spells;
mod styles;
pub(in crate::game) mod systems;

pub use plugin::WizardPlugin;
