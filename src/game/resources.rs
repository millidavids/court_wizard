use bevy::prelude::*;

use super::units::components::Team;

/// Tracks kill statistics throughout the game for the score screen.
#[derive(Resource, Default)]
pub struct KillStats {
    pub defenders_killed: u32,
    pub attackers_killed: u32,
    pub undead_killed: u32,
    /// Number of defenders killed by the player's own spells this battle.
    pub defenders_killed_by_spell: u32,
    /// Total number of attackers spawned at the start of this battle.
    pub total_attackers_spawned: u32,
    /// Elapsed game time in seconds since the battle started.
    pub elapsed_time: f32,
    /// Time of the first defender death (None if no defenders have died yet).
    pub first_defender_death_time: Option<f32>,
    /// Whether the king was killed by a spell (for Accidental Regicide).
    pub king_killed_by_spell: bool,
}

impl KillStats {
    pub fn record_kill(&mut self, team: Team) {
        match team {
            Team::Defenders => {
                self.defenders_killed += 1;
                if self.first_defender_death_time.is_none() {
                    self.first_defender_death_time = Some(self.elapsed_time);
                }
            }
            Team::Attackers => self.attackers_killed += 1,
            Team::Undead => self.undead_killed += 1,
        }
    }

    pub fn record_spell_kill_defender(&mut self) {
        self.defenders_killed_by_spell += 1;
    }

    pub fn record_king_killed_by_spell(&mut self) {
        self.king_killed_by_spell = true;
    }

    pub fn reset(&mut self) {
        self.defenders_killed = 0;
        self.attackers_killed = 0;
        self.undead_killed = 0;
        self.defenders_killed_by_spell = 0;
        self.total_attackers_spawned = 0;
        self.elapsed_time = 0.0;
        self.first_defender_death_time = None;
        self.king_killed_by_spell = false;
    }
}

/// Tracks whether the player won or lost the game.
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Victory,        // Player wins (all attackers and undead eliminated)
    Defeat,         // Player loses (all defenders eliminated)
    DefeatKingDied, // Player loses (King was killed)
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

/// Tracks how many times the player has retried the current level.
#[derive(Resource, Default)]
pub struct RetryTracker {
    /// The level being tracked.
    pub level: u32,
    /// Number of consecutive attempts at this level.
    pub attempts: u32,
}
