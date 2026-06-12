use super::super::save_cache::{load_unified_save, new_unified_save, save_unified};
use super::super::save_structs::EndlessLevelBest;
use crate::config::resources::ActiveSave;

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
