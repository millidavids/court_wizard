use bevy::prelude::*;
use rand::Rng;

use super::resources::GameSeed;
use crate::config::GameConfig;

/// Initializes the GameSeed resource from config or generates a random one.
pub fn init_game_seed(mut commands: Commands, config: Res<GameConfig>) {
    let seed = config.seed.unwrap_or_else(|| rand::thread_rng().gen_range(0..u64::MAX));
    commands.insert_resource(GameSeed(seed));
}
