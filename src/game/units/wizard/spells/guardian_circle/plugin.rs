use bevy::prelude::*;

use crate::state::InGameState;

use super::systems;

/// Plugin that handles Guardian Circle spell casting and behavior.
///
/// Registers systems for:
/// - Casting Guardian Circle with mouse button and cast time
/// - Visual circle indicator during cast
/// - Applying temporary HP buff to units in area
/// - Circle animation and updates
pub struct GuardianCirclePlugin;

impl Plugin for GuardianCirclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_guardian_circle_casting,
                systems::update_circle_indicator,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
