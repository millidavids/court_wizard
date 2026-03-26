use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::game::messages::AchievementUnlockedMessage;

use super::constants::steam_api_name;

/// Listens for in-game achievement unlocks and syncs them to Steam.
pub(super) fn sync_achievements_to_steam(
    client: Res<Client>,
    mut messages: MessageReader<AchievementUnlockedMessage>,
) {
    let user_stats = client.user_stats();
    let mut any_unlocked = false;

    for msg in messages.read() {
        let api_name = steam_api_name(msg.id);
        if user_stats.achievement(api_name).get().unwrap_or(false) {
            continue;
        }
        match user_stats.achievement(api_name).set() {
            Ok(()) => {
                info!("Steam achievement unlocked: {api_name}");
                any_unlocked = true;
            }
            Err(()) => {
                warn!("Failed to set Steam achievement: {api_name}");
            }
        }
    }

    if any_unlocked {
        if let Err(()) = user_stats.store_stats() {
            warn!("Failed to store Steam stats");
        }
    }
}
