use bevy::prelude::*;

/// Marker component for the spell range circle parent entity.
#[derive(Component)]
pub struct SpellRangeCircle;

/// Marker component for individual dot segments of the circle.
#[derive(Component)]
pub struct SpellRangeDash;
