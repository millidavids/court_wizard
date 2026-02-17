//! Spells plugin module.
//!
//! Handles wizard spells, projectiles, and spell effects.

mod black_hole;
mod chain_lightning;
mod components;
mod disintegrate;
mod entangle;
mod finger_of_death;
mod fireball;
mod guardian_circle;
mod haste;
mod lightning_rod;
mod magic_missile;
mod plugin;
mod raise_the_dead;
pub mod run_conditions;
mod spike_growth;
mod squall;
mod systems;
mod teleport;
mod wall_of_fire;
pub(in crate::game) mod wall_of_stone;

// Re-export constants for wizard setup and spell switching
pub(in crate::game::units::wizard) use black_hole::constants as black_hole_constants;
pub(in crate::game::units::wizard) use chain_lightning::constants as chain_lightning_constants;
pub(in crate::game::units::wizard) use disintegrate::constants as disintegrate_constants;
pub(in crate::game::units::wizard) use entangle::constants as entangle_constants;
pub(in crate::game::units::wizard) use finger_of_death::constants as finger_of_death_constants;
pub(in crate::game::units::wizard) use fireball::constants as fireball_constants;
pub(in crate::game::units::wizard) use guardian_circle::constants as guardian_circle_constants;
pub(in crate::game::units::wizard) use haste::constants as haste_constants;
pub(in crate::game::units::wizard) use lightning_rod::constants as lightning_rod_constants;
pub(in crate::game::units::wizard) use magic_missile::constants as magic_missile_constants;
pub(in crate::game::units::wizard) use raise_the_dead::constants as raise_the_dead_constants;
pub(in crate::game::units::wizard) use spike_growth::constants as spike_growth_constants;
pub(in crate::game::units::wizard) use squall::constants as squall_constants;
pub(in crate::game::units::wizard) use teleport::constants as teleport_constants;
pub(in crate::game::units::wizard) use wall_of_fire::constants as wall_of_fire_constants;
pub(in crate::game::units::wizard) use wall_of_stone::constants as wall_of_stone_constants;

pub use plugin::SpellsPlugin;
