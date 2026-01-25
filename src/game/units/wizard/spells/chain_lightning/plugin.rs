use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::*;

pub struct ChainLightningPlugin;

impl Plugin for ChainLightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chain_lightning_casting,
                process_chain_lightning_bounces,
                update_chain_lightning_arcs,
                cleanup_chain_lightning,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
