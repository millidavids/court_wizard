use bevy::prelude::*;

use crate::game::cauldron::brews::Brew;

/// Actions that can be triggered by cauldron menu buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CauldronMenuButtonAction {
    SelectBrew(Brew),
    CancelBrew,
    Close,
}

/// Marker component for entities that should be cleaned up when exiting cauldron menu.
#[derive(Component)]
pub(super) struct OnCauldronMenuScreen;
