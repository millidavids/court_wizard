use bevy::prelude::*;

use crate::config::save_data::AchievementId;

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
achievement_resource!(RightToBearArmsAchievement, AchievementId::RightToBearArms);
achievement_resource!(CloseCallAchievement, AchievementId::CloseCall);
achievement_resource!(StormbringerAchievement, AchievementId::Stormbringer);
achievement_resource!(PacifistAchievement, AchievementId::Pacifist);

// Roguelite Modifier Achievements
achievement_resource!(ModWaveSpeedMinAch, AchievementId::ModWaveSpeedMin);
achievement_resource!(ModWaveSpeed100Ach, AchievementId::ModWaveSpeed100);
achievement_resource!(ModWaveSpeed200Ach, AchievementId::ModWaveSpeed200);
achievement_resource!(ModWaveSpeed300Ach, AchievementId::ModWaveSpeed300);
achievement_resource!(ModEnemyStrengthMinAch, AchievementId::ModEnemyStrengthMin);
achievement_resource!(ModEnemyStrength100Ach, AchievementId::ModEnemyStrength100);
achievement_resource!(ModEnemyStrength200Ach, AchievementId::ModEnemyStrength200);
achievement_resource!(ModEnemyStrength300Ach, AchievementId::ModEnemyStrength300);
achievement_resource!(ModEnemyCountMinAch, AchievementId::ModEnemyCountMin);
achievement_resource!(ModEnemyCount100Ach, AchievementId::ModEnemyCount100);
achievement_resource!(ModEnemyCount200Ach, AchievementId::ModEnemyCount200);
achievement_resource!(ModEnemyCount300Ach, AchievementId::ModEnemyCount300);
achievement_resource!(ModAllMinAch, AchievementId::ModAllMin);
achievement_resource!(ModAll200Ach, AchievementId::ModAll200);
achievement_resource!(ModAllMaxAch, AchievementId::ModAllMax);
achievement_resource!(ModMixedExtremesAch, AchievementId::ModMixedExtremes);
achievement_resource!(ClickerAchievement, AchievementId::Clicker);

// Completionist
achievement_resource!(GrandCouncilAchievement, AchievementId::GrandCouncil);
achievement_resource!(WalkingLibraryAchievement, AchievementId::WalkingLibrary);
achievement_resource!(PeakWizardAchievement, AchievementId::PeakWizard);

// Boss encounters & defeats
achievement_resource!(LichEncounterAchievement, AchievementId::LichEncounter);
achievement_resource!(
    DarkMageEncounterAchievement,
    AchievementId::DarkMageEncounter
);
achievement_resource!(HagsDefeatedAchievement, AchievementId::HagsDefeated);
achievement_resource!(OgreDefeatedAchievement, AchievementId::OgreDefeated);
achievement_resource!(LichDefeatedAchievement, AchievementId::LichDefeated);
achievement_resource!(DarkMageDefeatedAchievement, AchievementId::DarkMageDefeated);
achievement_resource!(RayEncounterAchievement, AchievementId::RayEncounter);
achievement_resource!(RayDefeatedAchievement, AchievementId::RayDefeated);

/// Tracks which bosses appeared during the current battle for defeat achievements.
#[derive(Resource, Default)]
pub(crate) struct BossesSeenThisBattle {
    pub hag: bool,
    pub ogre: bool,
    pub lich: bool,
    pub dark_mage: bool,
    pub ray: bool,
}

impl BossesSeenThisBattle {
    /// Number of distinct bosses seen this battle.
    pub(crate) fn count(&self) -> u32 {
        [self.hag, self.ogre, self.lich, self.dark_mage, self.ray]
            .iter()
            .filter(|&&seen| seen)
            .count() as u32
    }
}

/// Run condition: returns true when the achievement resource is still locked.
pub(crate) fn achievement_locked<T: AchievementResource>(res: Res<T>) -> bool {
    res.is_locked()
}
