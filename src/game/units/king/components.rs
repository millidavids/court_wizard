use bevy::prelude::*;

/// Marker component for the King unit.
#[derive(Component)]
pub struct King;

/// Tracks whether a King has been spawned this round.
/// Used by win/lose system to trigger defeat on King death.
#[derive(Resource, Default)]
pub struct KingSpawned(pub bool);
