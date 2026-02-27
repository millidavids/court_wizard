use bevy::prelude::*;

/// Broadcast message requesting a CRT channel-change flicker effect.
///
/// Send this alongside any state transition that should show the effect:
/// ```ignore
/// commands.send(ChannelChangeMessage);
/// ```
#[derive(Message)]
pub(crate) struct ChannelChangeMessage;
