//! Archer combat: melee + ranged.

mod melee;
mod ranged;

pub use melee::archer_melee_combat;
pub use ranged::archer_ranged_combat;
pub use ranged::update_archer_movement_timers;
pub(crate) use ranged::wall_near_approach_path;
