//! Raise the Dead casting and corpse-raising logic.

mod input_casting;
mod raising;

pub(crate) use input_casting::compute_talent_params;
pub use input_casting::handle_raise_the_dead_casting;
pub(crate) use raising::{find_nearest_corpse, raise_corpse_as_undead};
