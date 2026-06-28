//! Steam Input lifecycle init.
//!
//! Calling `ISteamInput::Init` *captures* supported controllers (the OS / gilrs
//! stops seeing them), so this only runs because every action-state reader and the
//! poll producer are already in place — controllers never go dead between phases.

use bevy::prelude::*;
use bevy_steamworks::Client;

/// Relative path (from the executable) to the bundled IGA manifest. The build
/// script copies `assets/` next to the binary.
const MANIFEST_REL: &str = "assets/controller_config/game_actions_4550880.vdf";

/// Points Steam Input at the bundled action manifest, then initializes it.
/// `explicitly_call_run_frame = true` so [`super::run_frame`] drives updates.
pub(crate) fn init_steam_input(client: Res<Client>) {
    let input = client.input();

    match manifest_path() {
        Some(path) => {
            let ok = input.set_input_action_manifest_file_path(&path);
            info!("[Steam Input] action manifest '{path}' set: {ok}");
        }
        None => warn!(
            "[Steam Input] bundled action manifest not found on disk; relying on the \
             Steamworks-configured manifest path"
        ),
    }

    let ok = input.init(true);
    info!("[Steam Input] init: {ok}");
}

/// Best-effort absolute path to the bundled manifest: next to the executable
/// first (the shipped/built layout), falling back to the working directory.
fn manifest_path() -> Option<String> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join(MANIFEST_REL);
        if candidate.exists() {
            return candidate.to_str().map(str::to_string);
        }
    }
    let cwd_candidate = std::path::Path::new(MANIFEST_REL);
    if cwd_candidate.exists() {
        return cwd_candidate.to_str().map(str::to_string);
    }
    None
}
