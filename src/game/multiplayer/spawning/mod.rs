//! Multiplayer entity spawn helpers.

mod archer;
mod castle;
mod infantry;
mod king;
mod utils;
mod wizard;

pub(in crate::game::multiplayer) use archer::spawn_mp_archer;
pub(in crate::game::multiplayer) use castle::spawn_castle;
pub(in crate::game::multiplayer) use infantry::{spawn_mp_infantry, spawn_mp_kings_guard};
pub(in crate::game::multiplayer) use king::spawn_mp_king;
pub(in crate::game) use wizard::spawn_mp_wizard;
