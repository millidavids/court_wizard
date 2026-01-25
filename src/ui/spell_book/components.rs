use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Actions that can be triggered by spell book buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellBookButtonAction {
    SelectSpell(Spell),
    Close,
}

/// Marker component for entities that should be cleaned up when exiting spell book.
#[derive(Component)]
pub struct OnSpellBookScreen;
