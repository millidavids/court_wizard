mod combat;
mod lifetime;
mod projectile;
mod spawn;
mod tint;

pub use combat::{apply_spell_damage_to_rocks, units_attack_blocking_rocks};
pub use lifetime::{
    cleanup_sunk_rocks, destroy_dead_rocks, sync_teleported_rocks, tick_rock_lifetime,
};
pub use projectile::{animate_rock_projectiles, spawn_rock_projectile};
pub(in crate::game) use spawn::spawn_terrain_boulder;
pub use tint::{cleanup_rock_shadows, tick_boulder_heat, update_rock_damage_tint};
