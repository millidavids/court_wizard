use bevy::prelude::*;

use super::attacker::AttackerPlugin;
use super::defender::DefenderPlugin;
use super::wizard::WizardPlugin;

/// Plugin that coordinates all unit-related sub-plugins.
///
/// Registers sub-plugins for:
/// - Wizard entity (WizardPlugin)
/// - Defender units (DefenderPlugin)
/// - Attacker units (AttackerPlugin)
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WizardPlugin, DefenderPlugin, AttackerPlugin));
    }
}
