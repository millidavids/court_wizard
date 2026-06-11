//! Archer targeting, movement, and spawn helpers.

mod kiting;
mod spawning;
mod targeting;

pub use kiting::archer_movement;
pub(in crate::game) use spawning::spawn_single_attacker_archer;
pub(in crate::game) use spawning::spawn_single_defender_archer;
pub use targeting::update_archer_targeting;
