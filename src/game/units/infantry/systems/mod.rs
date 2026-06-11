mod activation;
mod spawn;
mod targeting;

pub use activation::{check_defender_activation, check_retreat_trigger};
pub(in crate::game) use spawn::{
    spawn_single_attacker, spawn_single_defender, spawn_single_kings_guard,
};
pub use targeting::{infantry_movement, update_infantry_targeting};
