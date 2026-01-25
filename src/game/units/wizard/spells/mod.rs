//! Spells plugin module.
//!
//! Handles wizard spells, projectiles, and spell effects.

mod components;
mod disintegrate;
mod fireball;
mod guardian_circle;
mod magic_missile;
mod plugin;
mod systems;

// Re-export constants for wizard setup and spell switching
pub use disintegrate::constants as disintegrate_constants;
pub use fireball::constants as fireball_constants;
pub use guardian_circle::constants as guardian_circle_constants;
pub use magic_missile::constants as magic_missile_constants;

pub use plugin::SpellsPlugin;
