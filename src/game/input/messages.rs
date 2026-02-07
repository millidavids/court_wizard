//! Input messages for game systems.
//!
//! All input messages are centralized here and sent from the input plugin
//! to be consumed by other game systems.

use bevy::prelude::*;

/// Message sent when the left mouse button is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftPressed {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Message sent when the left mouse button is held down.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftHeld {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Message sent when the left mouse button is released.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseLeftReleased;

/// Message sent when the right mouse button is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseRightPressed {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Message sent when the right mouse button is held down.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseRightHeld {
    /// Cursor position in window coordinates (if available).
    #[allow(dead_code)]
    pub cursor_position: Option<Vec2>,
}

/// Message sent when the right mouse button is released.
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseRightReleased;

/// Message sent when the spacebar is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarPressed;

/// Message sent when the spacebar is held down.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarHeld;

/// Message sent when the spacebar is released.
#[derive(Message, Debug, Clone, Copy)]
pub struct SpacebarReleased;

/// Message sent by UI systems to block spell input for one frame.
/// Prevents spells from casting when UI buttons are clicked.
#[derive(Message, Debug, Clone, Copy)]
pub struct BlockSpellInput;

/// Message sent when a UI button is clicked (mouse/touch pressed and released).
#[derive(Message, Debug, Clone, Copy)]
pub struct MouseClicked {
    /// The entity of the button that was clicked.
    pub button: Entity,
}

/// Message sent when a number key (1-10, using 0 for 10) is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActionBarKeyPressed {
    /// The action bar slot (0-9, where 0 represents the 10th slot).
    pub slot: u8,
}
