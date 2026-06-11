use super::super::helpers::{do_unlock, unlock_and_notify_wizard_type};
use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::GameOutcome;

use super::super::messages::{BattleEndedMessage, WizardTypeUnlockedMessage};
use super::super::resources::*;

// ---------------------------------------------------------------------------
// Defeat & Failure achievements (triggered by BattleEndedMessage)
// ---------------------------------------------------------------------------

pub(crate) fn check_tactical_retreat(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TacticalRetreatAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_the_king_is_dead(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TheKingIsDeadAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    use crate::config::save_data::grant_achievement_insight;
    for m in msg.read() {
        if m.outcome == GameOutcome::DefeatKingDied {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(TheKingIsDeadAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_total_wipe(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TotalWipeAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Defeat {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_speedrun_wrong_direction(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<SpeedrunWrongDirectionAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.elapsed_time < 30.0 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_pyrrhic_defeat(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<PyrrhicDefeatAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.total_attackers_spawned > 0 {
            let kill_ratio = m.attackers_killed as f32 / m.total_attackers_spawned as f32;
            if kill_ratio >= 0.9 {
                do_unlock(&mut res, &mut events);
            }
        }
    }
}

pub(crate) fn check_it_was_going_so_well(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ItWasGoingSoWellAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory
            && let Some(first_death_time) = m.first_defender_death_time
            && first_death_time >= 120.0
        {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_friendly_fire_department(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<FriendlyFireDepartmentAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.defenders_killed_by_spell >= 10 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_accidental_regicide(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<AccidentalRegicideAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.king_killed_by_spell {
            do_unlock(&mut res, &mut events);
            unlock_and_notify_wizard_type(WizardType::Psychopath, &mut wizard_unlocked);
        }
    }
}

pub(crate) fn check_stubborn(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<StubbornAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.retry_attempts >= 5 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_extremely_stubborn(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ExtremelyStubbornAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.retry_attempts >= 15 {
            do_unlock(&mut res, &mut events);
        }
    }
}
