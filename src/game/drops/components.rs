use bevy::prelude::*;

use crate::game::cauldron::brews::Ingredient;

/// Marks an entity as a pickup-able ingredient drop on the battlefield.
#[derive(Component)]
pub(crate) struct IngredientDrop {
    /// Which ingredient this drop will unlock when collected.
    pub ingredient: Ingredient,
    /// Time this drop has been alive (for despawn and fade).
    pub time_alive: f32,
}

/// Marks a drop that has been picked up and is flying toward the wizard.
/// The IngredientDrop component is removed when this is added, so the drop
/// is no longer targetable by Telekinesis.
#[derive(Component)]
pub(crate) struct FlyingToWizard {
    /// Which ingredient this will unlock on arrival.
    pub ingredient: Ingredient,
    /// Starting position of the flight (for arc calculation).
    pub start_pos: Vec3,
    /// Total distance to travel (for arc progress).
    pub total_distance: f32,
}

/// Resource caching which ingredients are still locked, to avoid repeated save reads.
#[derive(Resource, Default)]
pub(crate) struct LockedIngredients {
    pub locked: Vec<Ingredient>,
}
