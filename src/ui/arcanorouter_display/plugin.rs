use bevy::prelude::*;

use crate::{
    game::run_conditions::is_arcanorouter,
    state::{AppState, InGameState},
};

use super::systems::*;

/// Plugin that manages the Arcanorouter UI display (4 resource allocation sliders)
pub(crate) struct ArcanoRouterDisplayPlugin;

impl Plugin for ArcanoRouterDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_arcanorouter_display.run_if(is_arcanorouter),
        )
        .add_systems(
            Update,
            (update_slider_visuals, handle_slider_interaction)
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Running))
                .run_if(is_arcanorouter),
        );
    }
}
