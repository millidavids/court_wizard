use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{PlagueWindCloud, PlagueWindIndicator};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct PlagueWindPlugin;

impl Plugin for PlagueWindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_plague_wind_casting
                    .run_if(spell_is_primed(Spell::PlagueWind))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_plague_wind_indicator.run_if(any_exist::<PlagueWindIndicator>()),
                (
                    systems::move_plague_wind_cloud,
                    systems::apply_plague_wind_damage,
                    systems::fade_plague_wind_cloud,
                    systems::cleanup_plague_wind_cloud,
                )
                    .chain()
                    .run_if(any_exist::<PlagueWindCloud>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
