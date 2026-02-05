use crate::game::units::components::Team;
use bevy::prelude::*;

/// Marker component for behemoth units.
#[derive(Component)]
pub struct Behemoth;

/// Message sent when a behemoth attacks, containing the target position for AOE damage.
#[derive(Message)]
pub struct BehemothAttackEvent {
    pub target_position: Vec3,
    pub behemoth_team: Team,
}
