use web_sys::window;

use super::error::ConfigResult;

const CONFIG_KEY: &str = "court_wizard_config";

/// Saves config string to browser localStorage.
///
/// # Arguments
///
/// * `config_toml` - TOML-formatted configuration string
///
/// # Returns
///
/// `Ok(())` on success, `Err(ConfigError)` on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - Setting the item fails
pub(super) fn save_config(config_toml: &str) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .set_item(CONFIG_KEY, config_toml)
        .map_err(|_| std::io::Error::other("Failed to save to localStorage"))?;
    Ok(())
}

/// Loads config string from browser localStorage.
///
/// # Returns
///
/// `Ok(String)` containing TOML config on success, `Err(ConfigError)` on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - No config is found in localStorage
/// - Reading the item fails
pub(super) fn load_config() -> ConfigResult<String> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    let config = storage
        .get_item(CONFIG_KEY)
        .map_err(|_| std::io::Error::other("Failed to read from localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No config found in localStorage",
            )
        })?;

    Ok(config)
}

const PROGRESS_KEY: &str = "court_wizard_progress";

/// Saves signed progress string to browser localStorage.
#[allow(dead_code)]
pub(super) fn save_progress(data: &str) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .set_item(PROGRESS_KEY, data)
        .map_err(|_| std::io::Error::other("Failed to save progress to localStorage"))?;
    Ok(())
}

/// Loads signed progress string from browser localStorage.
pub(super) fn load_progress() -> ConfigResult<String> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    let data = storage
        .get_item(PROGRESS_KEY)
        .map_err(|_| std::io::Error::other("Failed to read progress from localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No progress found in localStorage",
            )
        })?;

    Ok(data)
}

/// Returns the localStorage key for a save slot.
fn save_slot_key(slot: usize) -> String {
    format!("court_wizard_save_{slot}")
}

/// Saves signed save data to a specific slot in localStorage.
pub(super) fn save_slot(slot: usize, data: &str) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .set_item(&save_slot_key(slot), data)
        .map_err(|_| std::io::Error::other("Failed to save slot to localStorage"))?;
    Ok(())
}

/// Loads signed save data from a specific slot in localStorage.
pub(super) fn load_slot(slot: usize) -> ConfigResult<String> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    let data = storage
        .get_item(&save_slot_key(slot))
        .map_err(|_| std::io::Error::other("Failed to read slot from localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "No save found in slot")
        })?;

    Ok(data)
}

/// Deletes a save slot from localStorage.
pub(super) fn delete_slot(slot: usize) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .remove_item(&save_slot_key(slot))
        .map_err(|_| std::io::Error::other("Failed to delete slot from localStorage"))?;
    Ok(())
}

/// Checks if a save slot exists in localStorage.
pub(super) fn slot_exists(slot: usize) -> bool {
    let Ok(window) = window().ok_or(()) else {
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return false;
    };
    matches!(storage.get_item(&save_slot_key(slot)), Ok(Some(_)))
}

/// Deletes the legacy progress key from localStorage (used during migration).
pub(super) fn delete_progress() -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .remove_item(PROGRESS_KEY)
        .map_err(|_| std::io::Error::other("Failed to delete progress from localStorage"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Unified save file storage
// ---------------------------------------------------------------------------

const UNIFIED_SAVE_KEY: &str = "court_wizard_saves_v2";

/// Saves the unified save data to localStorage.
pub(super) fn save_unified_save(data: &str) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .set_item(UNIFIED_SAVE_KEY, data)
        .map_err(|_| std::io::Error::other("Failed to save unified save to localStorage"))?;
    Ok(())
}

/// Loads the unified save data from localStorage.
pub(super) fn load_unified_save() -> ConfigResult<String> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    let data = storage
        .get_item(UNIFIED_SAVE_KEY)
        .map_err(|_| std::io::Error::other("Failed to read unified save from localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No unified save found in localStorage",
            )
        })?;

    Ok(data)
}

/// Checks if a unified save file exists in localStorage.
pub(super) fn unified_save_exists() -> bool {
    let Ok(window) = window().ok_or(()) else {
        return false;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return false;
    };
    matches!(storage.get_item(UNIFIED_SAVE_KEY), Ok(Some(_)))
}

// ---------------------------------------------------------------------------
// LAN IP address storage (plain string, no obfuscation)
// ---------------------------------------------------------------------------

const LAN_IP_KEY: &str = "court_wizard_lan_ip";

/// Saves the LAN IP address to localStorage.
pub(super) fn save_lan_ip(ip: &str) -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .set_item(LAN_IP_KEY, ip)
        .map_err(|_| std::io::Error::other("Failed to save LAN IP to localStorage"))?;
    Ok(())
}

/// Loads the saved LAN IP address from localStorage.
pub(super) fn load_lan_ip() -> ConfigResult<String> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    let data = storage
        .get_item(LAN_IP_KEY)
        .map_err(|_| std::io::Error::other("Failed to read LAN IP from localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No LAN IP found in localStorage",
            )
        })?;

    Ok(data)
}

/// Clears config from localStorage.
///
/// # Returns
///
/// `Ok(())` on success, `Err(ConfigError)` on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - Removing the item fails
#[allow(dead_code)]
pub(super) fn clear_config() -> ConfigResult<()> {
    let window = window()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No window object"))?;
    let storage = window
        .local_storage()
        .map_err(|_| std::io::Error::other("Failed to get localStorage"))?
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "localStorage not available")
        })?;

    storage
        .remove_item(CONFIG_KEY)
        .map_err(|_| std::io::Error::other("Failed to clear localStorage"))?;
    Ok(())
}
