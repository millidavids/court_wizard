//! UI module for the game.
//!
//! This module provides the user interface systems and components,
//! organized by menu/screen type.

pub(crate) mod action_bar;
pub(crate) mod arcanorouter_display;
pub(crate) mod button_systems;
mod cauldron_menu;
mod color_utils;
mod compendium;
pub(crate) mod components;
mod concentration;
pub(crate) mod constants;
pub(crate) mod focus;
mod game_over;
pub(crate) mod gamepad_glyphs;
mod in_game;
pub(crate) mod layout_helpers;
pub(crate) mod link_button;
mod loading;
pub(crate) mod main_menu;
pub(crate) mod manual;
pub(crate) mod markdown;
pub(crate) mod notification;
mod pause_menu;
pub(crate) mod plugin;
pub(crate) mod roulette_display;
pub(crate) mod rune_display;
mod scale;
mod spell_book;
mod splash_screen;
pub(crate) mod systems;
pub(crate) mod tutorial;
mod version;
pub(crate) mod weather_bar;
pub(crate) mod wizard_tower;

pub(crate) use game_over::accumulate_mode_level_stats;
pub use plugin::UiPlugin;
