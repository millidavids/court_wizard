use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

pub struct ElitePlugin;

impl Plugin for ElitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::apply_elite_health_bonus,
                systems::remove_elite_health_bonus,
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
