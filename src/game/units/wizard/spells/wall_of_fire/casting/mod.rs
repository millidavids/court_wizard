//! Wall of fire casting and spawn.

mod placement;
mod spell_casting;
mod state_machine;

pub(crate) use placement::wall_obstacle_bounds;
pub use spell_casting::handle_wall_of_fire_casting;
