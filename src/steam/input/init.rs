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

    match manifest_path(app_id) {
        Some(path) => {
            let ok = input.set_input_action_manifest_file_path(&path);
            info!("[Steam Input] action manifest '{path}' set: {ok}");
        }
        None => warn!(
            "[Steam Input] no bundled manifest for app {app_id} on disk; relying on \
             Steam's auto-discovery of controller_config/game_actions_{app_id}.vdf"
        ),
    }

    let ok = input.init(true);
    info!("[Steam Input] init: {ok}");
}

/// Best-effort absolute path to the bundled manifest for `app_id`. The filename
/// MUST carry the *running* app id — Steam finds/accepts an IGA by
/// `game_actions_<appid>.vdf`, and rejects one named for a different app. Prefers
/// Steam's canonical install-root location (`controller_config/…`, which Steam
/// also auto-discovers) over the `assets/` copy, and the exe dir over the cwd.
fn manifest_path(app_id: u32) -> Option<String> {
    let file = format!("game_actions_{app_id}.vdf");
    let rels = [
        format!("controller_config/{file}"),
        format!("assets/controller_config/{file}"),
    ];
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(std::path::Path::to_path_buf));
    for rel in &rels {
        if let Some(dir) = &exe_dir {
            let candidate = dir.join(rel);
            if candidate.exists() {
                return candidate.to_str().map(str::to_string);
            }
        }
        let candidate = std::path::Path::new(rel);
        if candidate.exists() {
            return candidate.to_str().map(str::to_string);
        }
    }
    None
}
