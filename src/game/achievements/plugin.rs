use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use crate::game::run_conditions::is_gameplay_active;
use crate::ui::main_menu::settings::components::SliderAdjusted;

use crate::game::messages::IngredientCollectedMessage;

use super::messages::{
    BattleEndedMessage, ClearProgressMessage, CloseCallMessage, DefenderKilledBySpellMessage,
    EnemyKilledMessage, EntangleHitDefenderMessage, GuardianCircleHitAttackerMessage,
    MarkedForDeathKillMessage, OutOfRangeMessage, QwerKeyPressedMessage, ScorchedEarthMessage,
    SpellCastMessage, StormbringerMessage, UnitSickenedMessage, WizardTypeUnlockedMessage,
};
use super::resources::*;
use super::systems;

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BattleEndedMessage>()
            .add_message::<DefenderKilledBySpellMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_message::<ClearProgressMessage>()
            .add_message::<SpellCastMessage>()
            .add_message::<QwerKeyPressedMessage>()
            .add_message::<OutOfRangeMessage>()
            .add_message::<ScorchedEarthMessage>()
            .add_message::<GuardianCircleHitAttackerMessage>()
            .add_message::<EntangleHitDefenderMessage>()
            .add_message::<UnitSickenedMessage>()
            .add_message::<MarkedForDeathKillMessage>()
            .add_message::<CloseCallMessage>()
            .add_message::<StormbringerMessage>()
            .add_message::<WizardTypeUnlockedMessage>()
            .init_resource::<MultiKillTracker>()
            .init_resource::<BossesSeenThisBattle>()
            // Initialize all achievement resources from save at startup
            .add_systems(Startup, init_achievements)
            // NOTE: send_battle_ended is registered in ScoreScreenPlugin's chain
            // for explicit ordering with save_efficiency/setup_game_over_screen.
            // Victory & Progression achievement systems
            .add_systems(
                Update,
                (
                    systems::check_first_victory
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<FirstVictoryAchievement>),
                    systems::check_apprentice_wizard
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<ApprenticeWizardAchievement>),
                    systems::check_court_wizard
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<CourtWizardAchievement>),
                    systems::check_archmage
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<ArchmageAchievement>),
                    systems::check_legends_speak_your_name
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<LegendsSpeakYourNameAchievement>),
                    systems::check_immortalized
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<ImmortalizedAchievement>),
                    systems::check_the_grind_never_stops
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<TheGrindNeverStopsAchievement>),
                    systems::check_one_more_level
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<OneMoreLevelAchievement>),
                    systems::check_into_the_deep
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<IntoTheDeepAchievement>),
                    systems::check_absurdity
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<AbsurdityAchievement>),
                    systems::check_level_100
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<Level100Achievement>),
                ),
            )
            // Defeat & Failure achievement systems
            .add_systems(
                Update,
                (
                    systems::check_tactical_retreat
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<TacticalRetreatAchievement>),
                    systems::check_the_king_is_dead
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<TheKingIsDeadAchievement>),
                    systems::check_total_wipe
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<TotalWipeAchievement>),
                    systems::check_speedrun_wrong_direction
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<SpeedrunWrongDirectionAchievement>),
                    systems::check_pyrrhic_defeat
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<PyrrhicDefeatAchievement>),
                    systems::check_it_was_going_so_well
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<ItWasGoingSoWellAchievement>),
                    systems::check_friendly_fire_department
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<FriendlyFireDepartmentAchievement>),
                    systems::check_accidental_regicide
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<AccidentalRegicideAchievement>),
                    systems::check_stubborn
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<StubbornAchievement>),
                    systems::check_extremely_stubborn
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<ExtremelyStubbornAchievement>),
                ),
            )
            // Mid-battle achievements
            .add_systems(
                Update,
                (
                    systems::check_friendly_fire
                        .run_if(on_message::<DefenderKilledBySpellMessage>)
                        .run_if(achievement_locked::<FriendlyFireAchievement>),
                    systems::track_multi_kills
                        .run_if(on_message::<EnemyKilledMessage>)
                        .run_if(achievement_locked::<ChainReactionAchievement>),
                ),
            )
            // Spell-unlock achievements (mid-battle detection + check)
            .add_systems(
                Update,
                systems::detect_out_of_range
                    .run_if(is_gameplay_active)
                    .run_if(achievement_locked::<OutOfRangeAchievement>),
            )
            .add_systems(
                Update,
                (
                    systems::check_out_of_range
                        .run_if(on_message::<OutOfRangeMessage>)
                        .run_if(achievement_locked::<OutOfRangeAchievement>),
                    systems::check_scorched_earth
                        .run_if(on_message::<ScorchedEarthMessage>)
                        .run_if(achievement_locked::<ScorchedEarthAchievement>),
                    systems::check_protective_instincts
                        .run_if(on_message::<GuardianCircleHitAttackerMessage>)
                        .run_if(achievement_locked::<ProtectiveInstinctsAchievement>),
                    systems::check_friendly_thorns
                        .run_if(on_message::<EntangleHitDefenderMessage>)
                        .run_if(achievement_locked::<FriendlyThornsAchievement>),
                ),
            )
            // Multi-kill tracker timer (runs during gameplay)
            .add_systems(
                Update,
                systems::update_multi_kill_tracker.run_if(is_gameplay_active),
            )
            // SliderFiddler — settings menu
            .add_systems(
                Update,
                systems::check_slider_fiddler
                    .run_if(on_message::<SliderAdjusted>)
                    .run_if(achievement_locked::<SliderFiddlerAchievement>),
            )
            // Spell cast and QWER key detection — runs during gameplay
            .add_systems(
                Update,
                (
                    systems::detect_spell_cast,
                    systems::detect_qwer_key.run_if(achievement_locked::<QwerAchievement>),
                )
                    .run_if(is_gameplay_active),
            )
            // QWER — unlocks RuneCaster on first Q/W/E/R press
            .add_systems(
                Update,
                systems::check_qwer
                    .run_if(on_message::<QwerKeyPressedMessage>)
                    .run_if(achievement_locked::<QwerAchievement>),
            )
            // Random Magic Surge — 1/100 chance on spell cast
            .add_systems(
                Update,
                systems::check_random_magic_surge
                    .run_if(on_message::<SpellCastMessage>)
                    .run_if(achievement_locked::<RandomMagicSurgeAchievement>),
            )
            // Unit encounter achievements (run during gameplay, once per type)
            .add_systems(
                Update,
                (
                    systems::check_brute_encounter
                        .run_if(achievement_locked::<MeetTheBruteAchievement>),
                    systems::check_elite_encounter
                        .run_if(achievement_locked::<EliteForcesAchievement>),
                    systems::check_commander_encounter
                        .run_if(achievement_locked::<CommanderOnTheFieldAchievement>),
                    systems::check_healer_encounter
                        .run_if(achievement_locked::<EnemyMedicAchievement>),
                    systems::check_dispeller_encounter
                        .run_if(achievement_locked::<MagicNullifierAchievement>),
                    systems::check_hag_encounter
                        .run_if(achievement_locked::<TheThreeHagsAchievement>),
                    systems::check_ogre_encounter
                        .run_if(achievement_locked::<OgreWarlordAchievement>),
                    systems::check_lich_encounter
                        .run_if(achievement_locked::<LichEncounterAchievement>),
                    systems::check_dark_mage_encounter
                        .run_if(achievement_locked::<DarkMageEncounterAchievement>),
                    systems::check_ray_encounter
                        .run_if(achievement_locked::<RayEncounterAchievement>),
                    systems::check_shielder_encounter,
                    systems::check_assassin_encounter,
                    systems::check_aerialist_encounter,
                    systems::mark_bosses_seen,
                )
                    .run_if(is_gameplay_active),
            )
            // Boss defeat achievements
            .add_systems(
                Update,
                (
                    systems::check_hags_defeated
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<HagsDefeatedAchievement>),
                    systems::check_ogre_defeated
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<OgreDefeatedAchievement>),
                    systems::check_lich_defeated
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<LichDefeatedAchievement>),
                    systems::check_dark_mage_defeated
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<DarkMageDefeatedAchievement>),
                    systems::check_ray_defeated
                        .run_if(on_message::<BattleEndedMessage>)
                        .run_if(achievement_locked::<RayDefeatedAchievement>),
                ),
            )
            // Reset boss-seen flags on level load
            .add_systems(
                OnEnter(crate::state::AppState::Loading),
                systems::reset_bosses_seen,
            )
            // Soiled Surprise — triggered by first sickened event (unlocks Excremage)
            .add_systems(
                Update,
                systems::check_soiled_surprise
                    .run_if(on_message::<UnitSickenedMessage>)
                    .run_if(achievement_locked::<SoiledSurpriseAchievement>),
            )
            // Master Brewer — triggered by collecting all 18 ingredients
            .add_systems(
                Update,
                systems::check_master_brewer
                    .run_if(on_message::<IngredientCollectedMessage>)
                    .run_if(achievement_locked::<MasterBrewerAchievement>),
            )
            // Right to Bear Arms — kill a marked-for-death enemy (unlocks Warglock)
            .add_systems(
                Update,
                systems::check_right_to_bear_arms
                    .run_if(on_message::<MarkedForDeathKillMessage>)
                    .run_if(achievement_locked::<RightToBearArmsAchievement>),
            )
            // Close Call — enemy killed near the wizard (unlocks Swordcerer)
            .add_systems(
                Update,
                systems::check_close_call
                    .run_if(on_message::<CloseCallMessage>)
                    .run_if(achievement_locked::<CloseCallAchievement>),
            )
            // Stormbringer — Lightning Rod placed within Squall AoE (unlocks Meteorologist)
            .add_systems(
                Update,
                systems::detect_stormbringer
                    .run_if(is_gameplay_active)
                    .run_if(achievement_locked::<StormbringerAchievement>),
            )
            .add_systems(
                Update,
                systems::check_stormbringer
                    .run_if(on_message::<StormbringerMessage>)
                    .run_if(achievement_locked::<StormbringerAchievement>),
            )
            // Pacifist — win without spells damaging enemies (unlocks Shepherd)
            .add_systems(
                Update,
                systems::check_pacifist
                    .run_if(on_message::<BattleEndedMessage>)
                    .run_if(achievement_locked::<PacifistAchievement>),
            )
            // Roguelite modifier achievements — checked on battle end
            .add_systems(
                Update,
                systems::check_roguelite_modifier_achievements
                    .run_if(on_message::<BattleEndedMessage>),
            )
            // Clicker — win roguelite run with all keybindings unbound (mouse only)
            .add_systems(
                Update,
                systems::check_clicker
                    .run_if(on_message::<BattleEndedMessage>)
                    .run_if(achievement_locked::<ClickerAchievement>),
            )
            // Grand Council — all wizard types unlocked
            .add_systems(
                Update,
                systems::check_grand_council
                    .run_if(on_message::<WizardTypeUnlockedMessage>)
                    .run_if(achievement_locked::<GrandCouncilAchievement>),
            )
            // Walking Library — all spells researched
            .add_systems(
                Update,
                systems::check_walking_library
                    .run_if(on_message::<crate::game::messages::SpellResearchedMessage>)
                    .run_if(achievement_locked::<WalkingLibraryAchievement>),
            )
            // Peak Wizard — all insight bonuses maxed
            .add_systems(
                Update,
                systems::check_peak_wizard
                    .run_if(on_message::<crate::game::messages::InsightBonusUpgradedMessage>)
                    .run_if(achievement_locked::<PeakWizardAchievement>),
            )
            // Reset all achievements when progress is cleared
            .add_systems(
                Update,
                reset_all_achievements.run_if(on_message::<ClearProgressMessage>),
            );
    }
}
