use bevy::prelude::*;

use crate::config::save_data::load_unified_save;

use super::progress::{
    AbsurdityAchievement, AccidentalRegicideAchievement, AchievementResource,
    ApprenticeWizardAchievement, ArchmageAchievement, ChainReactionAchievement, ClickerAchievement,
    CloseCallAchievement, CommanderOnTheFieldAchievement, CourtWizardAchievement,
    DarkMageDefeatedAchievement, DarkMageEncounterAchievement, EliteForcesAchievement,
    EnemyMedicAchievement, ExtremelyStubbornAchievement, FirstVictoryAchievement,
    FriendlyFireAchievement, FriendlyFireDepartmentAchievement, FriendlyThornsAchievement,
    GrandCouncilAchievement, HagsDefeatedAchievement, ImmortalizedAchievement,
    IntoTheDeepAchievement, ItWasGoingSoWellAchievement, LegendsSpeakYourNameAchievement,
    Level100Achievement, LichDefeatedAchievement, LichEncounterAchievement,
    MagicNullifierAchievement, MasterBrewerAchievement, MeetTheBruteAchievement, ModAll200Ach,
    ModAllMaxAch, ModAllMinAch, ModEnemyCount100Ach, ModEnemyCount200Ach, ModEnemyCount300Ach,
    ModEnemyCountMinAch, ModEnemyStrength100Ach, ModEnemyStrength200Ach, ModEnemyStrength300Ach,
    ModEnemyStrengthMinAch, ModMixedExtremesAch, ModWaveSpeed100Ach, ModWaveSpeed200Ach,
    ModWaveSpeed300Ach, ModWaveSpeedMinAch, OgreDefeatedAchievement, OgreWarlordAchievement,
    OneMoreLevelAchievement, OutOfRangeAchievement, PacifistAchievement, PeakWizardAchievement,
    ProtectiveInstinctsAchievement, PyrrhicDefeatAchievement, QwerAchievement,
    RandomMagicSurgeAchievement, RayDefeatedAchievement, RayEncounterAchievement,
    RightToBearArmsAchievement, ScorchedEarthAchievement, SliderFiddlerAchievement,
    SoiledSurpriseAchievement, SpeedrunWrongDirectionAchievement, StormbringerAchievement,
    StubbornAchievement, TacticalRetreatAchievement, TheGrindNeverStopsAchievement,
    TheKingIsDeadAchievement, TheThreeHagsAchievement, TotalWipeAchievement,
    WalkingLibraryAchievement,
};

/// Resource to track consecutive enemy kills for multi-kill achievements.
#[derive(Resource, Default)]
pub(crate) struct MultiKillTracker {
    /// Number of consecutive kills within the time window.
    pub kills: u32,
    /// Time since last kill (resets when enemy dies).
    pub time_since_last_kill: f32,
}

impl MultiKillTracker {
    /// Time window for consecutive kills (in seconds).
    const WINDOW: f32 = 2.0;

    /// Reset the kill counter.
    pub fn reset(&mut self) {
        self.kills = 0;
        self.time_since_last_kill = 0.0;
    }

    /// Register a kill and return the current kill count.
    pub fn register_kill(&mut self) -> u32 {
        if self.time_since_last_kill > Self::WINDOW {
            self.kills = 1;
        } else {
            self.kills += 1;
        }
        self.time_since_last_kill = 0.0;
        self.kills
    }

    /// Update the timer.
    pub fn update(&mut self, delta: f32) {
        self.time_since_last_kill += delta;
        if self.time_since_last_kill > Self::WINDOW {
            self.reset();
        }
    }
}

/// Resets all achievement resources back to locked state by re-inserting them.
/// Triggered by `ClearProgressMessage` when the player clears their progress.
pub(crate) fn reset_all_achievements(
    mut _msg: MessageReader<super::super::messages::ClearProgressMessage>,
    mut commands: Commands,
) {
    commands.insert_resource(FirstVictoryAchievement(false));
    commands.insert_resource(ChainReactionAchievement(false));
    commands.insert_resource(ApprenticeWizardAchievement(false));
    commands.insert_resource(CourtWizardAchievement(false));
    commands.insert_resource(ArchmageAchievement(false));
    commands.insert_resource(LegendsSpeakYourNameAchievement(false));
    commands.insert_resource(ImmortalizedAchievement(false));
    commands.insert_resource(TheGrindNeverStopsAchievement(false));
    commands.insert_resource(OneMoreLevelAchievement(false));
    commands.insert_resource(IntoTheDeepAchievement(false));
    commands.insert_resource(AbsurdityAchievement(false));
    commands.insert_resource(Level100Achievement(false));
    commands.insert_resource(TacticalRetreatAchievement(false));
    commands.insert_resource(TheKingIsDeadAchievement(false));
    commands.insert_resource(TotalWipeAchievement(false));
    commands.insert_resource(SpeedrunWrongDirectionAchievement(false));
    commands.insert_resource(PyrrhicDefeatAchievement(false));
    commands.insert_resource(ItWasGoingSoWellAchievement(false));
    commands.insert_resource(FriendlyFireDepartmentAchievement(false));
    commands.insert_resource(AccidentalRegicideAchievement(false));
    commands.insert_resource(StubbornAchievement(false));
    commands.insert_resource(ExtremelyStubbornAchievement(false));
    commands.insert_resource(FriendlyFireAchievement(false));
    commands.insert_resource(SliderFiddlerAchievement(false));
    commands.insert_resource(RandomMagicSurgeAchievement(false));
    commands.insert_resource(QwerAchievement(false));
    commands.insert_resource(MeetTheBruteAchievement(false));
    commands.insert_resource(EliteForcesAchievement(false));
    commands.insert_resource(CommanderOnTheFieldAchievement(false));
    commands.insert_resource(EnemyMedicAchievement(false));
    commands.insert_resource(MagicNullifierAchievement(false));
    commands.insert_resource(TheThreeHagsAchievement(false));
    commands.insert_resource(OgreWarlordAchievement(false));
    commands.insert_resource(OutOfRangeAchievement(false));
    commands.insert_resource(ScorchedEarthAchievement(false));
    commands.insert_resource(ProtectiveInstinctsAchievement(false));
    commands.insert_resource(FriendlyThornsAchievement(false));
    commands.insert_resource(SoiledSurpriseAchievement(false));
    commands.insert_resource(MasterBrewerAchievement(false));
    commands.insert_resource(RightToBearArmsAchievement(false));
    commands.insert_resource(CloseCallAchievement(false));
    commands.insert_resource(StormbringerAchievement(false));
    commands.insert_resource(PacifistAchievement(false));
    commands.insert_resource(ModWaveSpeedMinAch(false));
    commands.insert_resource(ModWaveSpeed100Ach(false));
    commands.insert_resource(ModWaveSpeed200Ach(false));
    commands.insert_resource(ModWaveSpeed300Ach(false));
    commands.insert_resource(ModEnemyStrengthMinAch(false));
    commands.insert_resource(ModEnemyStrength100Ach(false));
    commands.insert_resource(ModEnemyStrength200Ach(false));
    commands.insert_resource(ModEnemyStrength300Ach(false));
    commands.insert_resource(ModEnemyCountMinAch(false));
    commands.insert_resource(ModEnemyCount100Ach(false));
    commands.insert_resource(ModEnemyCount200Ach(false));
    commands.insert_resource(ModEnemyCount300Ach(false));
    commands.insert_resource(ModAllMinAch(false));
    commands.insert_resource(ModAll200Ach(false));
    commands.insert_resource(ModAllMaxAch(false));
    commands.insert_resource(ModMixedExtremesAch(false));
    commands.insert_resource(ClickerAchievement(false));
    commands.insert_resource(GrandCouncilAchievement(false));
    commands.insert_resource(WalkingLibraryAchievement(false));
    commands.insert_resource(PeakWizardAchievement(false));
    commands.insert_resource(LichEncounterAchievement(false));
    commands.insert_resource(DarkMageEncounterAchievement(false));
    commands.insert_resource(HagsDefeatedAchievement(false));
    commands.insert_resource(OgreDefeatedAchievement(false));
    commands.insert_resource(LichDefeatedAchievement(false));
    commands.insert_resource(DarkMageDefeatedAchievement(false));
    commands.insert_resource(RayEncounterAchievement(false));
    commands.insert_resource(RayDefeatedAchievement(false));
}

/// Initializes all achievement resources from the save file at startup.
pub(crate) fn init_achievements(mut commands: Commands) {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_achievements)
        .unwrap_or_default();

    /// Helper: insert a resource, marking it unlocked if present in save.
    macro_rules! init {
        ($res:ident) => {
            let is_unlocked = unlocked.contains(&$res::achievement_id().id().to_string());
            commands.insert_resource($res(is_unlocked));
        };
    }

    init!(FirstVictoryAchievement);
    init!(ChainReactionAchievement);
    init!(ApprenticeWizardAchievement);
    init!(CourtWizardAchievement);
    init!(ArchmageAchievement);
    init!(LegendsSpeakYourNameAchievement);
    init!(ImmortalizedAchievement);
    init!(TheGrindNeverStopsAchievement);
    init!(OneMoreLevelAchievement);
    init!(IntoTheDeepAchievement);
    init!(AbsurdityAchievement);
    init!(Level100Achievement);
    init!(TacticalRetreatAchievement);
    init!(TheKingIsDeadAchievement);
    init!(TotalWipeAchievement);
    init!(SpeedrunWrongDirectionAchievement);
    init!(PyrrhicDefeatAchievement);
    init!(ItWasGoingSoWellAchievement);
    init!(FriendlyFireDepartmentAchievement);
    init!(AccidentalRegicideAchievement);
    init!(StubbornAchievement);
    init!(ExtremelyStubbornAchievement);
    init!(FriendlyFireAchievement);
    init!(SliderFiddlerAchievement);
    init!(RandomMagicSurgeAchievement);
    init!(QwerAchievement);
    init!(MeetTheBruteAchievement);
    init!(EliteForcesAchievement);
    init!(CommanderOnTheFieldAchievement);
    init!(EnemyMedicAchievement);
    init!(MagicNullifierAchievement);
    init!(TheThreeHagsAchievement);
    init!(OgreWarlordAchievement);
    init!(OutOfRangeAchievement);
    init!(ScorchedEarthAchievement);
    init!(ProtectiveInstinctsAchievement);
    init!(FriendlyThornsAchievement);
    init!(SoiledSurpriseAchievement);
    init!(MasterBrewerAchievement);
    init!(RightToBearArmsAchievement);
    init!(CloseCallAchievement);
    init!(StormbringerAchievement);
    init!(PacifistAchievement);
    init!(ModWaveSpeedMinAch);
    init!(ModWaveSpeed100Ach);
    init!(ModWaveSpeed200Ach);
    init!(ModWaveSpeed300Ach);
    init!(ModEnemyStrengthMinAch);
    init!(ModEnemyStrength100Ach);
    init!(ModEnemyStrength200Ach);
    init!(ModEnemyStrength300Ach);
    init!(ModEnemyCountMinAch);
    init!(ModEnemyCount100Ach);
    init!(ModEnemyCount200Ach);
    init!(ModEnemyCount300Ach);
    init!(ModAllMinAch);
    init!(ModAll200Ach);
    init!(ModAllMaxAch);
    init!(ModMixedExtremesAch);
    init!(ClickerAchievement);
    init!(GrandCouncilAchievement);
    init!(WalkingLibraryAchievement);
    init!(PeakWizardAchievement);
    init!(LichEncounterAchievement);
    init!(DarkMageEncounterAchievement);
    init!(HagsDefeatedAchievement);
    init!(OgreDefeatedAchievement);
    init!(LichDefeatedAchievement);
    init!(DarkMageDefeatedAchievement);
    init!(RayEncounterAchievement);
    init!(RayDefeatedAchievement);
}
