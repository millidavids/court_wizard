//! System sets for unit movement and transform application.

use bevy::prelude::*;

/// System set for calculating unit movement (acceleration, velocity).
/// All unit-specific movement calculations should be in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementCalculationSet;

/// System set for applying transforms based on calculated movement.
/// This set runs after MovementCalculationSet.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyTransformsSet;

/// System set wrapping the multiplayer guest's snapshot-application system
/// (`apply_state_snapshot`). Animation systems order themselves after this
/// set so they read the synthesised ghost `Velocity` written by the snapshot
/// rather than the previous frame's value.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GuestSnapshotSet;
