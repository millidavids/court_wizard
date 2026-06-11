//! Chain lightning casting and arc spawning.

mod arc_spawn;
mod spell_cast;
mod talent_config;

pub(crate) use arc_spawn::spawn_arc;
pub use spell_cast::handle_chain_lightning_casting;
