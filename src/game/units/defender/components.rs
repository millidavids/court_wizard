use bevy::prelude::*;

/// Marker component for defender units (friendly).
#[derive(Component)]
pub struct Defender;

/// Resource tracking whether defenders should be active.
///
/// Defenders share activation - once any attacker gets within range,
/// all defenders activate and start moving.
#[derive(Resource, Default)]
pub struct DefendersActivated {
    pub active: bool,
}
