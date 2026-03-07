use bevy::prelude::*;

use super::constants::MAX_INGREDIENTS;
use crate::game::cauldron::brews::Ingredient;

/// Actions that can be triggered by cauldron menu buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CauldronMenuButtonAction {
    ToggleIngredient(Ingredient),
    TogglePhilosophersStone,
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
    pub philosophers_stone: bool,
}

impl IngredientSelection {
    pub fn toggle(&mut self, ingredient: Ingredient) {
        if let Some(pos) = self.selected.iter().position(|i| *i == ingredient) {
            self.selected.remove(pos);
        } else if self.selected.len() < MAX_INGREDIENTS {
            self.selected.push(ingredient);
        }
    }

    pub fn is_selected(&self, ingredient: &Ingredient) -> bool {
        self.selected.contains(ingredient)
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn at_limit(&self) -> bool {
        self.selected.len() >= MAX_INGREDIENTS
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.philosophers_stone = false;
    }

    pub fn toggle_stone(&mut self) {
        self.philosophers_stone = !self.philosophers_stone;
    }

    pub fn has_stone(&self) -> bool {
        self.philosophers_stone
    }

    /// Builds the full ingredient list including the Stone if selected.
    pub fn build_ingredients(&self) -> Vec<Ingredient> {
        let mut ingredients = self.selected.clone();
        if self.philosophers_stone {
            ingredients.push(Ingredient::PhilosophersStone);
        }
        ingredients
    }
}
