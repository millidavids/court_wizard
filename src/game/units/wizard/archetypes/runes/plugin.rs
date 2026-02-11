use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::InGameState;

use super::messages::*;
use super::resources::{LastActivatedSpell, RuneSequence};
use super::systems;

/// Plugin managing the rune system. Only active for the RuneCaster archetype.
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
                    .run_if(in_state(InGameState::Running))
                    .run_if(run_conditions::is_rune_caster),
            );
    }
}
