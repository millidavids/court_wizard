use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::GuardianCircleIndicator;
use super::systems;
use crate::game::run_conditions::is_gameplay_running;

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
                systems::handle_guardian_circle_casting
                    .run_if(spell_is_primed(Spell::GuardianCircle))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_circle_indicator.run_if(any_exist::<GuardianCircleIndicator>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
