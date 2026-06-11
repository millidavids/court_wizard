//! Dark mage spawn, movement, and AI.

mod ai_core;
mod movement;
mod spawn;
mod teleport;

pub use ai_core::dark_mage_ai;
pub use movement::{dark_mage_movement, dark_mage_spell_queue};
pub use spawn::spawn_dark_mage;
pub use teleport::dark_mage_teleport;
