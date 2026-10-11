use bevy::prelude::*;

use super::resources::{insert_game_seed, random_seed};
use crate::config::GameConfig;

/// Initializes the GameSeed and GameRng resources from config or generates a random seed.
pub(super) fn init_game_seed(mut commands: Commands, config: Res<GameConfig>) {
    insert_game_seed(&mut commands, config.seed.unwrap_or_else(random_seed));
}
