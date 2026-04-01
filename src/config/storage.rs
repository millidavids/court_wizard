use std::path::PathBuf;

use super::error::ConfigResult;

const CONFIG_FILENAME: &str = "config.toml";
const PROGRESS_FILENAME: &str = "progress.dat";
pub(crate) const UNIFIED_SAVE_FILENAME: &str = "saves_v2.json";

/// Returns the platform-appropriate data directory for Court Wizard.
pub(crate) fn save_dir() -> ConfigResult<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join("court_wizard"))
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine data directory",
            )
            .into()
        })
}

/// Ensures the save directory exists, then writes data to the given filename.
/// Uses write-to-temp-then-rename for crash safety: a partial write can never
/// corrupt the primary file. The previous version is kept as a `.bak` backup.
fn save_file(filename: &str, data: &str) -> ConfigResult<()> {
    let dir = save_dir()?;
    std::fs::create_dir_all(&dir)?;

    let final_path = dir.join(filename);
    let tmp_path = dir.join(format!("{filename}.tmp"));
    let bak_path = dir.join(format!("{filename}.bak"));

    // Write to temp file first (if this crashes, primary is untouched)
    std::fs::write(&tmp_path, data)?;

    // Backup existing file (ignore NotFound — file may not exist yet)
    let _ = std::fs::rename(&final_path, &bak_path);

    // Atomic rename (atomic on same filesystem on Linux/macOS/Windows)
    std::fs::rename(&tmp_path, &final_path)?;

    Ok(())
}

/// Reads file contents from the save directory.
/// If the primary file is missing or corrupted, falls back to the `.bak` backup.
fn load_file(filename: &str) -> ConfigResult<String> {
    let dir = save_dir()?;
    let path = dir.join(filename);

    match std::fs::read_to_string(&path) {
        Ok(data) if !data.is_empty() => Ok(data),
        primary_err => {
            // Try backup file directly (no existence check — avoids TOCTOU race)
            let bak_path = dir.join(format!("{filename}.bak"));
            if let Ok(data) = std::fs::read_to_string(&bak_path)
                && !data.is_empty()
            {
                // Restore backup as primary so future loads succeed directly
                let _ = std::fs::write(&path, &data);
                return Ok(data);
            }
            // No backup available — return the original error
            match primary_err {
                Ok(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Save file is empty",
                )
                .into()),
                Err(e) => Err(e.into()),
            }
        }
    }
}

/// Deletes a file from the save directory if it exists.
fn delete_file(filename: &str) -> ConfigResult<()> {
    let dir = save_dir()?;
    let path = dir.join(filename);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Checks if a file exists in the save directory.
fn file_exists(filename: &str) -> bool {
    save_dir()
        .map(|dir| dir.join(filename).exists())
        .unwrap_or(false)
}

/// Returns the filename for a save slot.
fn save_slot_filename(slot: usize) -> String {
    format!("save_{slot}.dat")
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

pub(super) fn save_config(config_toml: &str) -> ConfigResult<()> {
    save_file(CONFIG_FILENAME, config_toml)
}

pub(super) fn load_config() -> ConfigResult<String> {
    load_file(CONFIG_FILENAME)
}

// ---------------------------------------------------------------------------
// Legacy progress
// ---------------------------------------------------------------------------

pub(super) fn load_progress() -> ConfigResult<String> {
    load_file(PROGRESS_FILENAME)
}

pub(super) fn delete_progress() -> ConfigResult<()> {
    delete_file(PROGRESS_FILENAME)
}

// ---------------------------------------------------------------------------
// Save slots
// ---------------------------------------------------------------------------

pub(super) fn save_slot(slot: usize, data: &str) -> ConfigResult<()> {
    save_file(&save_slot_filename(slot), data)
}

pub(super) fn load_slot(slot: usize) -> ConfigResult<String> {
    load_file(&save_slot_filename(slot))
}

pub(super) fn delete_slot(slot: usize) -> ConfigResult<()> {
    delete_file(&save_slot_filename(slot))
}

pub(super) fn slot_exists(slot: usize) -> bool {
    file_exists(&save_slot_filename(slot))
}

// ---------------------------------------------------------------------------
// Unified save file
// ---------------------------------------------------------------------------

pub(super) fn save_unified_save(data: &str) -> ConfigResult<()> {
    save_file(UNIFIED_SAVE_FILENAME, data)
}

pub(super) fn load_unified_save() -> ConfigResult<String> {
    load_file(UNIFIED_SAVE_FILENAME)
}

pub(super) fn unified_save_exists() -> bool {
    file_exists(UNIFIED_SAVE_FILENAME)
}
