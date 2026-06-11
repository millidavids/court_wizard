mod combat;
mod movement;
mod spawn;

pub(in crate::game) use combat::brute_rock_throw;
pub(in crate::game) use movement::{brute_movement, update_brute_targeting};
pub(in crate::game) use spawn::spawn_brute;
