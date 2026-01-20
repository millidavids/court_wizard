use bevy::prelude::*;

use super::infantry::InfantryPlugin;
use super::wizard::WizardPlugin;

/// Plugin that coordinates all unit-related sub-plugins.
///
/// Registers sub-plugins for:
/// - Wizard entity (WizardPlugin)
/// - Infantry units on both teams (InfantryPlugin)
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WizardPlugin, InfantryPlugin));
    }
}
