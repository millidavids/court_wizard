//! Steam Input lifecycle init.
//!
//! Calling `ISteamInput::Init` *captures* supported controllers (the OS / gilrs
//! stops seeing them), so this only runs because every action-state reader and the
//! poll producer are already in place — controllers never go dead between phases.

use bevy::prelude::*;
use bevy_steamworks::Client;

/// Points Steam Input at the bundled action manifest for the app Steam launched
/// us with, then initializes it. `explicitly_call_run_frame = true` so
/// [`super::run_frame`] drives updates.
pub(crate) fn init_steam_input(client: Res<Client>) {
    let input = client.input();
    let app_id = client.utils().app_id().0;

    // Point Steam Input at the bundled action manifest with a clean, OS-native
    // path (backslashes on Windows) BEFORE Init — Steam rejects a mixed-slash
    // path, and the manifest MUST be the extended Action Manifest form (a plain
    // In-Game Actions file returns false and the runtime action handles never
    // resolve). `manifest_path` joins components individually so separators are
    // native.
    match manifest_path(app_id) {
        Some(path) => {
            let ok = input.set_input_action_manifest_file_path(&path);
            info!("[Steam Input] action manifest '{path}' set: {ok}");
        }
        None => warn!("[Steam Input] no bundled action manifest on disk for app {app_id}"),
    }

    let ok = input.init(true);
    info!("[Steam Input] init: {ok}");
}

/// Best-effort absolute path to the bundled manifest for `app_id`, built from
/// individually-joined components so the separators are OS-native (backslashes on
/// Windows — Steam rejects a mixed-slash path). The filename MUST carry the
/// *running* app id. Prefers Steam's canonical install-root location
/// (`controller_config/…`) over the `assets/` copy, and the bundled data dir
/// (exe dir, or Contents/Resources inside a macOS .app) over the cwd.
fn manifest_path(app_id: u32) -> Option<String> {
    use std::path::PathBuf;
    let file = format!("game_actions_{app_id}.vdf");
    let data_dir = crate::config::resource_root();
    [data_dir, Some(PathBuf::from("."))]
        .into_iter()
        .flatten()
        .flat_map(|base| {
            [
                base.join("controller_config").join(&file),
                base.join("assets").join("controller_config").join(&file),
            ]
        })
        .find(|p| p.exists())
        .and_then(|p| p.to_str().map(str::to_string))
}
