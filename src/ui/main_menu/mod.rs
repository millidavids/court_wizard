//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Changelog, Instructions,
//! and WizardSelect screens.

mod changelog;
mod credits;
mod landing;
mod multiplayer;
mod plugin;
pub(crate) mod settings;
mod wizard_select;
mod wizard_select_shared;

pub use plugin::MainMenuPlugin;
