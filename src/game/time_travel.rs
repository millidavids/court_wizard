//! Endless "time travel": replaying an already-completed level.
//!
//! Time travel is deliberately non-progressing — nothing the player does on a
//! replayed level touches their real level, efficiency, or permanent structures.
//! That's enforced in two places, and both are needed:
//!
//! - **In memory**: entering a replay overwrites `GameConfig`'s terrain with the
//!   target level's snapshot, so the real terrain is stashed here and restored on
//!   the way out. Without that, the next normal level would spawn the old level's
//!   walls and crystals and then re-persist them on victory.
//! - **On disk**: while `TimeTravelState` exists, `save_config_to_active_wizard`
//!   skips the whole progression block. Without that, the debounced config save
//!   writes the replay's terrain over the player's real save within half a second
//!   of pressing "Start Time Travel" — and `save_on_exit` makes an Alt-F4 during a
//!   replay permanent.

use bevy::prelude::*;

use crate::config::save_data::{
    SavedLevelTerrain, apply_terrain_to_config, load_level_terrain_into_config,
    snapshot_terrain_from_config,
};
use crate::config::{ActiveSave, GameConfig};

use super::resources::CurrentLevel;

/// When present, the player is replaying a past level.
/// Stores the real progression state to restore when the replay ends.
#[derive(Resource)]
pub struct TimeTravelState {
    /// The level the player was actually on before time travelling.
    pub real_level: u32,
    /// Live progression terrain captured when time travel started. Restored on
    /// exit so a replay can't leave the wrong permanent walls, crystals, or
    /// flora sitting in the live config.
    pub real_terrain: SavedLevelTerrain,
}

/// Enters time travel: stashes the real level and terrain, then points
/// `CurrentLevel` at the level being replayed.
///
/// Deliberately does **not** touch `GameConfig`'s terrain. `TimeTravelState` is
/// inserted through `Commands` and so isn't visible until the end of the frame,
/// while the debounced config save runs unordered in the same `Update` — writing
/// the replay's terrain here would leave a one-frame window where that save sees
/// the swapped terrain but not the marker guarding it. The swap happens in
/// `load_time_travel_terrain` instead, one frame later, with the marker resident.
pub fn begin_time_travel(
    commands: &mut Commands,
    current_level: &mut CurrentLevel,
    config: &GameConfig,
    level: u32,
) {
    commands.insert_resource(TimeTravelState {
        real_level: config.current_level,
        real_terrain: snapshot_terrain_from_config(config),
    });
    current_level.0 = level;
}

/// Loads the replayed level's terrain snapshot so the battlefield looks like it
/// did when that level was first played.
///
/// Runs on `OnEnter(AppState::Loading)` before `init_loading_progress` builds the
/// spawn queue from these fields. Also covers the retry path, where a defeat
/// sends the player back through `Loading` with the marker still resident.
pub(super) fn load_time_travel_terrain(
    current_level: Res<CurrentLevel>,
    active_save: Res<ActiveSave>,
    mut config: ResMut<GameConfig>,
) {
    load_level_terrain_into_config(&active_save, current_level.0, &mut config);
}

/// Restores the real level and terrain. Does not drop the marker — see
/// `finish_time_travel`.
fn restore_real_state(
    state: &TimeTravelState,
    current_level: &mut CurrentLevel,
    config: &mut GameConfig,
) {
    current_level.0 = state.real_level;
    apply_terrain_to_config(&state.real_terrain, config);
}

/// Ends time travel once the player has left the battle for good.
///
/// Registered on `OnEnter(MetaGame)` and `OnEnter(MainMenu)` — every exit from a
/// replay except the retry, which re-enters `Loading` and must keep the marker.
///
/// The marker deliberately survives `OnExit(AppState::InGame)`: several teardown
/// systems there check for it to avoid writing replay state into the wizard's
/// real save. `save_trampling_to_config` is the one that bites — tearing the
/// marker down inside the score-screen button handler would clear it a frame too
/// early and let the replayed level's mud grid overwrite the real one.
pub(super) fn finish_time_travel(
    mut commands: Commands,
    state: Option<Res<TimeTravelState>>,
    mut current_level: ResMut<CurrentLevel>,
    mut config: ResMut<GameConfig>,
) {
    let Some(state) = state else { return };
    restore_real_state(&state, &mut current_level, &mut config);
    commands.remove_resource::<TimeTravelState>();
}
