use web_sys::window;

const CONFIG_KEY: &str = "the_game_config";

/// Saves config string to browser localStorage.
///
/// # Arguments
///
/// * `config_toml` - TOML-formatted configuration string
///
/// # Returns
///
/// `Ok(())` on success, `Err(String)` with error message on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - Setting the item fails
pub fn save_config(config_toml: &str) -> Result<(), String> {
    let window = window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .set_item(CONFIG_KEY, config_toml)
        .map_err(|_| "Failed to save to localStorage".to_string())
}

/// Loads config string from browser localStorage.
///
/// # Returns
///
/// `Ok(String)` containing TOML config on success, `Err(String)` with error message on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - No config is found in localStorage
/// - Reading the item fails
pub fn load_config() -> Result<String, String> {
    let window = window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .get_item(CONFIG_KEY)
        .map_err(|_| "Failed to read from localStorage".to_string())?
        .ok_or("No config found in localStorage".to_string())
}

/// Clears config from localStorage.
///
/// # Returns
///
/// `Ok(())` on success, `Err(String)` with error message on failure
///
/// # Errors
///
/// Returns an error if:
/// - Window object is not available
/// - localStorage API is not available
/// - Removing the item fails
#[allow(dead_code)]
pub fn clear_config() -> Result<(), String> {
    let window = window().ok_or("No window object")?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("localStorage not available")?;

    storage
        .remove_item(CONFIG_KEY)
        .map_err(|_| "Failed to clear localStorage".to_string())
}
