//! Main menu module.
//!
//! Contains the MainMenuPlugin which aggregates Landing, Settings, Manual,
//! and other screens.

mod background;
mod landing;
mod multiplayer;
mod plugin;
pub(crate) mod settings;

pub use landing::constants::BACK_BUTTON_STYLE;
pub use plugin::MainMenuPlugin;
