//! Lightning Rod spell module.
//!
//! Places a lightning rod tower on the battlefield that periodically attracts
//! lightning strikes from the sky. When lightning hits the rod, arcs of
//! electricity jump to all nearby units, damaging friend and foe alike.

mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub use plugin::LightningRodPlugin;
