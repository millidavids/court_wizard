use serde::{Deserialize, Serialize};

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
#[derive(bevy::prelude::Resource, Clone, Debug, Serialize, Deserialize)]
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
    /// Terrain density multiplier (0.0–3.0, default 1.0).
    /// Controls how many trees, ponds, and bushes spawn.
    /// 0.0 = no terrain, 1.0 = normal, 3.0 = triple density.
    pub terrain_density: f32,
}

impl Default for RogueliteModifiers {
    fn default() -> Self {
        Self {
            game_speed: 1.0,
            enemy_effectiveness: 1.0,
            enemy_count: 1.0,
            terrain_density: 1.0,
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
            && (self.terrain_density - 1.0).abs() < 0.01
    }

    /// Returns `(label, percentage)` pairs for sliders that deviate from 100%.
    pub fn non_default_entries(&self) -> Vec<(&'static str, u32)> {
        let mut entries = Vec::new();
        let sliders: [(&str, f32); 4] = [
            ("Wave Speed", self.game_speed),
            ("Enemy Strength", self.enemy_effectiveness),
            ("Enemy Count", self.enemy_count),
            ("Terrain", self.terrain_density),
        ];
        for (label, value) in sliders {
            if (value - 1.0).abs() > 0.01 {
                entries.push((label, (value * 100.0) as u32));
            }
        }
        entries
    }
}

/// Last level of a roguelite run (tier 4 boss, level 25).
pub(crate) const ROGUELITE_MAX_LEVEL: u32 = 25;

/// Maximum stored roguelite runs per wizard.
pub(crate) const MAX_ROGUELITE_RUN_HISTORY: usize = 20;
