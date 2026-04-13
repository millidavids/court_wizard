use bevy::prelude::*;
use rand::Rng;

use super::resources::{GameRng, GameSeed};
use crate::config::GameConfig;

/// Initializes the GameSeed and GameRng resources from config or generates a random seed.
pub fn init_game_seed(mut commands: Commands, config: Res<GameConfig>) {
    let seed = config
        .seed
        .unwrap_or_else(|| rand::thread_rng().gen_range(0..u64::MAX));
    let game_seed = GameSeed(seed);
    commands.insert_resource(GameRng::new(&game_seed));
    commands.insert_resource(game_seed);
}
