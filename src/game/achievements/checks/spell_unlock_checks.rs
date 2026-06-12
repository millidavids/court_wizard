use super::super::helpers::{do_unlock, unlock_and_notify_wizard_type};
use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::GameOutcome;

use super::super::messages::{BattleEndedMessage, WizardTypeUnlockedMessage};
use super::super::resources::*;

// ---------------------------------------------------------------------------
// Master Brewer — triggered by IngredientCollectedMessage (all 18 ingredients)
// ---------------------------------------------------------------------------

pub(crate) fn check_master_brewer(
    mut msg: MessageReader<crate::game::messages::IngredientCollectedMessage>,
    mut res: ResMut<MasterBrewerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    use crate::game::cauldron::brews::Ingredient;

    if msg.read().next().is_some() {
        let save = crate::config::save_data::load_unified_save();
        let unlocked = save
            .map(|s| s.player.unlocked_content.ingredients)
            .unwrap_or_default();

        let all_collected = Ingredient::all()
            .iter()
            .all(|i| unlocked.iter().any(|n| n.as_str() == i.save_key()));

        if all_collected {
            do_unlock(&mut res, &mut events);
            unlock_and_notify_wizard_type(WizardType::Alchemist, &mut wizard_unlocked);
        }
    }
}

// ---------------------------------------------------------------------------
// Soiled Surprise — triggered by UnitSickenedMessage (first sickened event)
// ---------------------------------------------------------------------------

pub(crate) fn check_soiled_surprise(
    mut msg: MessageReader<super::super::messages::UnitSickenedMessage>,
    mut res: ResMut<SoiledSurpriseAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::Excremage, &mut wizard_unlocked);
    }
}

pub(crate) fn check_right_to_bear_arms(
    mut msg: MessageReader<super::super::messages::MarkedForDeathKillMessage>,
    mut res: ResMut<RightToBearArmsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::Warglock, &mut wizard_unlocked);
    }
}

// ---------------------------------------------------------------------------
// Pacifist — win a level without any spell damaging enemies (unlocks Shepherd)
// ---------------------------------------------------------------------------

pub(crate) fn check_pacifist(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<PacifistAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && !m.wizard_damaged_enemies {
            do_unlock(&mut res, &mut events);
            unlock_and_notify_wizard_type(WizardType::Shepherd, &mut wizard_unlocked);
        }
    }
}
