use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::plugin::VelocitySystemSet;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::components::{NarcolepticWave, NightTerrors, SleepModifier, Sleepwalking};

pub struct SleepPlugin;

impl Plugin for SleepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_sleep_casting
                    .run_if(spell_is_primed(Spell::Sleep))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                // Base sleep timer tick + cleanup
                systems::update_sleep_modifiers
                    .run_if(any_with_component::<SleepModifier>)
                    .run_if(is_spell_effects_active),
                // Night Terrors: DPS to sleeping units
                systems::update_night_terrors
                    .run_if(any_with_component::<NightTerrors>)
                    .run_if(is_spell_effects_active),
                // Narcoleptic Wave: spread sleep after delay
                systems::update_narcoleptic_wave
                    .run_if(any_with_component::<NarcolepticWave>)
                    .run_if(is_spell_effects_active),
                // Dreamwalker: override targeting velocity for sleepwalking units.
                // Runs after VelocitySystemSet so it overrides normal targeting.
                systems::update_sleepwalkers
                    .after(VelocitySystemSet)
                    .run_if(any_with_component::<Sleepwalking>)
                    .run_if(is_spell_effects_active),
            ),
        );
    }
}
