use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::game::run_conditions::is_gameplay_running;
use crate::state::AppState;

use super::messages::*;
use super::resources::CauldronBuffs;
use super::run_conditions::{
    cauldron_is_brewing, has_active_buffs, has_brew_bubbles, has_brewing_effects,
    needs_buff_cleanup,
};
use super::systems;

/// Plugin managing the cauldron brewing system.
pub struct CauldronPlugin;

impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CauldronBuffs>()
            .add_message::<StartBrewMessage>()
            .add_message::<BrewCompleteMessage>()
            .add_message::<CancelBrewMessage>()
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
            // Brewing loop, animation, and buff systems only run during gameplay.
            .add_systems(
                Update,
                (
                    systems::start_brewing_effects,
                    systems::update_brewing_timer.run_if(has_brewing_effects),
                    systems::update_cauldron_animation,
                    systems::update_brewing_effects.run_if(has_brewing_effects),
                    systems::update_brew_timer.run_if(cauldron_is_brewing),
                    systems::handle_brew_complete.run_if(on_message::<BrewCompleteMessage>),
                    systems::tick_active_buffs.run_if(has_active_buffs),
                    systems::heal_defenders.run_if(has_active_buffs),
                    systems::buff_defender_damage.run_if(has_active_buffs),
                    systems::buff_defender_resistance.run_if(has_active_buffs),
                    systems::apply_cauldron_speed_modifiers.run_if(has_active_buffs),
                    systems::shield_defenders.run_if(has_active_buffs),
                    systems::cleanup_cauldron_buff_components.run_if(needs_buff_cleanup),
                    systems::block_spells_during_brewing.run_if(cauldron_is_brewing),
                    systems::update_brew_bubble.run_if(has_brew_bubbles),
                )
                    .chain()
                    .run_if(is_gameplay_running),
            );
    }
}
