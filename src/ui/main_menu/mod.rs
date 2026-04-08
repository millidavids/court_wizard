//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Changelog, Instructions,
//! and WizardSelect screens.

mod background;
mod changelog;
mod credits;
mod game_mode_select;
mod landing;
mod multiplayer;
mod plugin;
mod roguelite_modifiers;
pub(crate) mod settings;
mod wizard_select;

pub use landing::constants::BACK_BUTTON_STYLE;
pub use plugin::MainMenuPlugin;
