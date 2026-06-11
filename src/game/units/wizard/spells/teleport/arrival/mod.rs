//! Teleport execution: scatter, swap, recall, unit teleport.

mod cleanup;
mod rift;
mod teleport_logic;

// pub items (used via `pub use super::arrival::*` in systems.rs)
pub use cleanup::cleanup_teleport_on_spell_switch;
pub use cleanup::update_circle_animations;
pub use rift::tick_dimensional_rift;
pub use rift::tick_lingering_gate;

// pub(super) items — visible to the teleport module (casting/ siblings need these)
pub(super) use cleanup::apply_post_teleport_effects;
pub(super) use teleport_logic::execute_teleport;

// pub(crate) items
pub(crate) use teleport_logic::teleport_units_with_radius;
