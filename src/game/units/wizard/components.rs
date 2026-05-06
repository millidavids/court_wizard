//! Re-export hub for wizard components.
//!
//! Original definitions split into `spell_enum.rs` (Spell/SpellCategory enums + their
//! methods) and `wizard_state.rs` (Wizard, Mana, casting state, animation, assets).

pub use super::spell_enum::*;
pub(crate) use super::spell_status_effects::compose_spell_description;
pub use super::wizard_state::*;
