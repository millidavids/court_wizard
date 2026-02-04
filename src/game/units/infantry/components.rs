use bevy::prelude::*;

/// Marker component for infantry units.
#[derive(Component)]
pub struct Infantry;

/// Resource tracking whether defenders should be active.
///
/// Defenders share activation - once any attacker gets within range,
/// all defenders activate and start moving.
#[derive(Resource)]
pub struct DefendersActivated {
    pub active: bool,
}

impl Default for DefendersActivated {
    fn default() -> Self {
        Self {
            active: false, // Start inactive - defenders wait for enemies to get close
        }
    }
}
