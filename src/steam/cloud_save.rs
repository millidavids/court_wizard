use std::io::{Read, Write};
use std::path::PathBuf;

use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::config::storage::UNIFIED_SAVE_FILENAME;

/// Returns the local save path if Steam Cloud is enabled, or None otherwise.
fn cloud_save_path(client: &Client) -> Option<PathBuf> {
    let remote = client.remote_storage();
    if !remote.is_cloud_enabled_for_account() || !remote.is_cloud_enabled_for_app() {
        return None;
    }
    let save_dir = match crate::config::storage::save_dir() {
        Ok(dir) => dir,
        Err(e) => {
            warn!("Steam Cloud: could not determine save dir: {e}");
            return None;
        }
    };
    Some(save_dir.join(UNIFIED_SAVE_FILENAME))
}

/// On startup, check if Steam Cloud has a save and restore it locally if needed.
///
/// Steam Auto-Cloud (configured in the Steamworks dashboard) handles uploading
/// local saves to the cloud automatically. This system only handles the
/// restore case: when a player starts on a new device with no local save,
/// we pull the cloud save down before the game loads it.
pub(super) fn restore_save_from_steam_cloud(client: Res<Client>) {
    let local_path = match cloud_save_path(&client) {
        Some(p) => p,
        None => {
            info!("Steam Cloud: disabled, skipping restore");
            return;
        }
    };

    let remote = client.remote_storage();
    let cloud_file = remote.file(UNIFIED_SAVE_FILENAME);
    if !cloud_file.exists() {
        info!("Steam Cloud: no cloud save found");
        return;
    }

    if local_path.exists() {
        info!("Steam Cloud: local save exists, skipping restore");
        return;
    }

    let mut reader = cloud_file.read();
    let mut data = Vec::new();
    match reader.read_to_end(&mut data) {
        Ok(bytes_read) => {
            if let Some(parent) = local_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                warn!("Steam Cloud: could not create save dir: {e}");
                return;
            }
            if let Err(e) = std::fs::write(&local_path, &data) {
                warn!("Steam Cloud: could not write local save: {e}");
                return;
            }
            info!("Steam Cloud: restored save from cloud ({bytes_read} bytes)");
        }
        Err(e) => {
            warn!("Steam Cloud: failed to read cloud save: {e}");
        }
    }
}

/// Sync the local save file to Steam Cloud.
/// Called at natural save checkpoints to keep cloud in sync.
pub(super) fn sync_save_to_steam_cloud(client: Res<Client>) {
    let local_path = match cloud_save_path(&client) {
        Some(p) => p,
        None => return,
    };

    let data = match std::fs::read(&local_path) {
        Ok(d) => d,
        Err(_) => return,
    };

    let remote = client.remote_storage();
    let mut writer = remote.file(UNIFIED_SAVE_FILENAME).write();
    match writer.write_all(&data) {
        Ok(()) => {
            info!("Steam Cloud: synced save ({} bytes)", data.len());
        }
        Err(e) => {
            warn!("Steam Cloud: failed to write: {e}");
        }
    }
}
