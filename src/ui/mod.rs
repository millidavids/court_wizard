//! UI module for the game.
//!
//! This module provides the user interface systems and components,
//! organized by menu/screen type.

mod components;
mod game_over;
mod in_game;
mod main_menu;
mod pause_menu;
mod plugin;
mod spell_book;
mod styles;
mod systems;

pub use plugin::UiPlugin;
