//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, spells, and archetypes.

mod aim_line;
pub(crate) mod archetypes;
pub(crate) mod components;
pub(in crate::game) mod constants;
pub(crate) mod messages;
mod plugin;
mod spell_range_indicator;
pub(crate) mod spells;
pub(in crate::game) mod systems;
pub(crate) mod talents;

pub use plugin::WizardPlugin;
