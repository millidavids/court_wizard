use std::path::PathBuf;

use super::error::ConfigResult;

const CONFIG_FILENAME: &str = "config.toml";
const PROGRESS_FILENAME: &str = "progress.dat";
const UNIFIED_SAVE_FILENAME: &str = "saves_v2.json";
const LAN_IP_FILENAME: &str = "lan_ip.txt";

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
fn save_file(filename: &str, data: &str) -> ConfigResult<()> {
    let dir = save_dir()?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(filename), data)?;
    Ok(())
}

/// Reads file contents from the save directory.
fn load_file(filename: &str) -> ConfigResult<String> {
    let dir = save_dir()?;
    let data = std::fs::read_to_string(dir.join(filename))?;
    Ok(data)
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

#[allow(dead_code)]
pub(super) fn clear_config() -> ConfigResult<()> {
    delete_file(CONFIG_FILENAME)
}

// ---------------------------------------------------------------------------
// Legacy progress
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub(super) fn save_progress(data: &str) -> ConfigResult<()> {
    save_file(PROGRESS_FILENAME, data)
}

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

// ---------------------------------------------------------------------------
// LAN IP address
// ---------------------------------------------------------------------------

pub(super) fn save_lan_ip(ip: &str) -> ConfigResult<()> {
    save_file(LAN_IP_FILENAME, ip)
}

pub(super) fn load_lan_ip() -> ConfigResult<String> {
    load_file(LAN_IP_FILENAME)
}
