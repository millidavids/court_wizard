use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};

use super::systems;

/// Plugin that handles battlefield and castle setup.
///
/// Registers systems for:
/// - Battlefield ground, castle platform, and lighting setup on entering InGame state
/// - Re-setup when entering Running state from GameOver (for replay)
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::setup_battlefield)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::setup_battlefield.run_if(run_conditions::coming_from_game_over),
            );
    }
}
