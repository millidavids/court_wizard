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
pub fn save_config(config_toml: &str) -> ConfigResult<()> {
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
pub fn load_config() -> ConfigResult<String> {
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
pub fn clear_config() -> ConfigResult<()> {
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
