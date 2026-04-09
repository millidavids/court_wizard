use bevy::prelude::*;

/// Marker component for the concentration UI root container.
#[derive(Component)]
pub(crate) struct ConcentrationUIRoot;

/// A cancel button for a specific concentration spell, storing its game entity.
#[derive(Component)]
pub(super) struct ConcentrationSpellButton {
    pub spell_entity: Entity,
}
