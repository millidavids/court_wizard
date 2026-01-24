//! UI module for the game.
//!
//! This module provides the user interface systems and components,
//! organized by menu/screen type.

pub mod components;
mod in_game;
mod main_menu;
mod pause_menu;
mod plugin;
mod spell_book;
pub mod styles;
pub mod systems;

pub use plugin::UiPlugin;
