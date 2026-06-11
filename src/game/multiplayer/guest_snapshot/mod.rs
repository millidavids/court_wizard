//! Guest snapshot application and CRDT sync.

pub(crate) mod apply_state_snapshot;
pub(crate) mod crdt_apply;
pub(crate) mod status_forwarding;

pub use apply_state_snapshot::apply_state_snapshot;
pub use crdt_apply::send_crdt_snapshot;
pub use status_forwarding::{
    cleanup_forwarded_marker, forward_spell_hits_to_host, forward_status_effects_to_host,
};
