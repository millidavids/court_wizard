use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::attacker::AttackerPlugin;
use super::battlefield::BattlefieldPlugin;
use super::defender::DefenderPlugin;
use super::shared_systems;
use super::wizard::WizardPlugin;

/// Main game plugin that coordinates all gameplay sub-plugins.
///
/// Registers sub-plugins for:
/// - Battlefield and castle setup (BattlefieldPlugin)
/// - Wizard entity (WizardPlugin)
/// - Defender units (DefenderPlugin)
/// - Attacker units (AttackerPlugin)
/// - Shared movement and cleanup systems
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BattlefieldPlugin,
            WizardPlugin,
            DefenderPlugin,
            AttackerPlugin,
        ))
        .add_systems(OnExit(AppState::InGame), shared_systems::cleanup_game)
        .add_systems(
            Update,
            shared_systems::move_units.run_if(in_state(InGameState::Running)),
        );
    }
}
