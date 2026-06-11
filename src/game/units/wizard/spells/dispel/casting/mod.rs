//! Dispel casting and projectile spawn.

mod input;
mod projectile;

pub use input::handle_dispel_casting;
pub(crate) use input::spawn_dispel_projectile;
pub use projectile::move_dispel_projectiles;
