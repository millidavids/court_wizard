use bevy::prelude::*;

/// Marker component for the concentration UI root container.
#[derive(Component)]
pub(super) struct ConcentrationUIRoot;

/// Marker component for the concentration spell name text.
#[derive(Component)]
pub(super) struct ConcentrationSpellNameText;

/// Marker component for the "End Concentration" button.
#[derive(Component)]
pub(super) struct EndConcentrationButton;
