use bevy::prelude::*;

/// Marker component for teleporter attacker units.
#[derive(Component)]
pub(in crate::game) struct Teleporter;

/// Channel state machine for the teleporter's ability.
#[derive(Component)]
pub(in crate::game) enum TeleporterState {
    /// Moving toward the king; will start channeling when within range.
    Approaching,
    /// Channeling in place. When `elapsed` reaches `CHANNEL_DURATION`, fires the teleport.
    Channeling { elapsed: f32, indicator: Entity },
    /// Recovering after a teleport; resumes approaching when `remaining` reaches 0.
    Cooldown { remaining: f32 },
}

impl Default for TeleporterState {
    fn default() -> Self {
        Self::Approaching
    }
}
