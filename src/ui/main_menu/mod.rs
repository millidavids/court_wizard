//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Changelog, and Instructions screens.

mod changelog;
mod landing;
mod plugin;
pub(super) mod settings;

pub use plugin::MainMenuPlugin;
