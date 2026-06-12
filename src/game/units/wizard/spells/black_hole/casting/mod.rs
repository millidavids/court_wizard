//! Black hole casting and spawn.

mod cast_logic;
mod spawn;
mod talents;

pub(super) use cast_logic::handle_black_hole_casting;
pub(crate) use spawn::spawn_black_hole;
