//! Exe-relative resolution of bundled runtime data (`assets/`, `controller_config/`).

use std::path::PathBuf;

/// Directory holding the game's bundled runtime data, resolved relative to the
/// running executable so launches work regardless of the current working directory.
///
/// Everywhere except a macOS `.app` bundle this is the executable's own
/// directory. Inside a bundle the executable lives at `*.app/Contents/MacOS/<bin>`
/// while data must live in `*.app/Contents/Resources/` — data files placed in
/// `Contents/MacOS/` break codesign's resource sealing — so that directory is
/// returned instead.
pub(crate) fn resource_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let contents_dir = exe_dir.parent();
    let in_app_bundle = exe_dir.file_name().is_some_and(|n| n == "MacOS")
        && contents_dir.is_some_and(|c| c.file_name().is_some_and(|n| n == "Contents"))
        && contents_dir
            .and_then(|c| c.parent())
            .and_then(|a| a.file_name())
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".app"));

    if in_app_bundle {
        contents_dir.map(|c| c.join("Resources"))
    } else {
        Some(exe_dir.to_path_buf())
    }
}
