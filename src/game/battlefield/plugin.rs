use bevy::prelude::*;

use crate::state::AppState;

use super::systems;

/// Plugin that handles battlefield and castle setup.
///
/// Registers systems for:
/// - Battlefield ground, castle platform, and lighting setup on entering InGame state
pub struct BattlefieldPlugin;

impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::setup_battlefield);
    }
}
