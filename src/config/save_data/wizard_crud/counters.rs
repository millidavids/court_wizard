use super::super::save_cache::{load_unified_save, new_unified_save, save_unified};
use super::super::save_structs::{EndlessLevelBest, UnifiedSaveFile};
use crate::config::resources::ActiveSave;
use crate::game::game_mode::components::ROGUELITE_MAX_LEVEL;
use crate::game::insight_bonuses::InsightBonusStat;

// ---------------------------------------------------------------------------
// Endless best stats
// ---------------------------------------------------------------------------

/// Returns the best stats for a specific endless level, if any have been recorded.
pub(crate) fn get_endless_best_stats(level: u32) -> Option<EndlessLevelBest> {
    let save_file = load_unified_save()?;
    let key = level.to_string();
    // Search all wizards for best stats at this level (return the best across wizards)
    save_file
        .wizards
        .iter()
        .filter_map(|w| w.endless_best_stats.get(&key).cloned())
        .max_by(|a, b| {
            a.best_efficiency
                .partial_cmp(&b.best_efficiency)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Update the best stats for an endless level if the current efficiency beats the stored best.
///
/// `coop` tags the entry when a co-op partner was connected; `coop_peer_name` is
/// their Steam display name (if known).
pub(crate) fn update_endless_best_stats(
    active_save: &ActiveSave,
    stats: &crate::game::game_mode::components::LevelRunStats,
    coop: bool,
    coop_peer_name: Option<String>,
) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        let key = stats.level.to_string();
        let should_update = wizard
            .endless_best_stats
            .get(&key)
            .is_none_or(|existing| stats.efficiency > existing.best_efficiency);

        if should_update {
            wizard.endless_best_stats.insert(
                key,
                EndlessLevelBest {
                    best_efficiency: stats.efficiency,
                    attackers_killed: stats.attackers_killed,
                    undead_killed: stats.undead_killed,
                    defenders_lost: stats.defenders_lost,
                    elapsed_time: stats.elapsed_time,
                    played_coop: coop,
                    coop_peer_name,
                },
            );
        }
    }

    save_unified(&save_file);
}

/// Records a co-op GUEST's endless result against their OWN wizard's save,
/// matched BY WIZARD TYPE (not the active save, which after a co-op match points
/// at a possibly-different wizard). Tags the entry as co-op and applies the
/// endless CONTIGUITY rule: the guest's frontier advances ONLY when this co-op
/// level is exactly their next level — joining a higher level and winning records
/// the result but does NOT skip them ahead. No-op (safe) if the guest has no save
/// for that wizard type, so it can never create or corrupt an unrelated save.
/// Records a co-op GUEST's level-end against their OWN save in a SINGLE load+save:
/// lifetime counters (games played, kill totals, and — on victory — levels
/// completed) plus, for an endless victory, the per-wizard endless best-stats with
/// the contiguity rule (frontier advances only when this is exactly their next
/// level). The per-wizard part is matched BY WIZARD TYPE and is a no-op if the
/// guest has no save for that wizard, so it can never corrupt an unrelated save.
/// `efficiency` is clamped to `[0, 1]`: a full wipe also kills the King's Guard,
/// which can push raw defender losses past the initial army size and make the raw
/// formula go negative.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_coop_guest_level_end(
    guest_wizard: crate::config::WizardType,
    is_endless: bool,
    victory: bool,
    level: u32,
    defenders_killed: u32,
    attackers_killed: u32,
    undead_killed: u32,
    efficiency: f32,
    elapsed_time: f32,
    coop_peer_name: Option<String>,
) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };

    // Lifetime counters (player-wide, every co-op mode, win or lose) — mirrors the
    // host's `send_battle_ended` accounting so shared activity counts for both.
    save_file.player.total_games_played += 1;
    save_file.player.total_defenders_killed += defenders_killed;
    save_file.player.total_attackers_killed += attackers_killed;
    save_file.player.total_undead_killed += undead_killed;
    // Play time is the one engagement counter already carried over the wire
    // (CoopLevelOver.elapsed_time). The others (friendly-fire / spells cast / bosses
    // defeated) need those fields added to the co-op level-over message before the
    // guest can credit them — tracked as a follow-up wire-protocol change.
    save_file.player.total_play_time_secs += elapsed_time as f64;
    if victory {
        save_file.player.total_levels_completed += 1;
    }

    // Per-wizard endless progression (endless victories only).
    if is_endless
        && victory
        && let Some(wizard) = save_file
            .wizards
            .iter_mut()
            .find(|w| w.wizard_type == guest_wizard)
    {
        let efficiency = efficiency.clamp(0.0, 1.0);
        let key = level.to_string();
        let should_update = wizard
            .endless_best_stats
            .get(&key)
            .is_none_or(|existing| efficiency > existing.best_efficiency);
        if should_update {
            wizard.endless_best_stats.insert(
                key,
                EndlessLevelBest {
                    best_efficiency: efficiency,
                    attackers_killed,
                    undead_killed,
                    defenders_lost: defenders_killed,
                    elapsed_time,
                    played_coop: true,
                    coop_peer_name,
                },
            );
        }
        // Contiguity: advance the guest's own frontier only if they cleared
        // their OWN next level.
        if level == wizard.current_level {
            wizard.current_level = level + 1;
            if wizard.highest_level_achieved < wizard.current_level {
                wizard.highest_level_achieved = wizard.current_level;
            }
        }
    }

    save_unified(&save_file);
}

// ---------------------------------------------------------------------------
// Meta-progression counters
// ---------------------------------------------------------------------------

/// Increment meta-progression counters on victory.
pub(crate) fn increment_levels_completed() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_levels_completed += 1;
    save_unified(&save_file);
}

/// Increment total games played counter.
pub(crate) fn increment_games_played() {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_games_played += 1;
    save_unified(&save_file);
}

/// Accumulate per-battle kill stats into lifetime totals.
pub(crate) fn accumulate_kill_stats(defenders: u32, attackers: u32, undead: u32) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_defenders_killed += defenders;
    save_file.player.total_attackers_killed += attackers;
    save_file.player.total_undead_killed += undead;
    save_unified(&save_file);
}

/// Returns the total number of levels completed (victories) across all time.
pub(crate) fn get_total_levels_completed() -> u32 {
    load_unified_save()
        .map(|s| s.player.total_levels_completed)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Engagement / Steam-stat counters
// ---------------------------------------------------------------------------

/// Accumulate per-battle engagement counters into lifetime totals. Flushed once at
/// run end (score-screen entry) — never per cast — since each call clones+rewrites
/// the whole save cache.
pub(crate) fn accumulate_engagement_stats(
    friendly_fire: u32,
    spells_cast: u32,
    bosses_defeated: u32,
    play_time_secs: f32,
) {
    let Some(mut save_file) = load_unified_save() else {
        return;
    };
    save_file.player.total_defenders_killed_by_spell = save_file
        .player
        .total_defenders_killed_by_spell
        .saturating_add(friendly_fire);
    save_file.player.total_spells_cast = save_file
        .player
        .total_spells_cast
        .saturating_add(spells_cast);
    save_file.player.total_bosses_defeated = save_file
        .player
        .total_bosses_defeated
        .saturating_add(bosses_defeated);
    save_file.player.total_play_time_secs += play_time_secs as f64;
    save_unified(&save_file);
}

/// Snapshot of every lifetime total surfaced as a Steam stat. Save-domain type
/// (Steam-agnostic); the steam module maps each field to its `STAT_*` API name.
#[derive(Debug, Clone, Default)]
pub(crate) struct LifetimeStatTotals {
    pub(crate) total_victories: u32,
    pub(crate) total_battles: u32,
    pub(crate) attackers_killed: u32,
    pub(crate) defenders_lost: u32,
    pub(crate) undead_killed: u32,
    pub(crate) highest_endless_level: u32,
    pub(crate) roguelite_clears: u32,
    pub(crate) wizards_unlocked: u32,
    pub(crate) friendly_fire_kills: u32,
    pub(crate) bosses_defeated: u32,
    pub(crate) spells_cast: u32,
    pub(crate) play_time_secs: f64,
    pub(crate) insight_bonuses_maxed: u32,
    pub(crate) spells_unlocked: u32,
}

/// Reads the save cache and computes the current value of every lifetime stat.
/// Returns all-zero defaults if no save is loaded.
pub(crate) fn lifetime_stat_totals() -> LifetimeStatTotals {
    load_unified_save()
        .as_ref()
        .map(compute_lifetime_stats)
        .unwrap_or_default()
}

fn compute_lifetime_stats(save: &UnifiedSaveFile) -> LifetimeStatTotals {
    let p = &save.player;
    LifetimeStatTotals {
        total_victories: p.total_levels_completed,
        total_battles: p.total_games_played,
        attackers_killed: p.total_attackers_killed,
        defenders_lost: p.total_defenders_killed,
        undead_killed: p.total_undead_killed,
        highest_endless_level: highest_level(save.wizards.iter().map(|w| w.highest_level_achieved)),
        // Persisted counter is authoritative and lossless going forward; max() with
        // the (FIFO-trimmed) run-history scan backfills clears recorded before the
        // counter existed without ever exceeding the true total.
        roguelite_clears: p.total_roguelite_clears.max(count_full_clears(
            save.wizards
                .iter()
                .flat_map(|w| w.roguelite.run_history.iter())
                .map(|r| (r.victory, r.levels_completed)),
        )),
        wizards_unlocked: p.unlocked_content.wizard_types.len() as u32,
        friendly_fire_kills: p.total_defenders_killed_by_spell,
        bosses_defeated: p.total_bosses_defeated,
        spells_cast: p.total_spells_cast,
        play_time_secs: p.total_play_time_secs,
        insight_bonuses_maxed: count_maxed_bonuses(
            InsightBonusStat::all()
                .iter()
                .map(|s| p.insight_bonuses.get(s.id()).copied().unwrap_or(0)),
            InsightBonusStat::max_level(),
        ),
        spells_unlocked: p.unlocked_content.spells.len() as u32,
    }
}

/// Highest endless level across all wizards (0 when none have played).
fn highest_level(levels: impl Iterator<Item = u32>) -> u32 {
    levels.max().unwrap_or(0)
}

/// Count of roguelite runs that fully cleared: a victory reaching the max level.
fn count_full_clears(runs: impl Iterator<Item = (bool, u32)>) -> u32 {
    runs.filter(|&(victory, levels)| victory && levels >= ROGUELITE_MAX_LEVEL)
        .count() as u32
}

/// Count of insight-bonus stats sitting at (or above) max level.
fn count_maxed_bonuses(levels: impl Iterator<Item = u8>, max_level: u8) -> u32 {
    levels.filter(|&level| level >= max_level).count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_level_is_max_or_zero() {
        assert_eq!(highest_level(std::iter::empty()), 0);
        assert_eq!(highest_level([3, 17, 9].into_iter()), 17);
    }

    #[test]
    fn full_clears_require_victory_at_max_level() {
        let runs = [
            (true, ROGUELITE_MAX_LEVEL),       // counts
            (true, ROGUELITE_MAX_LEVEL + 5),   // counts (>= max)
            (true, ROGUELITE_MAX_LEVEL - 1),   // too few levels
            (false, ROGUELITE_MAX_LEVEL),      // not a victory
            (false, ROGUELITE_MAX_LEVEL + 10), // not a victory
        ];
        assert_eq!(count_full_clears(runs.into_iter()), 2);
    }

    #[test]
    fn maxed_bonuses_counts_at_or_above_max() {
        assert_eq!(count_maxed_bonuses(std::iter::empty(), 5), 0);
        // max level 5: only 5 (and any overflow) count, 4 does not.
        assert_eq!(count_maxed_bonuses([5u8, 4, 5, 0].into_iter(), 5), 2);
        assert_eq!(count_maxed_bonuses([0u8, 0, 0, 0].into_iter(), 5), 0);
    }
}
