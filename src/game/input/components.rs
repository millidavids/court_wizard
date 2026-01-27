//! Components and resources for input tracking.

use bevy::prelude::*;

/// Tracks whether mouse button presses have been "consumed" by actions.
///
/// Prevents hold-through where completed actions immediately start new ones.
/// For example, prevents a completed spell cast from immediately starting
/// a new cast if the mouse is still held down.
#[derive(Resource, Default)]
pub struct MouseButtonState {
    /// True if current left button hold has been consumed by a completed action.
    ///
    /// Reset to false when:
    /// - Mouse button is released
    /// - State transitions occur
    ///
    /// Set to true when:
    /// - A spell cast completes
    /// - A channeling spell ends
    pub left_consumed: bool,
}
