use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Which game mode the player is currently playing.
/// Inserted when the player selects a mode on the GameModeSelect screen.
/// Removed when returning to the main menu.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameMode {
    Endless,
    Roguelite,
}

impl GameMode {
    pub(crate) fn is_roguelite(&self) -> bool {
        matches!(self, GameMode::Roguelite)
    }

    pub(crate) fn is_endless(&self) -> bool {
        matches!(self, GameMode::Endless)
    }
}

/// Helper to check if the optional game mode resource is roguelite.
pub(crate) fn is_roguelite_mode(game_mode: Option<&GameMode>) -> bool {
    game_mode.is_some_and(|m| m.is_roguelite())
}

/// Helper to check if the optional game mode resource is endless.
pub(crate) fn is_endless_mode(game_mode: Option<&GameMode>) -> bool {
    game_mode.is_some_and(|m| m.is_endless())
}

/// Tracks the current roguelite run's accumulated stats.
/// Inserted when a new roguelite run starts, removed when the run ends.
#[derive(Resource)]
pub(crate) struct RogueliteRunState {
    pub started_at: u64,
    pub level_stats: Vec<LevelRunStats>,
}

/// Stats for a single level within a roguelite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelRunStats {
    pub level: u32,
    pub efficiency: f32,
    pub attackers_killed: u32,
    pub undead_killed: u32,
    pub defenders_lost: u32,
    pub elapsed_time: f32,
}

impl LevelRunStats {
    /// Calculates total kills (attackers + undead) for this level.
    pub fn total_kills(&self) -> u32 {
        self.attackers_killed + self.undead_killed
    }
}

/// Aggregate stats for a roguelite run.
pub(crate) struct RunAggregateStats {
    pub total_kills: u32,
    pub avg_efficiency: f32,
    pub total_time: f32,
}

impl RunAggregateStats {
    /// Computes aggregate stats from a slice of level stats.
    pub fn from_level_stats(stats: &[LevelRunStats]) -> Self {
        let total_kills: u32 = stats.iter().map(|s| s.total_kills()).sum();
        let total_time: f32 = stats.iter().map(|s| s.elapsed_time).sum();
        let avg_efficiency = if stats.is_empty() {
            0.0
        } else {
            stats.iter().map(|s| s.efficiency).sum::<f32>() / stats.len() as f32
        };
        Self {
            total_kills,
            avg_efficiency,
            total_time,
        }
    }
}

/// Formats seconds into a human-readable time string.
pub(crate) fn format_time(seconds: f32) -> String {
    let total = seconds as u32;
    let mins = total / 60;
    let secs = total % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Player-chosen modifiers for a roguelite run.
/// Inserted from the modifier selection screen; removed on return to main menu.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RogueliteModifiers {
    /// Wave spawn frequency multiplier (0.2–3.0, default 1.0).
    /// Higher values make waves arrive faster.
    pub game_speed: f32,
    /// Attacker base effectiveness multiplier (0.2–3.0, default 1.0).
    /// Higher values make enemies hit harder and move faster.
    pub enemy_effectiveness: f32,
    /// Enemy count multiplier (0.2–3.0, default 1.0).
    /// Higher values spawn more enemies per wave.
    pub enemy_count: f32,
}

impl Default for RogueliteModifiers {
    fn default() -> Self {
        Self {
            game_speed: 1.0,
            enemy_effectiveness: 1.0,
            enemy_count: 1.0,
        }
    }
}

impl RogueliteModifiers {
    /// Returns true if all modifiers are at their default (100%) values.
    #[allow(dead_code)]
    pub fn is_default(&self) -> bool {
        (self.game_speed - 1.0).abs() < 0.01
            && (self.enemy_effectiveness - 1.0).abs() < 0.01
            && (self.enemy_count - 1.0).abs() < 0.01
    }
}

/// Last level of a roguelite run (tier 4 boss, level 25).
pub(crate) const ROGUELITE_MAX_LEVEL: u32 = 25;

/// Maximum stored roguelite runs per wizard.
pub(crate) const MAX_ROGUELITE_RUN_HISTORY: usize = 20;
