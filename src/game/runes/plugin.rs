use bevy::prelude::*;

use crate::state::InGameState;

use super::events::*;
use super::resources::{LastActivatedSpell, RuneSequence};
use super::systems;

/// Plugin managing the rune system.
pub struct RunePlugin;

impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RuneSequence>()
            .init_resource::<LastActivatedSpell>()
            .add_message::<RunePressed>()
            .add_message::<ActivateRuneSequence>()
            .add_message::<RuneSpellActivated>()
            .add_systems(
                Update,
                (
                    systems::handle_rune_pressed,
                    systems::handle_rune_activation,
                    systems::update_rune_timeout,
                )
                    .chain()
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
