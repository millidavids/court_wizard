//! Polymorph transformation component.

use bevy::prelude::*;

use super::super::components::Team;

/// Polymorph effect that transforms a unit into a sheep.
///
/// Stores the original unit state for restoration when the effect expires.
#[derive(Component)]
pub struct PolymorphedModifier {
    /// Time remaining before the unit reverts (in seconds).
    pub time_remaining: f32,
    /// Original current health to restore on revert.
    pub original_health_current: f32,
    /// Original max health to restore on revert.
    pub original_health_max: f32,
    /// Original material handle to restore on revert.
    pub original_material: Handle<StandardMaterial>,
    /// Original mesh handle to restore on revert.
    pub original_mesh: Handle<Mesh>,
    /// Original team to restore on revert.
    pub original_team: Team,
}

impl PolymorphedModifier {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        duration: f32,
        health_current: f32,
        health_max: f32,
        material: Handle<StandardMaterial>,
        mesh: Handle<Mesh>,
        team: Team,
    ) -> Self {
        Self {
            time_remaining: duration,
            original_health_current: health_current,
            original_health_max: health_max,
            original_material: material,
            original_mesh: mesh,
            original_team: team,
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.time_remaining -= delta;
        self.time_remaining <= 0.0
    }
}
