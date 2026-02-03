use bevy::prelude::*;

use super::components::{MouseButtonState, MouseRightHeldThisFrame, SpellInputBlockedThisFrame};

/// Check if spell input is NOT blocked
pub fn spell_input_not_blocked(spell_blocked: Res<SpellInputBlockedThisFrame>) -> bool {
    !spell_blocked.blocked
}

/// Check if mouse left is NOT consumed
pub fn mouse_left_not_consumed(mouse_state: Res<MouseButtonState>) -> bool {
    !mouse_state.left_consumed
}

/// Check if right mouse button is NOT held
/// Used to prevent spell casting while right-clicking
pub fn mouse_right_not_held(mouse_right_held: Res<MouseRightHeldThisFrame>) -> bool {
    !mouse_right_held.held
}
