use super::super::helpers::do_unlock;
use bevy::prelude::*;

use crate::config::save_data::grant_achievement_insight;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::GameOutcome;

use super::super::messages::BattleEndedMessage;
use super::super::resources::*;

pub(crate) fn check_first_victory(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<FirstVictoryAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 1 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(FirstVictoryAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_apprentice_wizard(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ApprenticeWizardAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 5 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(ApprenticeWizardAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_court_wizard(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<CourtWizardAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 10 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(CourtWizardAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_archmage(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ArchmageAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 25 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(ArchmageAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_legends_speak_your_name(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<LegendsSpeakYourNameAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 50 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(LegendsSpeakYourNameAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_immortalized(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ImmortalizedAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 100 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_the_grind_never_stops(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TheGrindNeverStopsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 200 {
            do_unlock(&mut res, &mut events);
        }
    }
}

// --- Level-based achievements (checked on both victory and defeat) ---

pub(crate) fn check_one_more_level(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<OneMoreLevelAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 10 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_into_the_deep(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<IntoTheDeepAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 25 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_absurdity(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<AbsurdityAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 50 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_level_100(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<Level100Achievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 100 {
            do_unlock(&mut res, &mut events);
        }
    }
}
