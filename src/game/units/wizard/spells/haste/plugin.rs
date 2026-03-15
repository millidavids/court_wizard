use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{ChainHasteSource, HasteSlowZone, MomentumBuff, MomentumPending};
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::systems::update_timed_modifier;

pub struct HastePlugin;

impl Plugin for HastePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // Local wizard casting (mouse input)
            systems::handle_haste_casting
                .run_if(spell_is_primed(Spell::Haste))
                .run_if(spell_input_not_blocked)
                .run_if(mouse_left_not_consumed)
                .run_if(mouse_held_or_wizard_casting)
                .run_if(is_spell_effects_active),
        );

        // Talent behavior systems
        app.add_systems(
            Update,
            systems::handle_haste_expiry
                .run_if(is_gameplay_running)
                .run_if(
                    any_with_component::<ChainHasteSource>
                        .or(any_with_component::<MomentumPending>),
                ),
        );

        app.add_systems(
            Update,
            update_timed_modifier::<MomentumBuff>
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<MomentumBuff>),
        );

        app.add_systems(
            Update,
            systems::tick_haste_slow_zone
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<HasteSlowZone>),
        );
    }
}
