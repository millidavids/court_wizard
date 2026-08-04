//! Wizard plugin module.
//!
//! Handles the wizard entity, castle setup, spells, and archetypes.

mod aim_line;
pub(crate) mod archetypes;
pub(crate) mod components;
pub(in crate::game) mod constants;
pub(crate) mod messages;
mod plugin;
mod primed_spell_indicator;
mod spell_enum;
mod spell_range_indicator;
mod spell_status_effects;
pub(crate) mod spells;
pub(in crate::game) mod systems;
pub(crate) mod talents;
mod wizard_state;

pub use plugin::WizardPlugin;
