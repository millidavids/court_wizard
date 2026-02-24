use bevy::prelude::*;

use crate::game::run_conditions::is_local_wizard_active;

use super::systems;

/// Plugin that handles spell range visualization.
///
/// Shows a light blue dotted circle on the battlefield indicating the wizard's spell range.
pub struct SpellRangeIndicatorPlugin;

impl Plugin for SpellRangeIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::setup_spell_range_indicator,
                systems::update_spell_range_indicator,
                systems::pulse_spell_range_indicator,
            )
                .run_if(is_local_wizard_active),
        );
    }
}
