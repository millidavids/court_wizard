use bevy::prelude::*;

use super::systems::*;
use crate::game::run_conditions::is_gameplay_running;

/// Plugin for the concentration UI that appears when the wizard is concentrating on a spell.
pub struct ConcentrationUIPlugin;

impl Plugin for ConcentrationUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_concentration_ui,
                update_button_hover,
                handle_end_concentration_click,
            )
                .run_if(is_gameplay_running),
        );
    }
}
