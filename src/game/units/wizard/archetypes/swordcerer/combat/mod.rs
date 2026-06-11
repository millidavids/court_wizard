//! Swordcerer combat: state, movement, and weapon firing.

mod avatar_movement;
mod missile;
mod sword_arc;

pub(crate) use avatar_movement::{
    apply_avatar_physics, block_spells_on_field, handle_retreat, player_movement,
    reset_swordcerer_state,
};
pub(crate) use missile::{fire_missile, spawn_avatar_missile};
pub(crate) use sword_arc::{spawn_sword_arc, sword_swing, update_sword_arcs};
