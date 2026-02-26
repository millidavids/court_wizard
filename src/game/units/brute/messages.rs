use bevy::prelude::*;

use crate::game::units::components::Team;

/// Message sent when a brute attacks, containing the target position for AOE damage.
#[derive(Message)]
pub struct BruteAttackMessage {
    pub target_position: Vec3,
    pub brute_team: Team,
}
