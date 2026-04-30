//! Lightning Rod spell module.
//!
//! Places a lightning rod tower on the battlefield that periodically attracts
//! lightning strikes from the sky. When lightning hits the rod, arcs of
//! electricity jump to all nearby units, damaging friend and foe alike.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod lifecycle;
mod plugin;
pub(crate) mod systems;

pub(crate) use components::LightningRod;
pub use plugin::LightningRodPlugin;
