//! Gamepad input resources.

use bevy::prelude::*;

use super::constants::{
    DEFAULT_DEADZONE, DEFAULT_RESPONSE_CURVE, DEFAULT_SENSITIVITY_X, DEFAULT_SENSITIVITY_Y,
    TRIGGER_THRESHOLD,
};

/// Which input device the player has most recently used.
///
/// Drives OS cursor visibility, button-glyph style, and the focus
/// navigation system's auto-activation. Updated every frame by
/// `detect_active_input_device`.
///
/// `SteamInputPad` is a controller whose physical device is captured by Steam
/// Input (so there is no gilrs `Entity` for it); its handle and details live in
/// [`crate::game::input::action_state::GamepadActionState`]. `Gamepad(Entity)` is
/// a controller read through the gilrs fallback path.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum ActiveInputDevice {
    #[default]
    MouseKeyboard,
    Gamepad(Entity),
    SteamInputPad,
}

impl ActiveInputDevice {
    /// True for any controller (Steam Input or gilrs) — drives the "controller
    /// is active" UI layer (radial action bar, glyph prompts, focus navigation).
    pub fn is_gamepad(&self) -> bool {
        matches!(self, Self::Gamepad(_) | Self::SteamInputPad)
    }

    /// The gilrs `Gamepad` entity, only on the fallback path (`None` for a
    /// Steam-captured pad or mouse/keyboard).
    pub fn gamepad_entity(&self) -> Option<Entity> {
        match self {
            Self::Gamepad(e) => Some(*e),
            Self::MouseKeyboard | Self::SteamInputPad => None,
        }
    }
}

/// Screen-space virtual cursor position, driven by the left stick when a
/// gamepad is the active input device.
///
/// The `update_virtual_cursor` system writes this into `CorrectedCursorPosition`
/// each frame so all downstream spell/UI systems read the virtual cursor
/// transparently.
#[derive(Resource, Debug, Default)]
pub(crate) struct VirtualCursorPosition {
    /// Current position in logical window pixels.
    pub screen_pos: Vec2,
}

/// Player-configurable gamepad aiming tuning.
#[derive(Resource, Debug, Clone)]
pub(crate) struct GamepadAimSettings {
    pub deadzone: f32,
    pub sensitivity: Vec2,
    pub response_curve: f32,
    pub trigger_threshold: f32,
}

impl Default for GamepadAimSettings {
    fn default() -> Self {
        Self {
            deadzone: DEFAULT_DEADZONE,
            sensitivity: Vec2::new(DEFAULT_SENSITIVITY_X, DEFAULT_SENSITIVITY_Y),
            response_curve: DEFAULT_RESPONSE_CURVE,
            trigger_threshold: TRIGGER_THRESHOLD,
        }
    }
}

/// Last hovered radial slot (set by the right-stick angle each frame).
#[derive(Resource, Debug, Default)]
pub(crate) struct RadialHoveredSlot(pub Option<u8>);
