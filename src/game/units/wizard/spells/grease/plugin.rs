use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{GreaseIndicator, GreaseZone};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct GreasePlugin;

impl Plugin for GreasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_grease_casting
                    .run_if(spell_is_primed(Spell::Grease))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_grease_indicator.run_if(any_exist::<GreaseIndicator>()),
                (
                    systems::apply_grease_slow,
                    systems::check_grease_ignition,
                    systems::update_grease_fire_spread,
                    systems::apply_grease_burn,
                    systems::fade_grease_zone,
                    systems::cleanup_grease_zone,
                )
                    .chain()
                    .run_if(any_exist::<GreaseZone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
