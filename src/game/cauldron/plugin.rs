use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::brews::BrewsPlugin;
use super::messages::*;
use super::resources::CauldronBuffs;
use super::run_conditions::{cauldron_is_brewing, has_active_buffs};
use super::systems;

/// Plugin managing the cauldron brewing system.
pub struct CauldronPlugin;

impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CauldronBuffs>()
            .add_message::<StartBrewMessage>()
            .add_message::<BrewCompleteMessage>()
            .add_message::<CancelBrewMessage>()
            .add_plugins(BrewsPlugin)
            // Message handlers run across all InGame states so messages sent
            // from CauldronMenu aren't lost during the state transition.
            .add_systems(
                Update,
                (
                    systems::handle_start_brew.run_if(on_message::<StartBrewMessage>),
                    systems::handle_cancel_brew.run_if(on_message::<CancelBrewMessage>),
                )
                    .run_if(in_state(AppState::InGame)),
            )
            // Brewing loop and buff systems only run during gameplay.
            .add_systems(
                Update,
                (
                    systems::update_brew_timer.run_if(cauldron_is_brewing),
                    systems::handle_brew_complete.run_if(on_message::<BrewCompleteMessage>),
                    systems::tick_active_buffs.run_if(has_active_buffs),
                    systems::block_spells_during_brewing.run_if(cauldron_is_brewing),
                )
                    .chain()
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
