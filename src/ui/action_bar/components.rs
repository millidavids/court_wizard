use bevy::prelude::*;

/// Marker component for the action bar root container. There is only one
/// action bar — its slots reorganize between linear (bottom-left row, mouse
/// mode) and radial (ring near the wizard, gamepad mode) layouts via an
/// animated transition rather than swapping between two different roots.
#[derive(Component)]
pub(crate) struct ActionBarRoot;

/// Component that marks an action bar slot button and stores its slot index.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ActionBarSlot {
    /// The slot index (0-9, where 0 represents key 1, and 9 represents key 0).
    pub(crate) slot: u8,
}

/// Marker component for the spell name text within an action bar slot.
#[derive(Component)]
pub(super) struct ActionBarSlotText {
    /// The slot index this text belongs to.
    pub(super) slot: u8,
}

/// Marker component for the hotkey indicator text within an action bar slot.
#[derive(Component)]
pub(super) struct ActionBarHotkeyText;

/// Marker component for the spell icon image within an action bar slot.
#[derive(Component)]
pub(super) struct ActionBarSlotIcon {
    /// The slot index this icon belongs to.
    pub(super) slot: u8,
}

/// Marker component for the debug infinite mana toggle button.
#[derive(Component)]
pub(super) struct DebugManaButton;

/// Marker for action bar slots currently highlighted by keyboard input.
#[derive(Component)]
pub(super) struct KeyboardHighlighted;

/// Short-lived visual pulse applied to a slot when the player commits it via
/// right-stick + RT. Decays over `RADIAL_COMMIT_FLASH_SECS`, painting the
/// button yellow, then removes itself and restores the default background.
#[derive(Component)]
pub(super) struct RadialCommitFlash {
    pub(super) remaining: f32,
}

/// Progress of the layout morph from linear (0.0) to radial (1.0). The
/// `animate_action_bar_layout` system ticks this toward the target implied by
/// `ActiveInputDevice` each frame and applies the resulting per-slot
/// position via absolute `Node` coordinates.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub(super) struct ActionBarLayoutProgress(pub f32);

/// Resource that tracks whether infinite mana is enabled.
#[derive(Resource, Default)]
pub struct InfiniteMana(pub bool);
