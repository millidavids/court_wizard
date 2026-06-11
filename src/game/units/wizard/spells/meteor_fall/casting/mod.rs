//! Meteor fall casting and projectile spawn.

mod input_casting;
mod projectile_spawn;

pub(crate) use input_casting::handle_meteor_fall_casting;
pub(crate) use projectile_spawn::{
    MeteorProjectileTalentFlags, spawn_explosion_entity, spawn_meteor_projectile_entity,
    spawn_meteor_projectiles,
};
