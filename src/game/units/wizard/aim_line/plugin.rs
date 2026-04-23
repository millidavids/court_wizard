use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::{spawn_aim_line, update_aim_line};

pub(in crate::game) struct AimLinePlugin;

impl Plugin for AimLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_aim_line, update_aim_line)
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
