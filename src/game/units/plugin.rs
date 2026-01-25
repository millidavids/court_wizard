use bevy::prelude::*;

use crate::state::InGameState;

use super::infantry::InfantryPlugin;
use super::systems;
use super::wizard::WizardPlugin;

/// Plugin that coordinates all unit-related sub-plugins.
///
/// Registers sub-plugins for:
/// - Wizard entity (WizardPlugin)
/// - Infantry units on both teams (InfantryPlugin)
///
/// Also registers global unit systems for:
/// - Temporary hit points expiration
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WizardPlugin, InfantryPlugin)).add_systems(
            Update,
            systems::update_temporary_hit_points.run_if(in_state(InGameState::Running)),
        );
    }
}
