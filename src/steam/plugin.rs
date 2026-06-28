use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;
use bevy_steamworks::{Client, SteamworksEvent};

use crate::game::achievements::systems::send_battle_ended;
use crate::state::{AppState, InGameState};

use super::achievements::sync_achievements_to_steam;
use super::cloud_save::{restore_save_from_steam_cloud, sync_save_to_steam_cloud};
use super::init::init_steam_plugin;
use super::input::SteamInputPlugin;
use super::leaderboards::LeaderboardsPlugin;
use super::multiplayer::SteamMultiplayerPlugin;
use super::overlay_pause::pause_on_steam_overlay;
use super::stats::sync_stats_to_steam;

/// Bevy plugin that integrates Steam features (achievements, cloud saves, overlay).
///
/// Initialization is graceful: if Steam is not running or the app ID is invalid,
/// the plugin logs a warning and the game continues without Steam features.
pub(crate) struct SteamPlugin;

impl Plugin for SteamPlugin {
    fn build(&self, app: &mut App) {
        match init_steam_plugin() {
            Ok(steamworks_plugin) => {
                app.add_plugins(steamworks_plugin);
                // Log the app id Steam ACTUALLY initialized under (4550880 main /
                // 4820340 playtest), not the constant — this is the line to check
                // when diagnosing playtest licensing.
                match app.world().get_resource::<Client>() {
                    Some(client) => info!(
                        "Steam initialized successfully (App ID: {})",
                        client.utils().app_id().0
                    ),
                    None => info!("Steam initialized successfully"),
                }
                app.add_plugins(LeaderboardsPlugin);
                app.add_plugins(SteamMultiplayerPlugin);
                app.add_plugins(SteamInputPlugin);

                // Restore cloud saves before the game loads save data.
                app.add_systems(Startup, restore_save_from_steam_cloud);

                // Sync achievements to Steam whenever one is unlocked. Runs in all
                // states because some achievements (e.g. SliderFiddler) fire in
                // menus, not just gameplay.
                app.add_systems(Update, sync_achievements_to_steam);

                // Pause the game when the Steam overlay is opened. Only polls the
                // callback bus when an event is actually waiting.
                app.add_systems(
                    Update,
                    pause_on_steam_overlay.run_if(on_message::<SteamworksEvent>),
                );

                // Sync save file to Steam Cloud at natural save checkpoints.
                app.add_systems(OnEnter(AppState::MainMenu), sync_save_to_steam_cloud);
                app.add_systems(OnEnter(AppState::MetaGame), sync_save_to_steam_cloud);

                // Mirror lifetime totals to Steam stats. Absolute/idempotent, so it's
                // safe to push at several checkpoints; the redundancy also self-heals
                // an early no-op before Steam has delivered current-user stats. The
                // score-screen push runs after send_battle_ended so it sees the run
                // just recorded into the save.
                app.add_systems(
                    OnEnter(AppState::MainMenu),
                    sync_stats_to_steam.run_if(resource_exists::<Client>),
                );
                app.add_systems(
                    OnEnter(AppState::MetaGame),
                    sync_stats_to_steam.run_if(resource_exists::<Client>),
                );
                app.add_systems(
                    OnEnter(InGameState::ScoreScreen),
                    sync_stats_to_steam
                        .after(send_battle_ended)
                        .run_if(resource_exists::<Client>),
                );
            }
            Err(e) => {
                warn!("Steam initialization failed: {e}. Running without Steam features.");
            }
        }
    }
}
