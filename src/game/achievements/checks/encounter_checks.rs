use super::super::helpers::do_unlock;
use bevy::prelude::*;

use crate::config::save_data::unlock_unit;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::units::UnitType;
use crate::game::units::aerialist::components::Aerialist;
use crate::game::units::assassin::components::Assassin;
use crate::game::units::boss::dark_mage::components::DarkMage;
use crate::game::units::boss::hags::components::Hag;
use crate::game::units::boss::lich::components::Lich;
use crate::game::units::boss::ogre::components::OgreEnrageState;
use crate::game::units::boss::ray::Ray;
use crate::game::units::brute::components::Brute;
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::healer::components::Healer;
use crate::game::units::shielder::components::Shielder;
use crate::game::units::{Commander, EliteHealthBonus};

use super::super::resources::*;

// ---------------------------------------------------------------------------
// Unit encounter achievements
// ---------------------------------------------------------------------------

/// Macro to generate an encounter-detection system for a unit type.
macro_rules! encounter_system {
    ($fn_name:ident, $marker:ty, $res:ty, $unit_type:expr) => {
        pub(crate) fn $fn_name(
            query: Query<(), With<$marker>>,
            mut res: ResMut<$res>,
            mut events: MessageWriter<AchievementUnlockedMessage>,
        ) {
            if !query.is_empty() {
                unlock_unit($unit_type);
                do_unlock(&mut res, &mut events);
            }
        }
    };
}

encounter_system!(
    check_brute_encounter,
    Brute,
    MeetTheBruteAchievement,
    UnitType::Brute
);
encounter_system!(
    check_elite_encounter,
    EliteHealthBonus,
    EliteForcesAchievement,
    UnitType::Elite
);
encounter_system!(
    check_commander_encounter,
    Commander,
    CommanderOnTheFieldAchievement,
    UnitType::Commander
);
encounter_system!(
    check_healer_encounter,
    Healer,
    EnemyMedicAchievement,
    UnitType::Healer
);
encounter_system!(
    check_dispeller_encounter,
    Dispeller,
    MagicNullifierAchievement,
    UnitType::Dispeller
);
encounter_system!(
    check_hag_encounter,
    Hag,
    TheThreeHagsAchievement,
    UnitType::Hag
);
encounter_system!(
    check_ogre_encounter,
    OgreEnrageState,
    OgreWarlordAchievement,
    UnitType::Ogre
);
encounter_system!(
    check_lich_encounter,
    Lich,
    LichEncounterAchievement,
    UnitType::Lich
);

/// Macro for unit encounters that just record the unlock — no achievement.
/// The `Local<bool>` latches after the first detection to avoid hitting the
/// save file on subsequent frames (`unlock_unit` reads the save unconditionally).
macro_rules! encounter_unlock_only {
    ($fn_name:ident, $marker:ty, $unit_type:expr) => {
        pub(crate) fn $fn_name(query: Query<(), With<$marker>>, mut done: Local<bool>) {
            if *done {
                return;
            }
            if !query.is_empty() {
                unlock_unit($unit_type);
                *done = true;
            }
        }
    };
}

encounter_unlock_only!(check_shielder_encounter, Shielder, UnitType::Shielder);
encounter_unlock_only!(check_assassin_encounter, Assassin, UnitType::Assassin);
encounter_unlock_only!(check_aerialist_encounter, Aerialist, UnitType::Aerialist);

encounter_system!(
    check_dark_mage_encounter,
    DarkMage,
    DarkMageEncounterAchievement,
    UnitType::DarkMage
);
encounter_system!(
    check_ray_encounter,
    Ray,
    RayEncounterAchievement,
    UnitType::Ray
);
