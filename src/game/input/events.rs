//! Input events for game systems.
//!
//! All input events are centralized here and sent from the input plugin
//! to be consumed by other game systems.

use bevy::prelude::*;

/// Event fired when the left mouse button is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftPressed {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Event fired when the left mouse button is held down.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftHeld {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Event fired when the left mouse button is released.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftReleased;

/// Event fired when the spacebar is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarPressed;

/// Event fired when the spacebar is held down.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarHeld;

/// Event fired when the spacebar is released.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarReleased;
