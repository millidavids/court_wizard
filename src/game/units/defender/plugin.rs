use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::components::DefendersActivated;
use super::systems;

/// Plugin that handles friendly defender units.
///
/// Registers systems for:
/// - Initial spawn of defenders on game start
/// - Updating defender targeting toward nearest attacker
/// - Combat between defenders and attackers
/// - Shared activation system
pub struct DefenderPlugin;

impl Plugin for DefenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefendersActivated>()
            .add_systems(OnEnter(AppState::InGame), systems::spawn_initial_defenders)
            .add_systems(
                Update,
                (systems::update_defender_targets, systems::combat)
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
