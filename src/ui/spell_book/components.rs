use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Actions that can be triggered by spell book buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpellBookButtonAction {
    /// Preview a spell in the detail panel (click in right list).
    SelectSpell(Spell),
    /// Prime the selected spell and return to gameplay.
    CastSpell,
    /// Close the spell book without priming.
    Close,
}

/// Marker component for entities that should be cleaned up when exiting spell book.
#[derive(Component)]
pub(super) struct OnSpellBookScreen;

/// Marker for the vertically scrollable spell list container.
#[derive(Component)]
pub(super) struct ScrollableSpellList;

/// Tracks which spell is currently shown in the detail panel.
#[derive(Resource)]
pub(super) struct SelectedSpellPreview(pub Spell);

/// Marker for the spell name text in the detail panel.
#[derive(Component)]
pub(super) struct DetailName;

/// Marker for the damage type text in the detail panel.
#[derive(Component)]
pub(super) struct DetailDamageType;

/// Marker for the description text in the detail panel.
#[derive(Component)]
pub(super) struct DetailDescription;

/// Marker for the instructions text in the detail panel.
#[derive(Component)]
pub(super) struct DetailInstructions;

/// Marks a hotkey slot button (0-4 corresponding to keys 1-5).
#[derive(Component)]
pub(super) struct HotkeySlotButton(pub u8);

/// Marks a spell button in the right list for border highlighting.
#[derive(Component)]
pub(super) struct SpellListButton(pub Spell);
