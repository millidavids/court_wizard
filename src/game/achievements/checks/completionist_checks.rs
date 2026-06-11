use super::super::helpers::do_unlock;
use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::grant_achievement_insight;
use crate::game::messages::AchievementUnlockedMessage;

use super::super::messages::WizardTypeUnlockedMessage;
use super::super::resources::*;

// ---------------------------------------------------------------------------
// Grand Council — all wizard types unlocked
// ---------------------------------------------------------------------------

pub(crate) fn check_grand_council(
    mut msg: MessageReader<WizardTypeUnlockedMessage>,
    mut res: ResMut<GrandCouncilAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_none() {
        return;
    }
    let unlocked_count = crate::config::save_data::load_unified_save()
        .map(|s| s.player.unlocked_content.wizard_types.len())
        .unwrap_or(0);
    if unlocked_count >= WizardType::all().len() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(GrandCouncilAchievement::achievement_id());
    }
}

// ---------------------------------------------------------------------------
// Walking Library — all spells unlocked
// ---------------------------------------------------------------------------

pub(crate) fn check_walking_library(
    mut msg: MessageReader<crate::game::messages::SpellResearchedMessage>,
    mut res: ResMut<WalkingLibraryAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_none() {
        return;
    }
    let unlocked_count = crate::config::save_data::load_unified_save()
        .map(|s| s.player.unlocked_content.spells.len())
        .unwrap_or(0);
    use crate::game::units::wizard::components::Spell;
    if unlocked_count >= Spell::all().len() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(WalkingLibraryAchievement::achievement_id());
    }
}

// ---------------------------------------------------------------------------
// Peak Wizard — all insight bonuses at max level
// ---------------------------------------------------------------------------

pub(crate) fn check_peak_wizard(
    mut msg: MessageReader<crate::game::messages::InsightBonusUpgradedMessage>,
    mut res: ResMut<PeakWizardAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_none() {
        return;
    }
    use crate::game::insight_bonuses::InsightBonusStat;
    let all_maxed = InsightBonusStat::all()
        .iter()
        .all(|stat| stat.current_level() >= InsightBonusStat::max_level());
    if all_maxed {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(PeakWizardAchievement::achievement_id());
    }
}
