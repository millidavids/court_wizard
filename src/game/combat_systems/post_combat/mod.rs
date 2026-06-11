//! Post-combat reactions: invulnerability and corpse conversion.

mod cleanup;
mod death_conversion;

pub use cleanup::convert_dead_to_corpses;
pub use death_conversion::enforce_invulnerability;
