use bevy::prelude::*;

use crate::config::save_data::{AchievementId, load_unified_save};

/// Trait implemented by all per-achievement resources.
pub(crate) trait AchievementResource: Resource {
    fn is_locked(&self) -> bool;
    fn unlock(&mut self);
    fn achievement_id() -> AchievementId;
}

/// Generates a per-achievement resource type with locked/unlocked state.
macro_rules! achievement_resource {
    ($name:ident, $id:expr) => {
        #[derive(Resource)]
        pub(crate) struct $name(pub bool);

        impl AchievementResource for $name {
            fn is_locked(&self) -> bool {
                !self.0
            }

            fn unlock(&mut self) {
                self.0 = true;
            }

            fn achievement_id() -> AchievementId {
                $id
            }
        }
    };
}

// Victory & Progression
achievement_resource!(FirstVictoryAchievement, AchievementId::FirstVictory);
achievement_resource!(ChainReactionAchievement, AchievementId::ChainReaction);
achievement_resource!(ApprenticeWizardAchievement, AchievementId::ApprenticeWizard);
achievement_resource!(CourtWizardAchievement, AchievementId::CourtWizard);
achievement_resource!(ArchmageAchievement, AchievementId::Archmage);
achievement_resource!(
    LegendsSpeakYourNameAchievement,
    AchievementId::LegendsSpeakYourName
);
achievement_resource!(ImmortalizedAchievement, AchievementId::Immortalized);
achievement_resource!(
    TheGrindNeverStopsAchievement,
    AchievementId::TheGrindNeverStops
);
achievement_resource!(OneMoreLevelAchievement, AchievementId::OneMoreLevel);
achievement_resource!(IntoTheDeepAchievement, AchievementId::IntoTheDeep);
achievement_resource!(AbsurdityAchievement, AchievementId::Absurdity);
achievement_resource!(Level100Achievement, AchievementId::Level100);

// Defeat & Failure
achievement_resource!(TacticalRetreatAchievement, AchievementId::TacticalRetreat);
achievement_resource!(TheKingIsDeadAchievement, AchievementId::TheKingIsDead);
achievement_resource!(TotalWipeAchievement, AchievementId::TotalWipe);
achievement_resource!(
    SpeedrunWrongDirectionAchievement,
    AchievementId::SpeedrunWrongDirection
);
achievement_resource!(PyrrhicDefeatAchievement, AchievementId::PyrrhicDefeat);
achievement_resource!(ItWasGoingSoWellAchievement, AchievementId::ItWasGoingSoWell);
achievement_resource!(
    FriendlyFireDepartmentAchievement,
    AchievementId::FriendlyFireDepartment
);
achievement_resource!(
    AccidentalRegicideAchievement,
    AchievementId::AccidentalRegicide
);
achievement_resource!(StubbornAchievement, AchievementId::Stubborn);
achievement_resource!(
    ExtremelyStubbornAchievement,
    AchievementId::ExtremelyStubborn
);

// Mid-battle
achievement_resource!(FriendlyFireAchievement, AchievementId::FriendlyFire);

// Meta / Unlocks
achievement_resource!(SliderFiddlerAchievement, AchievementId::SliderFiddler);
achievement_resource!(RandomMagicSurgeAchievement, AchievementId::RandomMagicSurge);
achievement_resource!(QwerAchievement, AchievementId::Qwer);

// Unit Encounters
achievement_resource!(MeetTheBruteAchievement, AchievementId::MeetTheBrute);
achievement_resource!(EliteForcesAchievement, AchievementId::EliteForces);
achievement_resource!(
    CommanderOnTheFieldAchievement,
    AchievementId::CommanderOnTheField
);
achievement_resource!(EnemyMedicAchievement, AchievementId::EnemyMedic);
achievement_resource!(MagicNullifierAchievement, AchievementId::MagicNullifier);
achievement_resource!(TheThreeHagsAchievement, AchievementId::TheThreeHags);
achievement_resource!(OgreWarlordAchievement, AchievementId::OgreWarlord);

// Spell Unlocks
achievement_resource!(OutOfRangeAchievement, AchievementId::OutOfRange);
achievement_resource!(ScorchedEarthAchievement, AchievementId::ScorchedEarth);
achievement_resource!(
    ProtectiveInstinctsAchievement,
    AchievementId::ProtectiveInstincts
);
achievement_resource!(FriendlyThornsAchievement, AchievementId::FriendlyThorns);
achievement_resource!(SoiledSurpriseAchievement, AchievementId::SoiledSurprise);
achievement_resource!(MasterBrewerAchievement, AchievementId::MasterBrewer);

/// Run condition: returns true when the achievement resource is still locked.
pub(crate) fn achievement_locked<T: AchievementResource>(res: Res<T>) -> bool {
    res.is_locked()
}

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
    mut _msg: MessageReader<super::messages::ClearProgressMessage>,
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
}
