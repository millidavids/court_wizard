//! Black hole gravity, damage, and persistent effects.

mod accretion;
mod collapse;
mod pull;

pub(super) use accretion::{
    apply_black_hole_damage, apply_crushing_pressure, apply_dimensional_rift,
    remove_units_from_black_hole,
};
pub(super) use collapse::{
    cleanup_black_hole_sfx, despawn_expired_black_holes, update_black_hole_visuals,
};
pub(super) use pull::{apply_corpse_gravity_and_despawn, apply_gravitational_forces};
