mod combat;
mod movement;
mod spawn;

pub(crate) use combat::{aerialist_combat, update_aerialist_targeting};
pub(crate) use movement::{aerialist_movement, clamp_aerialist_height};
pub(in crate::game) use spawn::spawn_single_attacker_aerialist;
