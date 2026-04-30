//! Game module containing gameplay logic for the wizard tower defense game.
//!
//! This module implements the core gameplay, including:
//! - Battlefield and castle setup
//! - Wizard entity
//! - Defender and attacker unit spawning
//! - Unit movement and targeting
//! - Simple collision-based combat

pub(crate) mod achievements;
mod battlefield;
#[cfg(feature = "benchmarking")]
mod benchmarking;
pub(crate) mod cauldron;
pub(crate) mod combat_systems;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod crt_effect;
pub(crate) mod drops;
pub(crate) mod game_mode;
pub(crate) mod input;
pub(crate) mod insight_bonuses;
mod loading;
pub(crate) mod messages;
pub(crate) mod movement_systems;
pub(crate) mod multiplayer;
pub(crate) mod pathfinding;
mod plugin;
pub(crate) mod resources;
pub(crate) mod run_conditions;
pub(crate) mod seeded_rng;
mod sets;
pub(crate) mod shared_systems;
pub(in crate::game) mod systems;
pub(crate) mod terrain;
pub(crate) mod units;
mod wave_systems;
mod win_lose_systems;

pub use plugin::GamePlugin;
