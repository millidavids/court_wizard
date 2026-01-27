//! Spells plugin module.
//!
//! Handles wizard spells, projectiles, and spell effects.

mod chain_lightning;
mod components;
mod disintegrate;
mod finger_of_death;
mod fireball;
mod guardian_circle;
mod magic_missile;
mod plugin;
mod raise_the_dead;
pub mod run_conditions;
mod systems;
mod teleport;

// Re-export constants for wizard setup and spell switching
pub use chain_lightning::constants as chain_lightning_constants;
pub use disintegrate::constants as disintegrate_constants;
pub use finger_of_death::constants as finger_of_death_constants;
pub use fireball::constants as fireball_constants;
pub use guardian_circle::constants as guardian_circle_constants;
pub use magic_missile::constants as magic_missile_constants;
pub use raise_the_dead::constants as raise_the_dead_constants;
pub use teleport::constants as teleport_constants;

pub use plugin::SpellsPlugin;
