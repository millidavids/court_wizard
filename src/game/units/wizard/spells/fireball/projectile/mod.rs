//! Fireball projectile movement, collisions, and explosion.

mod explosion_lifecycle;
mod explosion_spawn;
mod movement;
mod trail_effects;

pub use explosion_lifecycle::*;
pub(crate) use explosion_spawn::spawn_explosion_with_talents;
pub use movement::*;
pub use trail_effects::*;
