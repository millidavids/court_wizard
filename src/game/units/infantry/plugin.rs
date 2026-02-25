use bevy::prelude::*;

use crate::game::run_conditions::any_exist;
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

use super::components::{DefendersActivated, Infantry};
use super::resources;
use super::systems;

/// Plugin that handles infantry units (both defenders and attackers).
///
/// Registers systems for:
/// - Updating defender and attacker targeting
/// - Shared activation system for defenders
///
/// Infantry spawning is handled via the loading spawn queue.
pub struct InfantryPlugin;

impl Plugin for InfantryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DefendersActivated>()
            .add_systems(Startup, resources::preload_infantry_assets)
            .add_systems(
                Update,
                (
                    systems::check_defender_activation
                        .before(crate::game::plugin::VelocitySystemSet),
                    systems::update_infantry_targeting
                        .in_set(crate::game::plugin::VelocitySystemSet),
                    systems::infantry_movement.in_set(MovementCalculationSet),
                )
                    .run_if(any_exist::<Infantry>())
                    .run_if(is_gameplay_running),
            );
    }
}
