use bevy::prelude::*;

use super::super::components::{CastingState, GuestWizard, LocalWizard, PrimedSpell, Spell};
use crate::game::input::components::MouseLeftHeldThisFrame;
use crate::game::multiplayer::spell_commands::GuestInputState;

// Re-export commonly used run conditions for convenience
pub use crate::game::input::run_conditions::{
    mouse_left_not_consumed, mouse_right_not_held, spell_input_not_blocked,
};
pub use crate::game::run_conditions::any_exist;

/// Check if specific spell is primed on the local wizard OR the guest wizard.
pub fn spell_is_primed(
    spell: Spell,
) -> impl Fn(
    Query<&PrimedSpell, With<LocalWizard>>,
    Query<&PrimedSpell, With<GuestWizard>>,
) -> bool
       + Clone {
    move |local_query: Query<&PrimedSpell, With<LocalWizard>>,
          guest_query: Query<&PrimedSpell, With<GuestWizard>>| {
        let local_primed = local_query
            .single()
            .map(|primed| primed.spell == spell)
            .unwrap_or(false);
        let guest_primed = guest_query
            .single()
            .map(|primed| primed.spell == spell)
            .unwrap_or(false);
        local_primed || guest_primed
    }
}

/// Check if the local wizard OR guest wizard is currently casting or channeling.
/// This allows the system to run even when mouse is released, to handle cancellation.
pub fn wizard_is_casting_or_channeling(
    local_query: Query<&CastingState, With<LocalWizard>>,
    guest_query: Query<&CastingState, (With<GuestWizard>, Without<LocalWizard>)>,
) -> bool {
    let local_casting = local_query
        .single()
        .map(|state| !matches!(state, CastingState::Resting))
        .unwrap_or(false);
    let guest_casting = guest_query
        .single()
        .map(|state| !matches!(state, CastingState::Resting))
        .unwrap_or(false);
    local_casting || guest_casting
}

/// Check if mouse is held OR any wizard is currently casting/channeling, OR
/// the guest has input this frame.
/// This ensures the system runs both during active casting and when releasing to cancel.
pub fn mouse_held_or_wizard_casting(
    mouse_held: Res<MouseLeftHeldThisFrame>,
    guest_input: Option<Res<GuestInputState>>,
    local_query: Query<&CastingState, With<LocalWizard>>,
    guest_query: Query<&CastingState, (With<GuestWizard>, Without<LocalWizard>)>,
) -> bool {
    let guest_active = guest_input
        .as_ref()
        .is_some_and(|i| i.pressed || i.just_pressed || i.just_released);
    mouse_held.held || guest_active || wizard_is_casting_or_channeling(local_query, guest_query)
}
