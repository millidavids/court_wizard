use bevy::prelude::*;
use bevy_steamworks::SteamworksPlugin;

use crate::state::AppState;

use super::achievements::sync_achievements_to_steam;
use super::cloud_save::{restore_save_from_steam_cloud, sync_save_to_steam_cloud};
use super::constants::APP_ID;
use super::leaderboards::LeaderboardsPlugin;

/// Bevy plugin that integrates Steam features (achievements, cloud saves, overlay).
///
/// Initialization is graceful: if Steam is not running or the app ID is invalid,
/// the plugin logs a warning and the game continues without Steam features.
pub(crate) struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        match SteamworksPlugin::init_app(APP_ID) {
            Ok(steamworks_plugin) => {
                info!("Steam initialized successfully (App ID: {APP_ID})");
                app.add_plugins(steamworks_plugin);
                app.add_plugins(LeaderboardsPlugin);

                // Restore cloud saves before the game loads save data.
                app.add_systems(Startup, restore_save_from_steam_cloud);

                // Sync achievements to Steam whenever one is unlocked. Runs in all
                // states because some achievements (e.g. SliderFiddler) fire in
                // menus, not just gameplay.
                app.add_systems(Update, sync_achievements_to_steam);

                // Sync save file to Steam Cloud at natural save checkpoints.
                app.add_systems(OnEnter(AppState::MainMenu), sync_save_to_steam_cloud);
                app.add_systems(OnEnter(AppState::MetaGame), sync_save_to_steam_cloud);
            }
            Err(e) => {
                warn!("Steam initialization failed: {e}. Running without Steam features.");
            }
        }
    }
}
