use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::spells::SpellsPlugin;
use super::systems;

/// Plugin that handles wizard entity setup and spells.
///
/// Registers systems for:
/// - Wizard entity setup on entering InGame state
/// - Mana regeneration during gameplay
/// - Spell casting and projectile management (via SpellsPlugin)
pub struct WizardPlugin;

impl Plugin for WizardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpellsPlugin)
            .add_systems(OnEnter(AppState::InGame), systems::setup_wizard)
            .add_systems(
                Update,
                systems::regenerate_mana.run_if(in_state(InGameState::Running)),
            );
    }
}
