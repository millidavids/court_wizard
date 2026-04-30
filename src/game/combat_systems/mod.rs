//! Combat systems split by concern (Phase 18).

mod melee;
mod post_combat;

pub use melee::combat;
pub use post_combat::{convert_dead_to_corpses, enforce_invulnerability};
