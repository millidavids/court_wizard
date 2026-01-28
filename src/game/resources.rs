use bevy::prelude::*;

use super::units::components::Team;

/// Tracks kill statistics throughout the game for the score screen.
#[derive(Resource, Default)]
pub struct KillStats {
    pub defenders_killed: u32,
    pub attackers_killed: u32,
    pub undead_killed: u32,
}

impl KillStats {
    pub fn record_kill(&mut self, team: Team) {
        match team {
            Team::Defenders => self.defenders_killed += 1,
            Team::Attackers => self.attackers_killed += 1,
            Team::Undead => self.undead_killed += 1,
        }
    }

    pub fn reset(&mut self) {
        self.defenders_killed = 0;
        self.attackers_killed = 0;
        self.undead_killed = 0;
    }
}

/// Tracks whether the player won or lost the game.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Victory, // Player wins (all attackers and undead eliminated)
    Defeat,  // Player loses (all defenders eliminated)
}

/// Current difficulty level - scales enemy spawn counts.
/// Level 1 is base difficulty, higher levels spawn more attackers.
#[derive(Resource)]
pub struct CurrentLevel(pub u32);

impl Default for CurrentLevel {
    fn default() -> Self {
        Self(1)
    }
}
