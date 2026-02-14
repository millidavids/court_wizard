use bevy::prelude::*;

use crate::game::cauldron::brews::Ingredient;

/// Marker for entities that should be despawned when exiting WizardTower state.
#[derive(Component)]
pub(super) struct OnWizardTowerScreen;

/// Actions that can be triggered by buttons on the wizard tower screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum WizardTowerButtonAction {
    StartNextBattle,
    ReturnToMenu,
}

/// Resource to track newly unlocked ingredient (to display on wizard tower screen).
#[derive(Resource, Default)]
pub(super) struct NewlyUnlockedIngredient(pub Option<Ingredient>);
