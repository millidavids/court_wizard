//! Squall ice projectiles: spawn, movement, collisions.
//!
//! Split into concern-focused siblings; this file is re-exports only.

mod absolute_zero;
mod blizzard;
mod evasion;
mod explosion;
mod projectile;
mod snow;

pub(crate) use absolute_zero::{
    decay_absolute_zero_slow, end_absolute_zero_on_release, update_absolute_zero,
};
pub(crate) use blizzard::update_blizzard_position;
pub(crate) use evasion::apply_sleet_storm_evasion;
pub(crate) use explosion::{update_frozen_ground, update_ice_explosions, update_storm_ring};
pub(crate) use projectile::{
    check_ice_projectile_collisions, spawn_ice_projectiles, update_ice_projectiles,
};
pub(crate) use snow::{spawn_snow_particles, update_snow_particles};
