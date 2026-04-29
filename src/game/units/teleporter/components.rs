use bevy::prelude::*;

#[derive(Component)]
pub struct Teleporter;

/// Channel state machine for the teleporter's ability.
#[derive(Component, Default)]
pub(in crate::game) enum TeleporterState {
    /// Moving toward the king; will start channeling when within range.
    #[default]
    Approaching,
    /// Channeling in place. When `elapsed` reaches `CHANNEL_DURATION`, fires the teleport.
    Channeling { elapsed: f32, indicator: Entity },
    /// Recovering after a teleport; resumes approaching when `remaining` reaches 0.
    Cooldown { remaining: f32 },
}
