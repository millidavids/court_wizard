//! Meteor projectile movement, smoke trails, and collisions.

mod explosion;
mod ground_fire;
mod projectile;

pub(super) use explosion::{animate_meteor_explosions, apply_meteor_explosion_damage};
pub(super) use ground_fire::{
    apply_ground_fire_damage, cleanup_ground_fire, fade_ground_fire, spawn_ground_fire_particles,
};
pub(super) use projectile::{
    check_meteor_collisions, find_nearest_non_defender_xz, process_extinction_event,
    spawn_meteor_smoke_trail, update_meteor_projectiles,
};
