//! Post-combat reactions: corpse conversion (cleanup.rs) and invulnerability enforcement (death_conversion.rs).

mod cleanup;
mod death_conversion;

pub use cleanup::convert_dead_to_corpses;
pub use death_conversion::enforce_invulnerability;
