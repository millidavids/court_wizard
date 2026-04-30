//! Re-export hub for VFX systems split into feature files (Phase 17).
//!
//! Original 1568-line file split into:
//! - `fire_effects.rs` — fire glow, smoke wisps, sparks
//! - `explosion_effects.rs` — explosion smoke, missile glow/sparkles
//! - `area_effects.rs` — heat shimmer, plague/fog smoke, fire variants, embers
//! - `cast_effects.rs` — cast flares, motes, smoke poofs, dust, aura bubbles

pub use super::area_effects::*;
pub use super::cast_effects::*;
pub use super::explosion_effects::*;
pub use super::fire_effects::*;
