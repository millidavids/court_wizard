use bevy::prelude::*;

use super::components::TramplingOverlay;
use super::resources::TramplingGrid;
use super::systems;
use crate::game::run_conditions::{any_exist, is_gameplay_running};
use crate::state::AppState;

/// Plugin that tracks unit trampling on battlefield tiles and renders
/// a dynamic mud overlay that accumulates over time and decays between levels.
pub struct TramplingPlugin;

impl Plugin for TramplingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TramplingGrid::new())
            .add_systems(
                Update,
                (
                    systems::track_unit_trampling,
                    systems::sync_trampling_texture
                        .after(systems::track_unit_trampling),
                    systems::clear_dirty_flag
                        .after(systems::sync_trampling_texture),
                )
                    .run_if(is_gameplay_running)
                    .run_if(any_exist::<TramplingOverlay>()),
            )
            .add_systems(
                OnEnter(AppState::Loading),
                (
                    systems::restore_trampling_from_config,
                    systems::decay_trampling
                        .after(systems::restore_trampling_from_config),
                ),
            )
            .add_systems(
                OnExit(AppState::InGame),
                (
                    systems::save_trampling_to_config,
                    systems::reset_trampling
                        .after(systems::save_trampling_to_config),
                ),
            );
    }
}
