//! AchievementId enum + display/id/description methods.

/// Type-safe achievement identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AchievementId {
    FirstVictory,
    FriendlyFire,
    ChainReaction,
    // Defeat & Failure
    TacticalRetreat,
    TheKingIsDead,
    TotalWipe,
    SpeedrunWrongDirection,
    PyrrhicDefeat,
    ItWasGoingSoWell,
    FriendlyFireDepartment,
    AccidentalRegicide,
    // Victory & Progression
    ApprenticeWizard,
    CourtWizard,
    Archmage,
    LegendsSpeakYourName,
    Immortalized,
    TheGrindNeverStops,
    OneMoreLevel,
    IntoTheDeep,
    Absurdity,
    Level100,
    Stubborn,
    ExtremelyStubborn,
    // Meta / Unlocks
    SliderFiddler,
    RandomMagicSurge,
    Qwer,
    // Spell Unlocks
    OutOfRange,
    ScorchedEarth,
    ProtectiveInstincts,
    FriendlyThorns,
    // Unit Encounters
    MeetTheBrute,
    EliteForces,
    CommanderOnTheField,
    EnemyMedic,
    MagicNullifier,
    TheThreeHags,
    OgreWarlord,
    SoiledSurprise,
    MasterBrewer,
    RightToBearArms,
    CloseCall,
    Stormbringer,
    Pacifist,
    // Roguelite Modifier Achievements
    ModWaveSpeedMin,
    ModWaveSpeed100,
    ModWaveSpeed200,
    ModWaveSpeed300,
    ModEnemyStrengthMin,
    ModEnemyStrength100,
    ModEnemyStrength200,
    ModEnemyStrength300,
    ModEnemyCountMin,
    ModEnemyCount100,
    ModEnemyCount200,
    ModEnemyCount300,
    ModAllMin,
    ModAll200,
    ModAllMax,
    ModMixedExtremes,
    Clicker,
    // Completionist
    GrandCouncil,
    WalkingLibrary,
    PeakWizard,
    // Boss encounters & defeats
    LichEncounter,
    DarkMageEncounter,
    RayEncounter,
    HagsDefeated,
    OgreDefeated,
    LichDefeated,
    DarkMageDefeated,
    RayDefeated,
}

impl AchievementId {
    /// Returns all achievement variants.
    pub(crate) fn all() -> &'static [AchievementId] {
        &[
            AchievementId::FirstVictory,
            AchievementId::FriendlyFire,
            AchievementId::ChainReaction,
            AchievementId::TacticalRetreat,
            AchievementId::TheKingIsDead,
            AchievementId::TotalWipe,
            AchievementId::SpeedrunWrongDirection,
            AchievementId::PyrrhicDefeat,
            AchievementId::ItWasGoingSoWell,
            AchievementId::FriendlyFireDepartment,
            AchievementId::AccidentalRegicide,
            AchievementId::ApprenticeWizard,
            AchievementId::CourtWizard,
            AchievementId::Archmage,
            AchievementId::LegendsSpeakYourName,
            AchievementId::Immortalized,
            AchievementId::TheGrindNeverStops,
            AchievementId::OneMoreLevel,
            AchievementId::IntoTheDeep,
            AchievementId::Absurdity,
            AchievementId::Level100,
            AchievementId::Stubborn,
            AchievementId::ExtremelyStubborn,
            AchievementId::SliderFiddler,
            AchievementId::RandomMagicSurge,
            AchievementId::Qwer,
            AchievementId::OutOfRange,
            AchievementId::ScorchedEarth,
            AchievementId::ProtectiveInstincts,
            AchievementId::FriendlyThorns,
            AchievementId::MeetTheBrute,
            AchievementId::EliteForces,
            AchievementId::CommanderOnTheField,
            AchievementId::EnemyMedic,
            AchievementId::MagicNullifier,
            AchievementId::TheThreeHags,
            AchievementId::OgreWarlord,
            AchievementId::SoiledSurprise,
            AchievementId::MasterBrewer,
            AchievementId::RightToBearArms,
            AchievementId::CloseCall,
            AchievementId::Stormbringer,
            AchievementId::Pacifist,
            // Roguelite Modifier Achievements
            AchievementId::ModWaveSpeedMin,
            AchievementId::ModWaveSpeed100,
            AchievementId::ModWaveSpeed200,
            AchievementId::ModWaveSpeed300,
            AchievementId::ModEnemyStrengthMin,
            AchievementId::ModEnemyStrength100,
            AchievementId::ModEnemyStrength200,
            AchievementId::ModEnemyStrength300,
            AchievementId::ModEnemyCountMin,
            AchievementId::ModEnemyCount100,
            AchievementId::ModEnemyCount200,
            AchievementId::ModEnemyCount300,
            AchievementId::ModAllMin,
            AchievementId::ModAll200,
            AchievementId::ModAllMax,
            AchievementId::ModMixedExtremes,
            AchievementId::Clicker,
            AchievementId::GrandCouncil,
            AchievementId::WalkingLibrary,
            AchievementId::PeakWizard,
            AchievementId::LichEncounter,
            AchievementId::DarkMageEncounter,
            AchievementId::RayEncounter,
            AchievementId::HagsDefeated,
            AchievementId::OgreDefeated,
            AchievementId::LichDefeated,
            AchievementId::DarkMageDefeated,
            AchievementId::RayDefeated,
        ]
    }

    /// String identifier used for persistence.
    pub(crate) fn id(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "first_victory",
            AchievementId::FriendlyFire => "friendly_fire",
            AchievementId::ChainReaction => "chain_reaction",
            AchievementId::TacticalRetreat => "tactical_retreat",
            AchievementId::TheKingIsDead => "the_king_is_dead",
            AchievementId::TotalWipe => "total_wipe",
            AchievementId::SpeedrunWrongDirection => "speedrun_wrong_direction",
            AchievementId::PyrrhicDefeat => "pyrrhic_defeat",
            AchievementId::ItWasGoingSoWell => "it_was_going_so_well",
            AchievementId::FriendlyFireDepartment => "friendly_fire_department",
            AchievementId::AccidentalRegicide => "accidental_regicide",
            AchievementId::ApprenticeWizard => "apprentice_wizard",
            AchievementId::CourtWizard => "court_wizard",
            AchievementId::Archmage => "archmage",
            AchievementId::LegendsSpeakYourName => "legends_speak_your_name",
            AchievementId::Immortalized => "immortalized",
            AchievementId::TheGrindNeverStops => "the_grind_never_stops",
            AchievementId::OneMoreLevel => "one_more_level",
            AchievementId::IntoTheDeep => "into_the_deep",
            AchievementId::Absurdity => "absurdity",
            AchievementId::Level100 => "level_100",
            AchievementId::Stubborn => "stubborn",
            AchievementId::ExtremelyStubborn => "extremely_stubborn",
            AchievementId::SliderFiddler => "slider_fiddler",
            AchievementId::RandomMagicSurge => "random_magic_surge",
            AchievementId::Qwer => "qwer",
            AchievementId::OutOfRange => "out_of_range",
            AchievementId::ScorchedEarth => "scorched_earth",
            AchievementId::ProtectiveInstincts => "protective_instincts",
            AchievementId::FriendlyThorns => "friendly_thorns",
            AchievementId::MeetTheBrute => "meet_the_brute",
            AchievementId::EliteForces => "elite_forces",
            AchievementId::CommanderOnTheField => "commander_on_the_field",
            AchievementId::EnemyMedic => "enemy_medic",
            AchievementId::MagicNullifier => "magic_nullifier",
            AchievementId::TheThreeHags => "the_three_hags",
            AchievementId::OgreWarlord => "ogre_warlord",
            AchievementId::SoiledSurprise => "soiled_surprise",
            AchievementId::MasterBrewer => "master_brewer",
            AchievementId::RightToBearArms => "right_to_bear_arms",
            AchievementId::CloseCall => "close_call",
            AchievementId::Stormbringer => "stormbringer",
            AchievementId::Pacifist => "pacifist",
            AchievementId::ModWaveSpeedMin => "mod_wave_speed_min",
            AchievementId::ModWaveSpeed100 => "mod_wave_speed_100",
            AchievementId::ModWaveSpeed200 => "mod_wave_speed_200",
            AchievementId::ModWaveSpeed300 => "mod_wave_speed_300",
            AchievementId::ModEnemyStrengthMin => "mod_enemy_strength_min",
            AchievementId::ModEnemyStrength100 => "mod_enemy_strength_100",
            AchievementId::ModEnemyStrength200 => "mod_enemy_strength_200",
            AchievementId::ModEnemyStrength300 => "mod_enemy_strength_300",
            AchievementId::ModEnemyCountMin => "mod_enemy_count_min",
            AchievementId::ModEnemyCount100 => "mod_enemy_count_100",
            AchievementId::ModEnemyCount200 => "mod_enemy_count_200",
            AchievementId::ModEnemyCount300 => "mod_enemy_count_300",
            AchievementId::ModAllMin => "mod_all_min",
            AchievementId::ModAll200 => "mod_all_200",
            AchievementId::ModAllMax => "mod_all_max",
            AchievementId::ModMixedExtremes => "mod_mixed_extremes",
            AchievementId::Clicker => "clicker",
            AchievementId::GrandCouncil => "all_wizards",
            AchievementId::WalkingLibrary => "all_spells",
            AchievementId::PeakWizard => "all_bonuses_maxed",
            AchievementId::LichEncounter => "lich_encounter",
            AchievementId::DarkMageEncounter => "dark_mage_encounter",
            AchievementId::RayEncounter => "ray_encounter",
            AchievementId::HagsDefeated => "hags_defeated",
            AchievementId::OgreDefeated => "ogre_defeated",
            AchievementId::LichDefeated => "lich_defeated",
            AchievementId::DarkMageDefeated => "dark_mage_defeated",
            AchievementId::RayDefeated => "ray_defeated",
        }
    }

    /// Display name shown in the achievement popup.
    pub(crate) fn display_name(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "First Victory",
            AchievementId::FriendlyFire => "Friendly Fire",
            AchievementId::ChainReaction => "Chain Reaction",
            AchievementId::TacticalRetreat => "Tactical Retreat",
            AchievementId::TheKingIsDead => "The King is Dead",
            AchievementId::TotalWipe => "Total Wipe",
            AchievementId::SpeedrunWrongDirection => "Speedrun (Wrong Direction)",
            AchievementId::PyrrhicDefeat => "Pyrrhic Defeat",
            AchievementId::ItWasGoingSoWell => "It Was Going So Well",
            AchievementId::FriendlyFireDepartment => "Friendly Fire Department",
            AchievementId::AccidentalRegicide => "Accidental Regicide",
            AchievementId::ApprenticeWizard => "Apprentice Wizard",
            AchievementId::CourtWizard => "Court Wizard",
            AchievementId::Archmage => "Archmage",
            AchievementId::LegendsSpeakYourName => "Legends Speak Your Name",
            AchievementId::Immortalized => "Immortalized",
            AchievementId::TheGrindNeverStops => "The Grind Never Stops",
            AchievementId::OneMoreLevel => "One More Level",
            AchievementId::IntoTheDeep => "Into the Deep",
            AchievementId::Absurdity => "Absurdity",
            AchievementId::Level100 => "Level 100",
            AchievementId::Stubborn => "Stubborn",
            AchievementId::ExtremelyStubborn => "Extremely Stubborn",
            AchievementId::SliderFiddler => "Slider Fiddler",
            AchievementId::RandomMagicSurge => "Random Magic Surge",
            AchievementId::Qwer => "QWER",
            AchievementId::OutOfRange => "Out of Range",
            AchievementId::ScorchedEarth => "Scorched Earth",
            AchievementId::ProtectiveInstincts => "Protective Instincts",
            AchievementId::FriendlyThorns => "Friendly Thorns",
            AchievementId::MeetTheBrute => "Meet the Brute",
            AchievementId::EliteForces => "Elite Forces",
            AchievementId::CommanderOnTheField => "Commander on the Field",
            AchievementId::EnemyMedic => "Enemy Medic!",
            AchievementId::MagicNullifier => "Magic Nullifier",
            AchievementId::TheThreeHags => "The Three Hags",
            AchievementId::OgreWarlord => "Ogre Warlord",
            AchievementId::SoiledSurprise => "Soiled Surprise",
            AchievementId::MasterBrewer => "Master Brewer",
            AchievementId::RightToBearArms => "The Right to Bear Arms",
            AchievementId::CloseCall => "Close Call",
            AchievementId::Stormbringer => "Stormbringer",
            AchievementId::Pacifist => "Pacifist",
            AchievementId::ModWaveSpeedMin => "Leisurely Pace",
            AchievementId::ModWaveSpeed100 => "Right On Time",
            AchievementId::ModWaveSpeed200 => "Double Time",
            AchievementId::ModWaveSpeed300 => "Blitz",
            AchievementId::ModEnemyStrengthMin => "Mercy Rule",
            AchievementId::ModEnemyStrength100 => "Fair Fight",
            AchievementId::ModEnemyStrength200 => "Battle Hardened",
            AchievementId::ModEnemyStrength300 => "Unstoppable Force",
            AchievementId::ModEnemyCountMin => "Skeleton Crew",
            AchievementId::ModEnemyCount100 => "Standard Forces",
            AchievementId::ModEnemyCount200 => "Double Trouble",
            AchievementId::ModEnemyCount300 => "Army of Darkness",
            AchievementId::ModAllMin => "Minimalist",
            AchievementId::ModAll200 => "Overachiever",
            AchievementId::ModAllMax => "Absolute Madness",
            AchievementId::ModMixedExtremes => "Glass Cannon",
            AchievementId::Clicker => "Clicker",
            AchievementId::GrandCouncil => "Grand Council",
            AchievementId::WalkingLibrary => "Walking Library",
            AchievementId::PeakWizard => "Peak Wizard",
            AchievementId::LichEncounter => "Beyond the Grave",
            AchievementId::DarkMageEncounter => "Arcane Rival",
            AchievementId::RayEncounter => "Under Scrutiny",
            AchievementId::HagsDefeated => "Family Reunion Cancelled",
            AchievementId::OgreDefeated => "Brought Down to Size",
            AchievementId::LichDefeated => "Final Death",
            AchievementId::DarkMageDefeated => "Tenure Denied",
            AchievementId::RayDefeated => "Blind Justice",
        }
    }

    /// Returns the unlock reward text for this achievement.
    pub(crate) fn unlock_reward(&self) -> Option<&'static str> {
        match self {
            AchievementId::FirstVictory => Some("Grants: 25 Arcane Insight"),
            AchievementId::FriendlyFire => Some("Grants: 15 Arcane Insight"),
            AchievementId::ChainReaction => Some("Grants: 20 Arcane Insight"),
            AchievementId::TheKingIsDead => Some("Grants: 15 Arcane Insight"),
            AchievementId::ApprenticeWizard => Some("Grants: 40 Arcane Insight"),
            AchievementId::CourtWizard => Some("Grants: 60 Arcane Insight"),
            AchievementId::Archmage => Some("Grants: 80 Arcane Insight"),
            AchievementId::LegendsSpeakYourName => Some("Grants: 100 Arcane Insight"),
            AchievementId::SliderFiddler => Some("Unlocks: Arcanorouter wizard"),
            AchievementId::RandomMagicSurge => Some("Unlocks: Randomancer wizard"),
            AchievementId::Qwer => Some("Unlocks: Rune Caster wizard"),
            AchievementId::OutOfRange => Some("Grants: 15 Arcane Insight"),
            AchievementId::ScorchedEarth => Some("Grants: 15 Arcane Insight"),
            AchievementId::ProtectiveInstincts => Some("Grants: 15 Arcane Insight"),
            AchievementId::FriendlyThorns => Some("Grants: 15 Arcane Insight"),
            AchievementId::SoiledSurprise => Some("Unlocks: Excremage wizard"),
            AchievementId::MasterBrewer => Some("Unlocks: The Alchemist wizard"),
            AchievementId::RightToBearArms => Some("Unlocks: Warglock wizard"),
            AchievementId::CloseCall => Some("Unlocks: Swordcerer wizard"),
            AchievementId::Stormbringer => Some("Unlocks: Meteorologist wizard"),
            AchievementId::Pacifist => Some("Unlocks: Shepherd wizard"),
            AchievementId::AccidentalRegicide => Some("Unlocks: Psychopath wizard"),
            AchievementId::ModWaveSpeed300 => Some("Grants: 25 Arcane Insight"),
            AchievementId::ModEnemyStrength300 => Some("Grants: 25 Arcane Insight"),
            AchievementId::ModEnemyCount300 => Some("Grants: 25 Arcane Insight"),
            AchievementId::ModAll200 => Some("Grants: 50 Arcane Insight"),
            AchievementId::ModAllMax => Some("Grants: 100 Arcane Insight"),
            AchievementId::Clicker => Some("Grants: 50 Arcane Insight"),
            AchievementId::GrandCouncil => Some("Grants: 100 Arcane Insight"),
            AchievementId::WalkingLibrary => Some("Grants: 100 Arcane Insight"),
            AchievementId::PeakWizard => Some("Grants: 100 Arcane Insight"),
            _ => None,
        }
    }

    /// Returns the Arcane Insight reward for this achievement (0 if none).
    pub(crate) fn insight_reward(&self) -> u32 {
        match self {
            AchievementId::FirstVictory => 25,
            AchievementId::FriendlyFire => 15,
            AchievementId::ChainReaction => 20,
            AchievementId::TheKingIsDead => 15,
            AchievementId::ApprenticeWizard => 40,
            AchievementId::CourtWizard => 60,
            AchievementId::Archmage => 80,
            AchievementId::LegendsSpeakYourName => 100,
            AchievementId::OutOfRange => 15,
            AchievementId::ScorchedEarth => 15,
            AchievementId::ProtectiveInstincts => 15,
            AchievementId::FriendlyThorns => 15,
            AchievementId::ModWaveSpeed300 => 25,
            AchievementId::ModEnemyStrength300 => 25,
            AchievementId::ModEnemyCount300 => 25,
            AchievementId::ModAll200 => 50,
            AchievementId::ModAllMax => 100,
            AchievementId::Clicker => 50,
            AchievementId::GrandCouncil => 100,
            AchievementId::WalkingLibrary => 100,
            AchievementId::PeakWizard => 100,
            _ => 0,
        }
    }

    /// Description shown in the achievement popup.
    pub(crate) fn description(&self) -> &'static str {
        match self {
            AchievementId::FirstVictory => "You won your first battle!",
            AchievementId::FriendlyFire => "You killed a defender with a spell. Oops!",
            AchievementId::ChainReaction => "Killed multiple enemies in quick succession.",
            AchievementId::TacticalRetreat => "A strategic withdrawal to reconsider your options.",
            AchievementId::TheKingIsDead => "You had one job.",
            AchievementId::TotalWipe => {
                "Not a single soldier left standing. Impressive, in a horrible way."
            }
            AchievementId::SpeedrunWrongDirection => "The battle barely started. What happened?",
            AchievementId::PyrrhicDefeat => "Close, but no cigar.",
            AchievementId::ItWasGoingSoWell => "Everything was fine. And then it wasn't.",
            AchievementId::FriendlyFireDepartment => "Maybe stop casting into your own army?",
            AchievementId::AccidentalRegicide => {
                "The defense has been called off on account of you."
            }
            AchievementId::ApprenticeWizard => "You're getting the hang of this.",
            AchievementId::CourtWizard => "The king trusts you. Probably a mistake.",
            AchievementId::Archmage => "Your beard has grown three inches since you started.",
            AchievementId::LegendsSpeakYourName => {
                "Bards write songs. Most of them are inaccurate."
            }
            AchievementId::Immortalized => {
                "Statues of you line the courtyard. They all look wrong."
            }
            AchievementId::TheGrindNeverStops => "You could have learned a real trade by now.",
            AchievementId::OneMoreLevel => "Just one more, you told yourself 9 levels ago.",
            AchievementId::IntoTheDeep => "The attackers are starting to look... different.",
            AchievementId::Absurdity => "This many enemies shouldn't fit on one battlefield.",
            AchievementId::Level100 => {
                "You've been doing this longer than some civilizations lasted."
            }
            AchievementId::Stubborn => {
                "Insanity is doing the same thing over and over. But maybe this time..."
            }
            AchievementId::ExtremelyStubborn => "At this point, the enemies feel bad for you.",
            AchievementId::SliderFiddler => "You adjusted a slider. The Arcanorouter approves.",
            AchievementId::RandomMagicSurge => "Lol, you're so random.",
            AchievementId::Qwer => "Spell-keyboard?",
            AchievementId::OutOfRange => "A defender wandered beyond your reach.",
            AchievementId::ScorchedEarth => "Your own fire claimed a life.",
            AchievementId::ProtectiveInstincts => "You shielded the enemy. On purpose?",
            AchievementId::FriendlyThorns => "Your vines don't discriminate.",
            AchievementId::MeetTheBrute => "They're bigger than expected.",
            AchievementId::EliteForces => "The enemy brought their best.",
            AchievementId::CommanderOnTheField => "Someone out there is giving orders.",
            AchievementId::EnemyMedic => "They have healers now. Great.",
            AchievementId::MagicNullifier => "Your spells are being countered.",
            AchievementId::TheThreeHags => "Three sisters. One shared grudge.",
            AchievementId::OgreWarlord => "The biggest one yet. And it's angry.",
            AchievementId::SoiledSurprise => {
                "Something terrible happened. And it unlocked a wizard type."
            }
            AchievementId::MasterBrewer => {
                "You've collected every ingredient. The cauldron recognizes a true master."
            }
            AchievementId::RightToBearArms => {
                "Killed an enemy marked by Finger of Death. Guns are now an option."
            }
            AchievementId::CloseCall => {
                "An enemy got dangerously close to the tower. Maybe it's time to get your hands dirty."
            }
            AchievementId::Stormbringer => "Lightning meets wind. The sky bends to your will.",
            AchievementId::Pacifist => {
                "You won without your spells hurting a single enemy. There is another way."
            }
            AchievementId::ModWaveSpeedMin => {
                "Completed a roguelite run at the slowest wave speed."
            }
            AchievementId::ModWaveSpeed100 => "Completed a roguelite run at normal wave speed.",
            AchievementId::ModWaveSpeed200 => "Completed a roguelite run at double wave speed.",
            AchievementId::ModWaveSpeed300 => {
                "Completed a roguelite run at maximum wave speed. No breaks!"
            }
            AchievementId::ModEnemyStrengthMin => {
                "Completed a roguelite run with enemies at their weakest."
            }
            AchievementId::ModEnemyStrength100 => {
                "Completed a roguelite run with normal enemy strength."
            }
            AchievementId::ModEnemyStrength200 => {
                "Completed a roguelite run with double-strength enemies."
            }
            AchievementId::ModEnemyStrength300 => {
                "Completed a roguelite run with maximum-strength enemies."
            }
            AchievementId::ModEnemyCountMin => {
                "Completed a roguelite run with a skeleton crew of enemies."
            }
            AchievementId::ModEnemyCount100 => {
                "Completed a roguelite run with the standard enemy count."
            }
            AchievementId::ModEnemyCount200 => "Completed a roguelite run with double the enemies.",
            AchievementId::ModEnemyCount300 => {
                "Completed a roguelite run against the maximum enemy horde."
            }
            AchievementId::ModAllMin => "Completed a roguelite run with every modifier at minimum.",
            AchievementId::ModAll200 => "Completed a roguelite run with everything doubled.",
            AchievementId::ModAllMax => {
                "Completed a roguelite run with every modifier maxed out. You're insane."
            }
            AchievementId::ModMixedExtremes => {
                "Completed a roguelite run with one modifier maxed and another at minimum."
            }
            AchievementId::Clicker => "Won a roguelite run using only your mouse.",
            AchievementId::GrandCouncil => {
                "Every wizard in the realm has answered your summons. Whether that's a good thing remains to be seen."
            }
            AchievementId::WalkingLibrary => {
                "Every spell ever catalogued now lives in your memory. Or close enough."
            }
            AchievementId::PeakWizard => {
                "Every permanent upgrade, fully mastered. The court whispers of a wizard without ceiling."
            }
            AchievementId::LichEncounter => "Death was supposed to be the end. Someone disagrees.",
            AchievementId::DarkMageEncounter => {
                "A wizard who chose the other side. How embarrassing for them."
            }
            AchievementId::RayEncounter => {
                "Six eyes, one body, and a very bad attitude. It sees everything."
            }
            AchievementId::HagsDefeated => "Three sisters walked in. None walked out.",
            AchievementId::OgreDefeated => "The bigger they are, the more satisfying the thud.",
            AchievementId::LichDefeated => {
                "You'd think dying once would be enough. This time, stay dead."
            }
            AchievementId::DarkMageDefeated => {
                "The dark mage's application has been rejected. Permanently."
            }
            AchievementId::RayDefeated => "Every last eye, shut. Nothing left to see here.",
        }
    }
}
