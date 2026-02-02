use bevy::prelude::*;

/// Marker for the root rune display container.
#[derive(Component)]
pub(super) struct RuneDisplayRoot;

/// Marker for the text showing the current rune sequence.
#[derive(Component)]
pub(super) struct RuneSequenceText;

/// Marker for the validity indicator text.
#[derive(Component)]
pub(super) struct RuneValidityText;
