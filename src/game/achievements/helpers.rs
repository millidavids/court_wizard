//! Achievement helper functions and battle-ended trigger.

use bevy::prelude::*;

use crate::config::save_data::{
    accumulate_kill_stats, get_total_levels_completed, grant_insight, increment_games_played,
    increment_levels_completed, unlock_achievement,
};
use crate::config::{GameConfig, WizardType};
use crate::game::messages::{AchievementUnlockedMessage, TalentTierUnlockedMessage};
use crate::game::resources::{
    BattleInsightData, CurrentLevel, GameOutcome, KillStats, RetryTracker,
};

use super::messages::{BattleEndedMessage, WizardTypeUnlockedMessage};
use super::resources::*;

/// Helper to unlock an achievement: persists to save, updates resource, sends popup.
pub(super) fn do_unlock<T: AchievementResource>(
    res: &mut ResMut<T>,
    achievement_events: &mut MessageWriter<AchievementUnlockedMessage>,
) {
    let id = T::achievement_id();
    res.unlock();
    unlock_achievement(id);
    achievement_events.write(AchievementUnlockedMessage { id });
}

pub(super) fn unlock_and_notify_wizard_type(
    wizard_type: WizardType,
    msg: &mut MessageWriter<WizardTypeUnlockedMessage>,
) {
    let newly_unlocked = crate::config::save_data::unlock_wizard_type(wizard_type);
    msg.write(WizardTypeUnlockedMessage {
        wizard_type,
        newly_unlocked,
    });
}

// ---------------------------------------------------------------------------
// send_battle_ended — runs OnEnter(InGameState::ScoreScreen)
// ---------------------------------------------------------------------------

/// Collects battle data, updates meta-progression counters, and writes `BattleEndedMessage`.
/// Replaces the counter-incrementing part of the old `check_victory_progression_achievements`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn send_battle_ended(
    game_outcome: Res<GameOutcome>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    kill_stats: Res<KillStats>,
    mut retry_tracker: ResMut<RetryTracker>,
    mut message: MessageWriter<BattleEndedMessage>,
    mut battle_insight: ResMut<BattleInsightData>,
    talent_progress: Option<
        Res<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    mut talent_tier_msg: MessageWriter<TalentTierUnlockedMessage>,
) {
    let is_victory = *game_outcome == GameOutcome::Victory;

    // Always increment games played and accumulate kill stats
    increment_games_played();
    accumulate_kill_stats(
        kill_stats.defenders_killed,
        kill_stats.attackers_killed,
        kill_stats.undead_killed,
    );

    if is_victory {
        increment_levels_completed();
        retry_tracker.level = 0;
        retry_tracker.attempts = 0;
    } else {
        // Track retries for this level
        if retry_tracker.level == current_level.0 {
            retry_tracker.attempts += 1;
        } else {
            retry_tracker.level = current_level.0;
            retry_tracker.attempts = 1;
        }
    }

    let total_wins = get_total_levels_completed();
    let effective_highest = if is_victory {
        config.highest_level_achieved.max(current_level.0 + 1)
    } else {
        config.highest_level_achieved
    };

    // Calculate and grant Arcane Insight
    let total_defenders = (crate::game::constants::INITIAL_DEFENDER_COUNT
        + crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT)
        as f32;
    let defenders_lost = kill_stats.defenders_killed as f32;
    let efficiency = 1.0 - (defenders_lost / total_defenders).min(1.0);

    let mut insight = 5 + (current_level.0 * 2); // Base: 5 + level * 2
    if is_victory {
        insight += 10; // Victory bonus
    }
    insight += (efficiency * 10.0) as u32; // Efficiency bonus: 0-10

    // Toggle modifier bonus: each active toggle adds a percentage bonus
    let toggle_bonus_pct = active_toggles
        .as_ref()
        .map(|t| t.total_insight_bonus_percent())
        .unwrap_or(0);
    if toggle_bonus_pct > 0 {
        insight += (insight * toggle_bonus_pct) / 100;
    }

    battle_insight.insight_earned = insight;
    grant_insight(insight);

    // Flush accumulated talent progress and notify per crossed tier threshold.
    if let Some(tp) = &talent_progress {
        for (spell, tier) in tp.flush_to_save() {
            talent_tier_msg.write(TalentTierUnlockedMessage { spell, tier });
        }
    }

    message.write(BattleEndedMessage {
        outcome: *game_outcome,
        total_wins,
        highest_level: effective_highest,
        elapsed_time: kill_stats.elapsed_time,
        attackers_killed: kill_stats.attackers_killed,
        total_attackers_spawned: kill_stats.total_attackers_spawned,
        defenders_killed_by_spell: kill_stats.defenders_killed_by_spell,
        king_killed_by_spell: kill_stats.king_killed_by_spell,
        first_defender_death_time: kill_stats.first_defender_death_time,
        retry_attempts: retry_tracker.attempts,
        wizard_damaged_enemies: kill_stats.wizard_damaged_enemies,
    });
}

// ---------------------------------------------------------------------------
// Victory & Progression achievements (triggered by BattleEndedMessage)
// ---------------------------------------------------------------------------
