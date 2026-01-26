use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::*;

pub struct FingerOfDeathPlugin;

impl Plugin for FingerOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_finger_of_death_casting,
                apply_finger_of_death_damage,
                update_finger_of_death_beam_visuals,
                cleanup_finger_of_death_beams,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
