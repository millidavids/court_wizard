//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send unit state snapshots to the guest every frame.

mod crdt_receive;
mod dispel_receive;
mod entity_id_assign;
mod king_death;
mod message_receive;
mod snapshot_send;
mod status_receive;

pub use crdt_receive::receive_crdt_snapshot;
pub use dispel_receive::receive_dispel_messages;
pub use entity_id_assign::assign_network_ids;
pub use king_death::check_mp_king_death;
pub(crate) use king_death::end_mp_match;
pub use message_receive::{
    receive_mp_forfeit, receive_raise_corpse_messages, receive_spell_hit_messages,
    receive_teleport_message,
};
pub use snapshot_send::send_state_snapshots;
pub use status_receive::receive_apply_status_effect;
