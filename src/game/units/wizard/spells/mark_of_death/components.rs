use bevy::prelude::*;

/// Marker for the currently active Mark of Death target, so we can remove old marks.
#[derive(Component)]
pub struct ActiveMarkOfDeath;
