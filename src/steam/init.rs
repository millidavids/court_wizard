//! Steam client initialization. Kept out of `plugin.rs` so that file stays
//! registration-only.

use bevy::prelude::*;
use bevy_steamworks::{SteamAPIInitError, SteamworksPlugin};

#[cfg(debug_assertions)]
use super::constants::APP_ID;

/// Initialize Steam using the app id Steam launched us with — correct for BOTH
/// the main app (4550880) and the Playtest (4820340) from a single binary, since
/// Steam launches each under its own app id. `init()` reads that id from the
/// launch context (or `steam_appid.txt` in the working dir when launched outside
/// Steam). In dev builds only, fall back to the hardcoded main app id so a binary
/// run directly from the command line (no `steam_appid.txt` in cwd) still gets
/// Steam features. The fallback is compiled OUT of release builds, so a shipped
/// playtest binary can ONLY use the Steam-provided app id and can never force the
/// main app id (which playtest testers have no license for — that was the bug).
pub(super) fn init_steam_plugin() -> Result<SteamworksPlugin, SteamAPIInitError> {
    let result = SteamworksPlugin::init();

    #[cfg(debug_assertions)]
    let result = result.or_else(|e| {
        warn!("Steam auto-init failed ({e}); falling back to dev app id {APP_ID}");
        SteamworksPlugin::init_app(APP_ID)
    });

    result
}
