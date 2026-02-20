use bevy::prelude::*;

use crate::game::run_conditions::is_arcanorouter;
use crate::game::run_conditions::is_gameplay_running;
use crate::state::InGameState;

use super::systems::*;

/// Plugin that manages the Arcanorouter UI display (4 resource allocation sliders)
pub(crate) struct ArcanoRouterDisplayPlugin;

impl Plugin for ArcanoRouterDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::Running),
            spawn_arcanorouter_display.run_if(is_arcanorouter),
        )
        .add_systems(
            Update,
            (update_slider_visuals, handle_slider_interaction)
                .run_if(is_gameplay_running)
                .run_if(is_arcanorouter),
        );
    }
}
