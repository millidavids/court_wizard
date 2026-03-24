use bevy::prelude::*;

use crate::state::AppState;
use crate::ui::systems::cleanup_screen;

use super::components::OnMenuBackground;
use super::systems;

/// Plugin that manages the parallax scrolling background for the main menu.
///
/// The background persists across all `MenuState` sub-screens and is only
/// cleaned up when leaving `AppState::MainMenu`.
#[derive(Default)]
pub(crate) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::load_assets)
            .add_systems(OnEnter(AppState::MainMenu), systems::setup)
            .add_systems(
                OnExit(AppState::MainMenu),
                cleanup_screen::<OnMenuBackground>,
            )
            .add_systems(
                Update,
                systems::scroll_parallax.run_if(in_state(AppState::MainMenu)),
            );
    }
}
