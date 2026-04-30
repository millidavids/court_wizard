pub(in crate::game::units) mod components;
pub(in crate::game::units) mod constants;
mod plugin;
pub(in crate::game::units) mod systems;

pub use plugin::ElitePlugin;

pub use components::{EliteAttackSpeedBonus, EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};
pub use constants::{
    ELITE_ATTACK_SPEED_BONUS, ELITE_DAMAGE_BONUS, ELITE_HEALTH_BONUS, ELITE_SPEED_BONUS,
};
