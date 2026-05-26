use bevy::prelude::*;

/// Marker component for the King unit.
#[derive(Component)]
pub struct King;

/// Tracks whether a King has been spawned this round.
/// Used by win/lose system to trigger defeat on King death.
#[derive(Resource, Default)]
pub struct KingSpawned(pub bool);

/// Marker component that makes the King immune to spell damage.
/// Only active in multiplayer. Removed when fewer than 10% of the king's
/// own team's non-king units remain alive.
#[derive(Component)]
pub struct SpellShield;

/// Marker component for the king's aura sphere visual (child of King).
/// Used to find and despawn the aura when the king dies (its parent does
/// not auto-despawn the child — the parent stays as a corpse).
#[derive(Component)]
pub struct KingAuraVisual;
