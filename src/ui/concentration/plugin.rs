use bevy::prelude::*;

use super::systems::*;
use crate::game::run_conditions::is_local_wizard_active;

/// Plugin for the concentration UI that appears when the wizard is concentrating on a spell.
pub struct ConcentrationUIPlugin;

impl Plugin for ConcentrationUIPlugin {
    fn build(&self, app: &mut App) {
        // Both peers need this in MP — the local wizard's concentration UI
        // is driven by the local peer's own `ConcentrationSpell` entities,
        // which exist on whichever peer cast the channeled spell. Gating
        // on `is_gameplay_running` (host-only) hid the cancel button from
        // the guest entirely.
        app.add_systems(
            Update,
            (
                spawn_concentration_ui,
                update_button_hover,
                handle_end_concentration_click,
            )
                .run_if(is_local_wizard_active),
        );
    }
}
