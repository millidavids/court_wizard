//! Wall of stone casting and spawn.

mod placement;
mod talents;
mod wizard_system;

pub(crate) use talents::compute_talent_params;
pub use wizard_system::handle_wall_of_stone_casting;
