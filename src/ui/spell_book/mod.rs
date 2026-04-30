//! Spell book UI module.
//!
//! Displays spell selection screen when player opens the spell book.

mod components;
mod constants;
pub(super) mod interaction;
mod plugin;
pub(super) mod setup;
mod systems;

pub use plugin::SpellBookPlugin;

// Re-exports for tutorial system
pub(crate) use components::{DetailName, HotkeySlotButton, ScrollableSpellList};
