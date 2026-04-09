use bevy::prelude::*;

/// Marker component for the King unit.
#[derive(Component)]
pub struct King;

/// Tracks whether a King has been spawned this round.
/// Used by win/lose system to trigger defeat on King death.
#[derive(Resource, Default)]
pub struct KingSpawned(pub bool);

/// Marker component that makes the King immune to spell damage.
/// Only active in multiplayer. Removed when fewer than 10% of non-King
/// defenders remain alive.
#[derive(Component)]
pub struct SpellShield;

/// Marker component for the spell shield visual sphere (child of King).
#[derive(Component)]
pub struct SpellShieldVisual;

