//! UI module for the game.
//!
//! This module provides the user interface systems and components,
//! organized by menu/screen type.

mod achievement_popup;
pub(crate) mod action_bar;
pub(crate) mod arcanorouter_display;
mod cauldron_menu;
mod compendium;
pub(crate) mod components;
mod concentration;
pub(crate) mod constants;
mod game_over;
mod in_game;
mod instructions;
mod loading;
pub(crate) mod main_menu;
mod pause_menu;
pub(crate) mod plugin;
pub(crate) mod roulette_display;
pub(crate) mod rune_display;
mod spell_book;
mod splash_screen;
mod styles;
pub(crate) mod systems;
pub(crate) mod tutorial;
mod version;
mod weather_bar;
mod wizard_tower;

pub use plugin::UiPlugin;
