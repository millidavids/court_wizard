use bevy::prelude::*;

use crate::game::cauldron::brews::Ingredient;

/// Actions that can be triggered by cauldron menu buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CauldronMenuButtonAction {
    ToggleIngredient(Ingredient),
    StartBrew,
    CancelBrew,
    Close,
}

/// Marker component for entities that should be cleaned up when exiting cauldron menu.
#[derive(Component)]
pub(super) struct OnCauldronMenuScreen;

/// Resource tracking which ingredients the player has selected.
#[derive(Resource, Default)]
pub(super) struct IngredientSelection {
    pub selected: Vec<Ingredient>,
}

impl IngredientSelection {
    pub fn toggle(&mut self, ingredient: Ingredient) {
        if let Some(pos) = self.selected.iter().position(|i| *i == ingredient) {
            self.selected.remove(pos);
        } else {
            self.selected.push(ingredient);
        }
    }

    pub fn is_selected(&self, ingredient: &Ingredient) -> bool {
        self.selected.contains(ingredient)
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }
}
