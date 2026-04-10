//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Changelog, Instructions,
//! and WizardSelect screens.

mod background;
mod changelog;
mod credits;
mod landing;
mod multiplayer;
mod plugin;
pub(crate) mod settings;

pub use landing::constants::BACK_BUTTON_STYLE;
pub use plugin::MainMenuPlugin;
