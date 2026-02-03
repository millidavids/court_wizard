use bevy::prelude::*;

use super::super::components::{CastingState, PrimedSpell, Spell, Wizard};
use crate::game::input::components::MouseLeftHeldThisFrame;

// Re-export commonly used run conditions for convenience
pub use crate::game::input::run_conditions::{
    mouse_left_not_consumed, mouse_right_not_held, spell_input_not_blocked,
};
pub use crate::game::run_conditions::any_exist;

/// Check if specific spell is primed
pub fn spell_is_primed(spell: Spell) -> impl Fn(Query<&PrimedSpell, With<Wizard>>) -> bool + Clone {
    move |primed_spell_query: Query<&PrimedSpell, With<Wizard>>| {
        primed_spell_query
            .single()
            .map(|primed| primed.spell == spell)
            .unwrap_or(false)
    }
}

/// Check if wizard is currently casting or channeling
/// This allows the system to run even when mouse is released, to handle cancellation
pub fn wizard_is_casting_or_channeling(wizard_query: Query<&CastingState, With<Wizard>>) -> bool {
    wizard_query
        .single()
        .map(|state| !matches!(state, CastingState::Resting))
        .unwrap_or(false)
}

/// Check if mouse is held OR wizard is currently casting/channeling
/// This ensures the system runs both during active casting and when releasing to cancel
pub fn mouse_held_or_wizard_casting(
    mouse_held: Res<MouseLeftHeldThisFrame>,
    wizard_query: Query<&CastingState, With<Wizard>>,
) -> bool {
    mouse_held.held || wizard_is_casting_or_channeling(wizard_query)
}
