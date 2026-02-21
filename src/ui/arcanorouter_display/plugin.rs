use bevy::prelude::*;

use crate::game::run_conditions::{is_arcanorouter, is_local_wizard_active};
use crate::state::{InGameState, MultiplayerGameState};

use super::systems::*;

/// Plugin that manages the Arcanorouter UI display (4 resource allocation sliders)
pub(crate) struct ArcanoRouterDisplayPlugin;

impl Plugin for ArcanoRouterDisplayPlugin {
    fn build(&self, app: &mut App) {
        // SP spawn
        app.add_systems(
            OnEnter(InGameState::Running),
            spawn_arcanorouter_display.run_if(is_arcanorouter),
        )
        // MP spawn
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            spawn_arcanorouter_display.run_if(is_arcanorouter),
        )
        .add_systems(
            Update,
            (update_slider_visuals, handle_slider_interaction)
                .run_if(is_local_wizard_active)
                .run_if(is_arcanorouter),
        );
    }
}
