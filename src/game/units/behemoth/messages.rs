use bevy::prelude::*;

use crate::game::units::components::Team;

/// Message sent when a behemoth attacks, containing the target position for AOE damage.
#[derive(Message)]
pub struct BehemothAttackMessage {
    pub target_position: Vec3,
    pub behemoth_team: Team,
}
