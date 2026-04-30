//! Top-level system sets for ordering gameplay updates.

use bevy::prelude::*;

/// System set for velocity calculation systems.
///
/// These systems use immutable queries to calculate velocities and accelerations:
/// - Targeting: Sets TargetingVelocity based on nearest enemy
/// - Flocking/Separation: Adds forces to Acceleration
///
/// All systems in this set can run in parallel since they only read Transform.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VelocitySystemSet;

/// System set for unit movement systems.
///
/// Movement systems query their specific unit type (mutable Transform) and apply velocities.
/// This set runs after velocity calculations to ensure all velocities are computed first.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSystemSet;

/// System set that runs after combat resolution (wall collision → combat → corpse conversion).
/// Used by systems that need to react to combat results (e.g., brute AOE splash).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostCombatSet;
