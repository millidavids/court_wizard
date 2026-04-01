use bevy::prelude::*;

use super::systems;
use crate::state::AppState;

pub struct SeededRngPlugin;

impl Plugin for SeededRngPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), systems::init_game_seed);
    }
}
