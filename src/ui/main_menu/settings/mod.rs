//! Settings menu UI module.
//!
//! Contains the settings menu screen.

pub(crate) mod components;
mod constants;
mod controller_diagrams;
#[cfg(debug_assertions)]
pub(crate) mod debug_unlock;
pub(super) mod plugin;

// Systems are split into submodules but re-exported for convenience
pub(super) mod builders;
pub(super) mod interaction;
pub(crate) mod systems;
