use bevy::prelude::*;

use super::super::components::{CastingState, LocalWizard, PrimedSpell, Spell};
use crate::game::game_mode::components::LastCastSpell;
use crate::game::input::components::MouseLeftHeldThisFrame;

// Re-export commonly used run conditions for convenience
pub use crate::game::input::run_conditions::{
    mouse_left_not_consumed, mouse_right_not_held, spell_input_not_blocked,
};
pub use crate::game::run_conditions::any_exist;

/// Check if specific spell is primed on the local wizard.
///
/// When Spell Rotation is active (`LastCastSpell` resource exists), also blocks
/// the spell if it was the last one cast. LastCastSpell is only set when mana is
/// consumed (not when CastingState changes), so multi-step spells like Teleport
/// complete naturally before the block kicks in.
pub fn spell_is_primed(
    spell: Spell,
) -> impl Fn(Query<&PrimedSpell, With<LocalWizard>>, Option<Res<LastCastSpell>>) -> bool + Clone {
    move |local_query: Query<&PrimedSpell, With<LocalWizard>>,
          last_cast: Option<Res<LastCastSpell>>| {
        local_query
            .single()
            .map(|primed| {
                if primed.spell != spell {
                    return false;
                }
                // Spell Rotation: block if this spell was the last one cast
                if let Some(ref lc) = last_cast {
                    if !lc.bypass_until_cast && lc.spell == Some(spell) {
                        return false;
                    }
                }
                true
            })
            .unwrap_or(false)
    }
}

/// Check if the local wizard is currently casting or channeling.
pub fn wizard_is_casting_or_channeling(
    local_query: Query<&CastingState, With<LocalWizard>>,
) -> bool {
    local_query
        .single()
        .map(|state| !matches!(state, CastingState::Resting))
        .unwrap_or(false)
}

/// Check if local mouse is held OR local wizard is currently casting/channeling.
/// This ensures the local system runs both during active casting and when releasing to cancel.
pub fn mouse_held_or_wizard_casting(
    mouse_held: Res<MouseLeftHeldThisFrame>,
    local_query: Query<&CastingState, With<LocalWizard>>,
) -> bool {
    mouse_held.held || wizard_is_casting_or_channeling(local_query)
}
