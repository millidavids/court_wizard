//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, spells, and archetypes.

pub(crate) mod archetypes;
pub(crate) mod components;
pub(crate) mod talents;
pub(in crate::game) mod constants;
pub(crate) mod messages;
mod plugin;
mod spell_range_indicator;
pub(crate) mod spells;
pub(in crate::game) mod styles;
pub(in crate::game) mod systems;

pub use plugin::WizardPlugin;
