mod behaviors;
mod casting;
mod core;
mod livestock;
mod shared;

pub use behaviors::{handle_pig_movement, tick_dire_sheep};
pub(crate) use casting::handle_polymorph_casting;
pub use livestock::{apply_sheep_visual, check_explosive_sheep_deaths, tick_polymorphed_units};
