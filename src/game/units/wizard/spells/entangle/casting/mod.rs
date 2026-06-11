//! Entangle casting and zone setup.

mod cast_input;
mod ground_effect;
mod root_effects;

pub use cast_input::handle_entangle_casting;
pub use ground_effect::{
    cleanup_entangle_ground_effect, overgrowth_root_new_units, tick_entangle_ground_effect,
};
pub(crate) use root_effects::apply_entangle_to_unit;
pub use root_effects::{
    handle_entangle_root_expire, nourishing_roots_mana_regen, thorny_vines_tick,
};
