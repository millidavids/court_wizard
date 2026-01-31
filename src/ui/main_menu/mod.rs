//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, and Changelog screens.

mod changelog;
mod landing;
mod plugin;
pub mod settings;

pub use plugin::MainMenuPlugin;
