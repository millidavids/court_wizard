use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::SleepIndicator;
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

pub struct SleepPlugin;

impl Plugin for SleepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_sleep_casting
                    .run_if(spell_is_primed(Spell::Sleep))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_sleep_indicator.run_if(any_exist::<SleepIndicator>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
