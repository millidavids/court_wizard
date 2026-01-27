use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems::*;
use crate::state::InGameState;

pub struct ChainLightningPlugin;

impl Plugin for ChainLightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chain_lightning_casting
                    .run_if(spell_is_primed(Spell::ChainLightning))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                process_chain_lightning_bounces,
                update_chain_lightning_arcs,
                cleanup_chain_lightning,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
