//! Grease ignition, fire spread, and burn application.

mod burn;
mod lifecycle;
mod spawn;
mod vfx;

pub use burn::*;
pub use lifecycle::*;
pub(crate) use spawn::spawn_grease_zone;
pub use vfx::*;
