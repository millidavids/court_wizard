//! Tutorial step definitions and content.

/// Identifies which UI element to highlight in a tutorial step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightTarget {
    // Wizard Tower
    LevelDisplay,
    InsightDisplay,
    StudySpellsButton,
    StartBattleButton,
    TimeTravelButton,
    TimeTravelList,
    // Study
    SpellGraphArea,
    DetailPanel,
    CommitButton,
    // InGame
    ActionBar,
    ManaBar,
    KingHealthBar,
    WaveDisplay,
    SpellBookButton,
    CauldronButton,
    // SpellBook
    SpellList,
    SpellDetail,
    HotkeySlots,
    // Cauldron
    BrewButton,
    // No highlight (text-only step)
    None,
}

/// Where to anchor the tutorial panel on screen.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) enum PanelAnchor {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    CenterLeft,
    CenterRight,
}

/// A single step in a tutorial.
pub(super) struct TutorialStep {
    pub target: HighlightTarget,
    pub text: &'static str,
    pub anchor: PanelAnchor,
}

/// Tutorial identifier enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum TutorialId {
    WizardTowerIntro,
    TimeTravelIntro,
    StudyIntro,
    InGameIntro,
    SpellBookIntro,
    CauldronIntro,
    /// Controller-specific menu navigation primer. Shown once on the first
    /// time the Wizard Tower is entered while a gamepad is the active
    /// input device.
    ControllerMenusIntro,
    /// Controller-specific in-game combat primer. Shown once on the first
    /// gameplay session entered while a gamepad is the active input device.
    ControllerInGameIntro,
}

impl TutorialId {
    /// String identifier used for persistence.
    pub(crate) fn id(&self) -> &'static str {
        match self {
            TutorialId::WizardTowerIntro => "wizard_tower_intro",
            TutorialId::TimeTravelIntro => "time_travel_intro",
            TutorialId::StudyIntro => "study_intro",
            TutorialId::InGameIntro => "in_game_intro",
            TutorialId::SpellBookIntro => "spell_book_intro",
            TutorialId::CauldronIntro => "cauldron_intro",
            TutorialId::ControllerMenusIntro => "controller_menus_intro",
            TutorialId::ControllerInGameIntro => "controller_in_game_intro",
        }
    }

    /// Returns the steps for this tutorial.
    pub(super) fn steps(&self) -> &'static [TutorialStep] {
        match self {
            TutorialId::WizardTowerIntro => WIZARD_TOWER_STEPS,
            TutorialId::TimeTravelIntro => TIME_TRAVEL_STEPS,
            TutorialId::StudyIntro => STUDY_STEPS,
            TutorialId::InGameIntro => IN_GAME_STEPS,
            TutorialId::SpellBookIntro => SPELL_BOOK_STEPS,
            TutorialId::CauldronIntro => CAULDRON_STEPS,
            TutorialId::ControllerMenusIntro => CONTROLLER_MENUS_STEPS,
            TutorialId::ControllerInGameIntro => CONTROLLER_IN_GAME_STEPS,
        }
    }
}

// Wizard Tower: UI is centered vertically. Level/Insight at top-center, buttons below.
static WIZARD_TOWER_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Welcome to your Wizard Tower! This is your home base between battles.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::LevelDisplay,
        text: "Here you can see your current level. Higher levels mean tougher enemies.",
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::InsightDisplay,
        text: "Arcane Insight is your currency for researching new spells. Earn it from battles and achievements.",
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::StudySpellsButton,
        text: "Click here to research and unlock new spells.",
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::StartBattleButton,
        text: "When you're ready, click here to start the next battle!",
        anchor: PanelAnchor::BottomCenter,
    },
];

// Time Travel: Appears on the right side of the Wizard Tower screen.
static TIME_TRAVEL_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "You've unlocked Time Travel! Replay past levels to earn more Arcane Insight.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelList,
        text: "Select a level from this list to revisit it.",
        anchor: PanelAnchor::TopLeft,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelButton,
        text: "Then click here to start the battle at the selected level.",
        anchor: PanelAnchor::TopLeft,
    },
];

// Study: Spell graph fills most of screen, detail panel on the right.
static STUDY_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::SpellGraphArea,
        text: "This is your spell web. Each node is a spell you can research.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Click and drag to pan around. Scroll to zoom in and out.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::DetailPanel,
        text: "Click a spell node to see its details and research cost.",
        anchor: PanelAnchor::BottomLeft,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Drag the slider to allocate Arcane Insight toward researching a spell.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::CommitButton,
        text: "Click Commit to finalize your research. Insight is spent when you commit.",
        anchor: PanelAnchor::BottomRight,
    },
];

// InGame: Action bar bottom-left, mana bottom-right, king health middle,
// wave display top-right, spell/cauldron buttons top-left.
static IN_GAME_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Enemies are attacking! Protect the King using your spells.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ManaBar,
        text: "This is your mana bar. Spells cost mana, and it regenerates over time.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ActionBar,
        text: "This is your action bar where you can assign spells for quick selection. Press keys 1-5 to select a spell, then click the battlefield to cast.",
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::KingHealthBar,
        text: "Watch the King's health. If the King falls, you lose!",
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::WaveDisplay,
        text: "Enemies arrive in waves. Survive all waves to win.",
        anchor: PanelAnchor::BottomLeft,
    },
    TutorialStep {
        target: HighlightTarget::SpellBookButton,
        text: "Need a different spell? Open your spell book here to swap hotkeys.",
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::CauldronButton,
        text: "Open the cauldron here to brew powerful buffs from ingredients dropped by enemies.",
        anchor: PanelAnchor::CenterRight,
    },
];

// SpellBook: List on left, detail on right, hotkey slots at bottom.
static SPELL_BOOK_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::SpellList,
        text: "Browse your available spells here. Click one to prime it — the menu stays open so you can keep exploring.",
        anchor: PanelAnchor::TopRight,
    },
    TutorialStep {
        target: HighlightTarget::SpellDetail,
        text: "Read descriptions to understand each spell's effects, damage, and range.",
        anchor: PanelAnchor::TopLeft,
    },
    TutorialStep {
        target: HighlightTarget::HotkeySlots,
        text: "Click a hotkey slot (1-5) to assign the primed spell to your action bar.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press the back button (or B / Circle on a controller) to close the Spell Book when you're done.",
        anchor: PanelAnchor::Center,
    },
];

// Cauldron: Ingredients in center, brew button at bottom.
static CAULDRON_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "The Cauldron lets you brew buffs from ingredients dropped by enemies.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Select up to 3 ingredients. Different combinations create different effects.",
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::BrewButton,
        text: "Click Brew to start. Brews take time but provide powerful temporary buffs.",
        anchor: PanelAnchor::Center,
    },
];

// Controller menu navigation primer. Text-only; covers focus nav, confirm,
// back, tab switching, and pause.
static CONTROLLER_MENUS_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Controller detected! Here's how to get around menus.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Use the D-pad or left stick to move the highlight between buttons.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press the bottom face button (A / Cross) to select the highlighted button.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press the right face button (B / Circle) to go back or close a menu.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press the Left / Right bumpers (LB / RB) to cycle between tabs.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press Start / Options at any time during battle to pause.",
        anchor: PanelAnchor::Center,
    },
];

// Controller in-game combat primer. Teaches the aim + cast + radial + HUD
// shortcuts tied to the controller layout. Fires with `paused_gameplay: true`
// so the player can read it without the battle running behind.
static CONTROLLER_IN_GAME_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Combat on a controller: let's go through the battle controls.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "The left stick aims. A thin line from your staff shows where your spell will land.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ActionBar,
        text: "Hold the right stick in a direction and press the Right Trigger to pick that spell from the action bar.",
        anchor: PanelAnchor::TopCenter,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Once a spell is primed, tap or hold the Right Trigger to cast it.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press the Left Trigger to cancel an in-progress cast.",
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::SpellBookButton,
        text: "Press the left face button (X / Square) to open the Spell Book and swap spells.",
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::CauldronButton,
        text: "Press the top face button (Y / Triangle) to open the Cauldron and brew buffs.",
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press Start / Options to pause. That's everything — good luck!",
        anchor: PanelAnchor::Center,
    },
];
