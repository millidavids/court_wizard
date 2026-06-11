use super::super::helpers::do_unlock;
use bevy::prelude::*;

use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::GameOutcome;
use crate::game::units::boss::dark_mage::components::DarkMage;
use crate::game::units::boss::hags::components::Hag;
use crate::game::units::boss::lich::components::Lich;
use crate::game::units::boss::ogre::components::OgreEnrageState;
use crate::game::units::boss::ray::Ray;

use super::super::messages::BattleEndedMessage;
use super::super::resources::*;

// ---------------------------------------------------------------------------
// Boss mark-seen (tracks presence for defeat achievements)
// ---------------------------------------------------------------------------

pub(crate) fn mark_bosses_seen(
    hags: Query<(), With<Hag>>,
    ogres: Query<(), With<OgreEnrageState>>,
    liches: Query<(), With<Lich>>,
    dark_mages: Query<(), With<DarkMage>>,
    rays: Query<(), With<Ray>>,
    mut seen: ResMut<BossesSeenThisBattle>,
) {
    if seen.hag && seen.ogre && seen.lich && seen.dark_mage && seen.ray {
        return;
    }
    if !seen.hag && !hags.is_empty() {
        seen.hag = true;
    }
    if !seen.ogre && !ogres.is_empty() {
        seen.ogre = true;
    }
    if !seen.lich && !liches.is_empty() {
        seen.lich = true;
    }
    if !seen.dark_mage && !dark_mages.is_empty() {
        seen.dark_mage = true;
    }
    if !seen.ray && !rays.is_empty() {
        seen.ray = true;
    }
}

pub(crate) fn reset_bosses_seen(mut seen: ResMut<BossesSeenThisBattle>) {
    *seen = BossesSeenThisBattle::default();
}

// ---------------------------------------------------------------------------
// Boss defeat achievements (victory on a level where boss was present)
// ---------------------------------------------------------------------------

macro_rules! boss_defeat_system {
    ($fn_name:ident, $field:ident, $res:ty) => {
        pub(crate) fn $fn_name(
            mut msg: MessageReader<BattleEndedMessage>,
            seen: Res<BossesSeenThisBattle>,
            mut res: ResMut<$res>,
            mut events: MessageWriter<AchievementUnlockedMessage>,
        ) {
            for m in msg.read() {
                if m.outcome == GameOutcome::Victory && seen.$field {
                    do_unlock(&mut res, &mut events);
                }
            }
        }
    };
}

boss_defeat_system!(check_hags_defeated, hag, HagsDefeatedAchievement);
boss_defeat_system!(check_ogre_defeated, ogre, OgreDefeatedAchievement);
boss_defeat_system!(check_lich_defeated, lich, LichDefeatedAchievement);
boss_defeat_system!(
    check_dark_mage_defeated,
    dark_mage,
    DarkMageDefeatedAchievement
);
boss_defeat_system!(check_ray_defeated, ray, RayDefeatedAchievement);
