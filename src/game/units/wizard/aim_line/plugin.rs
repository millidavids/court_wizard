use bevy::prelude::*;

use crate::state::{InGameState, MultiplayerGameState};

use super::systems::{spawn_aim_line, update_aim_line};

pub(in crate::game) struct AimLinePlugin;

impl Plugin for AimLinePlugin {
    fn build(&self, app: &mut App) {
        // Aim line is purely local-input visual — both MP peers see their own
        // wizard's aim line for their local cursor. Strictly gated on
        // `Running` substates so the 3D mesh doesn't render behind the
        // spell-book / pause / cauldron overlays (the original SP gate was
        // also strict on `InGameState::Running`).
        app.add_systems(
            Update,
            (spawn_aim_line, update_aim_line).chain().run_if(
                in_state(InGameState::Running).or_else(in_state(MultiplayerGameState::Running)),
            ),
        );
    }
}
