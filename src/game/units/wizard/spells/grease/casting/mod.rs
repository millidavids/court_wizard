//! Grease casting and slip effect.

mod input;
mod obstacle;
mod slow;
mod talents;

pub use input::handle_grease_casting;
pub(super) use obstacle::write_grease_obstacle;
pub use slow::apply_grease_slow;
