mod aura_vfx;
mod movement;
mod spawn;
mod spell_shield;

pub use aura_vfx::{spawn_commander_aura_particles, update_commander_aura_particles};
pub use movement::{
    king_cohesion_force, king_movement, snap_kings_guard_to_king, update_king_targeting,
};
pub(in crate::game) use spawn::spawn_king_aura_visual;
pub use spawn::{despawn_king_aura_on_death, spawn_king};
pub use spell_shield::{
    attach_king_spell_shield, expire_king_spell_shield_on_timeout, update_king_spell_shield,
};
