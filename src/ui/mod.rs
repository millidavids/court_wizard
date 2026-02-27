//! UI module for the game.
//!
//! This module provides the user interface systems and components,
//! organized by menu/screen type.

mod achievement_popup;
mod action_bar;
mod arcanorouter_display;
mod cauldron_menu;
pub(crate) mod components;
mod concentration;
mod game_over;
mod in_game;
mod instructions;
mod loading;
pub(crate) mod main_menu;
mod pause_menu;
pub(crate) mod plugin;
pub(crate) mod progress;
mod roulette_display;
mod rune_display;
mod splash_screen;
mod spell_book;
mod styles;
pub(crate) mod systems;
mod version;
mod wizard_tower;

pub use plugin::UiPlugin;
