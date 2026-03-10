use bevy::prelude::*;

/// Marker component for the action bar root container.
#[derive(Component)]
pub(crate) struct ActionBarRoot;

/// Component that marks an action bar slot button and stores its slot index.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ActionBarSlot {
    /// The slot index (0-9, where 0 represents key 1, and 9 represents key 0).
    pub(super) slot: u8,
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

/// Resource that tracks whether infinite mana is enabled.
#[derive(Resource, Default)]
pub struct InfiniteMana(pub bool);
