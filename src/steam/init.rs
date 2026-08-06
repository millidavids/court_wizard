//! Steam client initialization. Kept out of `plugin.rs` so that file stays
//! registration-only.

use bevy::prelude::*;
use bevy_steamworks::{SteamAPIInitError, SteamworksPlugin};

#[cfg(debug_assertions)]
use super::constants::APP_ID;

/// Initialize Steam using the app id Steam launched us with, never a hardcoded
/// one. `init()` reads that id from the launch context (or `steam_appid.txt` in
/// the working dir when launched outside Steam).
///
/// In dev builds only, fall back to the hardcoded app id so a binary run
/// directly from the command line (no `steam_appid.txt` in cwd) still gets Steam
/// features. That fallback is compiled OUT of release builds on purpose: a
/// shipped binary that forced an app id would break for anyone whose license is
/// for a different id (a separate demo or playtest app, say) — they'd be pushed
/// onto an app they don't own. Taking the id Steam supplies means one binary
/// serves every app id correctly.
pub(super) fn init_steam_plugin() -> Result<SteamworksPlugin, SteamAPIInitError> {
    let result = SteamworksPlugin::init();

    #[cfg(debug_assertions)]
    let result = result.or_else(|e| {
        warn!("Steam auto-init failed ({e}); falling back to dev app id {APP_ID}");
        SteamworksPlugin::init_app(APP_ID)
    });

    result
}
