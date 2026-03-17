use bevy::prelude::*;

/// Emitted when an enraged unit with ContagiousRage kills an enemy.
/// The spread system reads these to propagate rage to nearby allies.
#[derive(Message)]
pub(crate) struct ContagiousRageKillMessage {
    pub killer: Entity,
}
