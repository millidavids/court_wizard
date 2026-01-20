use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

/// Plugin that handles wizard entity setup.
///
/// Registers systems for:
/// - Wizard entity setup on entering InGame state
pub struct WizardPlugin;

impl Plugin for WizardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::setup_wizard);
    }
}
