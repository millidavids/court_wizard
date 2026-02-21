//! Spells plugin module.
//!
//! Handles wizard spells, projectiles, and spell effects.

pub(crate) mod arcane_crystal;
mod banishment;
mod battle_hymn;
mod berserker_rage;
pub(crate) mod black_hole;
pub(crate) mod chain_lightning;
mod components;
pub(crate) mod disintegrate;
pub(crate) mod entangle;
pub(crate) mod finger_of_death;
pub(crate) mod fireball;
pub(crate) mod fog_cloud;
pub(crate) mod grease;
mod guardian_circle;
mod haste;
pub(crate) mod healing_plume;
mod hypnotic_pattern;
pub(crate) mod lightning_rod;
pub(crate) mod magic_missile;
mod mark_of_death;
pub(crate) mod meteor_fall;
mod phantasmal_force;
pub(crate) mod plague_wind;
mod plugin;
mod polymorph;
mod raise_the_dead;
pub mod run_conditions;
mod sleep;
pub(crate) mod spike_growth;
pub(crate) mod squall;
mod systems;
mod telekinesis;
mod teleport;
pub(crate) mod wall_of_fire;
pub(crate) mod wall_of_stone;

// Re-export constants for wizard setup and spell switching
pub(in crate::game::units::wizard) use arcane_crystal::constants as arcane_crystal_constants;
pub(in crate::game::units::wizard) use banishment::constants as banishment_constants;
pub(in crate::game::units::wizard) use battle_hymn::constants as battle_hymn_constants;
pub(in crate::game::units::wizard) use berserker_rage::constants as berserker_rage_constants;
pub(in crate::game::units::wizard) use black_hole::constants as black_hole_constants;
pub(in crate::game::units::wizard) use chain_lightning::constants as chain_lightning_constants;
pub(in crate::game::units::wizard) use disintegrate::constants as disintegrate_constants;
pub(in crate::game::units::wizard) use entangle::constants as entangle_constants;
pub(in crate::game::units::wizard) use finger_of_death::constants as finger_of_death_constants;
pub(in crate::game::units::wizard) use fireball::constants as fireball_constants;
pub(in crate::game::units::wizard) use fog_cloud::constants as fog_cloud_constants;
pub(in crate::game::units::wizard) use grease::constants as grease_constants;
pub(in crate::game::units::wizard) use guardian_circle::constants as guardian_circle_constants;
pub(in crate::game::units::wizard) use haste::constants as haste_constants;
pub(in crate::game::units::wizard) use healing_plume::constants as healing_plume_constants;
pub(in crate::game::units::wizard) use hypnotic_pattern::constants as hypnotic_pattern_constants;
pub(in crate::game::units::wizard) use lightning_rod::constants as lightning_rod_constants;
pub(in crate::game) use magic_missile::constants as magic_missile_constants;
pub(in crate::game::units::wizard) use mark_of_death::constants as mark_of_death_constants;
pub(in crate::game::units::wizard) use meteor_fall::constants as meteor_fall_constants;
pub(in crate::game::units::wizard) use phantasmal_force::constants as phantasmal_force_constants;
pub(in crate::game::units::wizard) use plague_wind::constants as plague_wind_constants;
pub(in crate::game::units::wizard) use polymorph::constants as polymorph_constants;
pub(in crate::game::units::wizard) use raise_the_dead::constants as raise_the_dead_constants;
pub(in crate::game::units::wizard) use sleep::constants as sleep_constants;
pub(in crate::game::units::wizard) use spike_growth::constants as spike_growth_constants;
pub(in crate::game::units::wizard) use squall::constants as squall_constants;
pub(in crate::game::units::wizard) use telekinesis::constants as telekinesis_constants;
pub(in crate::game::units::wizard) use teleport::constants as teleport_constants;
pub(in crate::game::units::wizard) use wall_of_fire::constants as wall_of_fire_constants;
pub(in crate::game::units::wizard) use wall_of_stone::constants as wall_of_stone_constants;

pub use plugin::SpellsPlugin;
