use bevy::prelude::*;

/// Marker component for infantry units.
#[derive(Component)]
pub struct Infantry;

/// Resource tracking whether defenders should be active.
///
/// Defenders share activation - once any attacker gets within range,
/// all defenders activate and start moving.
#[derive(Resource, Default)]
pub struct DefendersActivated {
    #[allow(clippy::struct_field_names)]
    pub active: bool,
}

/// Tracks the King's retreat state.
///
/// When the King gets too close to the wizard's max spell range,
/// he sounds a retreat. Defenders disengage and fall back to spawn.
/// Number of retreats per level equals `tier + 1`.
#[derive(Resource)]
pub struct RetreatState {
    /// Remaining retreat duration (seconds). Active when > 0.
    pub retreat_timer: f32,
    /// Number of retreats remaining this level.
    pub retreats_remaining: u32,
}

impl Default for RetreatState {
    fn default() -> Self {
        Self {
            retreat_timer: 0.0,
            retreats_remaining: 1,
        }
    }
}

impl RetreatState {
    /// Returns true if retreat is currently active.
    pub fn is_active(&self) -> bool {
        self.retreat_timer > 0.0
    }

    /// Returns true if the King has retreats available.
    pub fn can_retreat(&self) -> bool {
        self.retreats_remaining > 0
    }
}

/// Marker component for units currently retreating (suppresses attacks).
#[derive(Component)]
pub struct Retreating;
