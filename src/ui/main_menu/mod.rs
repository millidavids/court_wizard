//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Changelog, Instructions,
//! WizardSelect, and SaveSelect screens.

mod changelog;
mod landing;
mod plugin;
mod save_select;
pub(super) mod settings;
mod wizard_select;

pub use plugin::MainMenuPlugin;
