pub(in crate::game::units) mod components;
pub(in crate::game::units) mod constants;
mod plugin;
pub(in crate::game::units) mod systems;

pub use plugin::ElitePlugin;

// Re-export components for external use
pub use components::{EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};

// Re-export constants for convenience
#[allow(unused_imports)]
pub use constants::{ELITE_DAMAGE_BONUS, ELITE_HEALTH_BONUS, ELITE_SPEED_BONUS};
