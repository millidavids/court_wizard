use crate::config::save_data::AchievementId;

/// Dev/command-line fallback Steam App ID for Court Wizard (the main app).
///
/// This is ONLY used as a debug-build fallback when the game is launched outside
/// Steam with no `steam_appid.txt` in the working directory (see
/// `init_steam_plugin` in `plugin.rs`). The real app id for shipped builds is
/// provided by Steam at launch — `4550880` for the main app, `4820340` for the
/// Playtest — so a single binary serves both. Do NOT use this constant as the
/// source of truth for "which app are we"; read `client.utils().app_id()` instead.
///
/// Only compiled into debug builds — release builds never use the hardcoded id
/// (they always take the Steam-provided one), so it would be dead code there.
#[cfg(debug_assertions)]
pub(crate) const APP_ID: u32 = 4550880;

/// Maps an in-game AchievementId to its Steam API name string.
/// These names must match exactly what's configured in the Steamworks dashboard.
pub(super) fn steam_api_name(id: AchievementId) -> &'static str {
    match id {
        // Victory & Progression
        AchievementId::FirstVictory => "ACH_FIRST_VICTORY",
        AchievementId::ChainReaction => "ACH_CHAIN_REACTION",
        AchievementId::ApprenticeWizard => "ACH_APPRENTICE_WIZARD",
        AchievementId::CourtWizard => "ACH_COURT_WIZARD",
        AchievementId::Archmage => "ACH_ARCHMAGE",
        AchievementId::LegendsSpeakYourName => "ACH_LEGENDS_SPEAK_YOUR_NAME",
        AchievementId::Immortalized => "ACH_IMMORTALIZED",
        AchievementId::TheGrindNeverStops => "ACH_THE_GRIND_NEVER_STOPS",
        AchievementId::OneMoreLevel => "ACH_ONE_MORE_LEVEL",
        AchievementId::IntoTheDeep => "ACH_INTO_THE_DEEP",
        AchievementId::Absurdity => "ACH_ABSURDITY",
        AchievementId::Level100 => "ACH_LEVEL_100",
        AchievementId::Stubborn => "ACH_STUBBORN",
        AchievementId::ExtremelyStubborn => "ACH_EXTREMELY_STUBBORN",

        // Defeat & Failure
        AchievementId::TacticalRetreat => "ACH_TACTICAL_RETREAT",
        AchievementId::TheKingIsDead => "ACH_THE_KING_IS_DEAD",
        AchievementId::TotalWipe => "ACH_TOTAL_WIPE",
        AchievementId::SpeedrunWrongDirection => "ACH_SPEEDRUN_WRONG_DIRECTION",
        AchievementId::PyrrhicDefeat => "ACH_PYRRHIC_DEFEAT",
        AchievementId::ItWasGoingSoWell => "ACH_IT_WAS_GOING_SO_WELL",
        AchievementId::FriendlyFireDepartment => "ACH_FRIENDLY_FIRE_DEPARTMENT",
        AchievementId::AccidentalRegicide => "ACH_ACCIDENTAL_REGICIDE",

        // Mid-battle
        AchievementId::FriendlyFire => "ACH_FRIENDLY_FIRE",

        // Meta / Unlocks
        AchievementId::SliderFiddler => "ACH_SLIDER_FIDDLER",
        AchievementId::RandomMagicSurge => "ACH_RANDOM_MAGIC_SURGE",
        AchievementId::Qwer => "ACH_QWER",

        // Unit Encounters
        AchievementId::MeetTheBrute => "ACH_MEET_THE_BRUTE",
        AchievementId::EliteForces => "ACH_ELITE_FORCES",
        AchievementId::CommanderOnTheField => "ACH_COMMANDER_ON_THE_FIELD",
        AchievementId::EnemyMedic => "ACH_ENEMY_MEDIC",
        AchievementId::MagicNullifier => "ACH_MAGIC_NULLIFIER",
        AchievementId::TheThreeHags => "ACH_THE_THREE_HAGS",
        AchievementId::OgreWarlord => "ACH_OGRE_WARLORD",

        // Spell Unlocks
        AchievementId::OutOfRange => "ACH_OUT_OF_RANGE",
        AchievementId::ScorchedEarth => "ACH_SCORCHED_EARTH",
        AchievementId::ProtectiveInstincts => "ACH_PROTECTIVE_INSTINCTS",
        AchievementId::FriendlyThorns => "ACH_FRIENDLY_THORNS",
        AchievementId::SoiledSurprise => "ACH_SOILED_SURPRISE",
        AchievementId::MasterBrewer => "ACH_MASTER_BREWER",
        AchievementId::RightToBearArms => "ACH_RIGHT_TO_BEAR_ARMS",
        AchievementId::CloseCall => "ACH_CLOSE_CALL",
        AchievementId::Stormbringer => "ACH_STORMBRINGER",
        AchievementId::Pacifist => "ACH_PACIFIST",

        // Roguelite Modifier Achievements
        AchievementId::ModWaveSpeedMin => "ACH_MOD_WAVE_SPEED_MIN",
        AchievementId::ModWaveSpeed100 => "ACH_MOD_WAVE_SPEED_100",
        AchievementId::ModWaveSpeed200 => "ACH_MOD_WAVE_SPEED_200",
        AchievementId::ModWaveSpeed300 => "ACH_MOD_WAVE_SPEED_300",
        AchievementId::ModEnemyStrengthMin => "ACH_MOD_ENEMY_STRENGTH_MIN",
        AchievementId::ModEnemyStrength100 => "ACH_MOD_ENEMY_STRENGTH_100",
        AchievementId::ModEnemyStrength200 => "ACH_MOD_ENEMY_STRENGTH_200",
        AchievementId::ModEnemyStrength300 => "ACH_MOD_ENEMY_STRENGTH_300",
        AchievementId::ModEnemyCountMin => "ACH_MOD_ENEMY_COUNT_MIN",
        AchievementId::ModEnemyCount100 => "ACH_MOD_ENEMY_COUNT_100",
        AchievementId::ModEnemyCount200 => "ACH_MOD_ENEMY_COUNT_200",
        AchievementId::ModEnemyCount300 => "ACH_MOD_ENEMY_COUNT_300",
        AchievementId::ModAllMin => "ACH_MOD_ALL_MIN",
        AchievementId::ModAll200 => "ACH_MOD_ALL_200",
        AchievementId::ModAllMax => "ACH_MOD_ALL_MAX",
        AchievementId::ModMixedExtremes => "ACH_MOD_MIXED_EXTREMES",
        AchievementId::Clicker => "ACH_CLICKER",
        AchievementId::GrandCouncil => "ACH_GRAND_COUNCIL",
        AchievementId::WalkingLibrary => "ACH_WALKING_LIBRARY",
        AchievementId::PeakWizard => "ACH_PEAK_WIZARD",
        AchievementId::LichEncounter => "ACH_LICH_ENCOUNTER",
        AchievementId::DarkMageEncounter => "ACH_DARK_MAGE_ENCOUNTER",
        AchievementId::RayEncounter => "ACH_RAY_ENCOUNTER",
        AchievementId::HagsDefeated => "ACH_HAGS_DEFEATED",
        AchievementId::OgreDefeated => "ACH_OGRE_DEFEATED",
        AchievementId::LichDefeated => "ACH_LICH_DEFEATED",
        AchievementId::DarkMageDefeated => "ACH_DARK_MAGE_DEFEATED",
        AchievementId::RayDefeated => "ACH_RAY_DEFEATED",
    }
}
