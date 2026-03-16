use bevy::prelude::*;

use super::components::*;
use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

/// Plugin for the Raise The Dead spell.
pub struct RaiseTheDeadPlugin;

impl Plugin for RaiseTheDeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_raise_the_dead_casting
                    .run_if(spell_is_primed(Spell::RaiseTheDead))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                // Talent systems
                systems::tick_plague_bearer_aura
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<PlagueBearerAura>),
                systems::pull_corpses_to_cursor
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<CorpseMagnetActive>),
                systems::handle_undead_detonation
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<UndeadDetonation>),
                systems::handle_perpetual_unrest
                    .after(PostCombatSet)
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<PerpetualUnrest>),
                systems::tick_revenant_raise
                    .run_if(is_gameplay_running)
                    .run_if(any_with_component::<RevenantLord>),
            ),
        );
    }
}
