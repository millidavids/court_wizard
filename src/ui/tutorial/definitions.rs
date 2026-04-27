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
    // Wizard Tower hub
    TabRow,
    WizardSelectGrid,
    LeftPanel,
    RightPanel,
    // Study sub-elements
    AllocAdjustControls,
    TalentList,
    // Roguelite sub-elements
    RogueliteModifierPanel,
    SeedInput,
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
    /// Text shown on controller. May contain `{token}` placeholders rendered
    /// as inline glyphs.
    pub text: &'static str,
    /// Optional override shown when the active input device is mouse +
    /// keyboard. When `None`, `text` is used and any `{token}` placeholders
    /// fall back to keyboard-label words.
    pub text_kbm: Option<&'static str>,
    pub anchor: PanelAnchor,
}

/// Tutorial identifier enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names, dead_code)]
pub(crate) enum TutorialId {
    /// Deprecated; replaced by per-tab tutorials. Kept for save-compat so
    /// players who completed it once aren't shown anything from this slot.
    WizardTowerIntro,
    /// Deprecated; replaced by `EndlessTabIntro` (which now covers time
    /// travel as part of the endless walkthrough).
    TimeTravelIntro,
    /// Fires the first time the player opens the Roguelite tab.
    RogueliteTabIntro,
    /// Fires the first time the player opens the Endless tab.
    EndlessTabIntro,
    /// Fires the first time the player opens the Study tab.
    StudyTabIntro,
    /// Deprecated original Study walkthrough; superseded by `StudyTabIntro`.
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
    /// Mouse + keyboard menu navigation primer. Shown once on the first
    /// time the Wizard Tower is entered while mouse/keyboard is the active
    /// input device.
    KbmMenusIntro,
    /// Walkthrough for the wizard-select panel reached via "Switch Wizard".
    WizardSelectIntro,
    /// Fires the first time the player selects a spell node in the Study,
    /// so the +/− and talent walkthrough only shows when those panels are
    /// actually visible.
    StudySpellSelectedIntro,
}

/// Which input modality a tutorial is tailored for. `Any` tutorials use
/// `text` + `text_kbm` to adapt text per-frame; modality-specific tutorials
/// are dismissed without completion if the player switches mid-overlay so the
/// matching counterpart (if any) can fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TutorialModality {
    Any,
    Gamepad,
    MouseKeyboard,
}

impl TutorialId {
    /// String identifier used for persistence.
    pub(crate) fn id(&self) -> &'static str {
        match self {
            TutorialId::WizardTowerIntro => "wizard_tower_intro",
            TutorialId::TimeTravelIntro => "time_travel_intro",
            TutorialId::RogueliteTabIntro => "roguelite_tab_intro",
            TutorialId::EndlessTabIntro => "endless_tab_intro",
            TutorialId::StudyTabIntro => "study_tab_intro",
            TutorialId::StudyIntro => "study_intro",
            TutorialId::InGameIntro => "in_game_intro",
            TutorialId::SpellBookIntro => "spell_book_intro",
            TutorialId::CauldronIntro => "cauldron_intro",
            TutorialId::ControllerMenusIntro => "controller_menus_intro",
            TutorialId::ControllerInGameIntro => "controller_in_game_intro",
            TutorialId::KbmMenusIntro => "kbm_menus_intro",
            TutorialId::WizardSelectIntro => "wizard_select_intro",
            TutorialId::StudySpellSelectedIntro => "study_spell_selected_intro",
        }
    }

    /// Returns the input modality this tutorial is tailored for.
    pub(crate) fn modality(&self) -> TutorialModality {
        match self {
            TutorialId::ControllerMenusIntro | TutorialId::ControllerInGameIntro => {
                TutorialModality::Gamepad
            }
            TutorialId::KbmMenusIntro | TutorialId::InGameIntro => TutorialModality::MouseKeyboard,
            _ => TutorialModality::Any,
        }
    }

    /// Returns the steps for this tutorial.
    pub(super) fn steps(&self) -> &'static [TutorialStep] {
        match self {
            TutorialId::WizardTowerIntro => WIZARD_TOWER_STEPS,
            TutorialId::TimeTravelIntro => TIME_TRAVEL_STEPS,
            TutorialId::RogueliteTabIntro => ROGUELITE_TAB_STEPS,
            TutorialId::EndlessTabIntro => ENDLESS_TAB_STEPS,
            TutorialId::StudyTabIntro => STUDY_TAB_STEPS,
            TutorialId::StudyIntro => STUDY_STEPS,
            TutorialId::InGameIntro => IN_GAME_STEPS,
            TutorialId::SpellBookIntro => SPELL_BOOK_STEPS,
            TutorialId::CauldronIntro => CAULDRON_STEPS,
            TutorialId::ControllerMenusIntro => CONTROLLER_MENUS_STEPS,
            TutorialId::ControllerInGameIntro => CONTROLLER_IN_GAME_STEPS,
            TutorialId::KbmMenusIntro => KBM_MENUS_STEPS,
            TutorialId::WizardSelectIntro => WIZARD_SELECT_STEPS,
            TutorialId::StudySpellSelectedIntro => STUDY_SPELL_SELECTED_STEPS,
        }
    }
}

// Per-tab tutorials — fired the first time each tab is opened. These are
// the modern walkthroughs and replace the old WizardTowerIntro/TimeTravelIntro
// flow that ran on tower entry.

static ROGUELITE_TAB_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Welcome to Roguelite mode! Build a custom run by tuning modifiers, then jump in.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::RogueliteModifierPanel,
        text: "The modifier panel has sliders — enemy speed, mana regen, wave size, and more. Drag a slider or use its +/- buttons to adjust the run's challenge.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::RogueliteModifierPanel,
        text: "Below the sliders are toggle modifiers that change the rules of the run. Some need to be unlocked with Arcane Insight before they can be used.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::SeedInput,
        text: "Set a custom seed for a reproducible run, or use the dice button to randomize it.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "When you're set, hit Start Run. Surviving waves earns Insight to spend in the Study.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
];

static ENDLESS_TAB_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Endless mode is a single escalating wave count — see how far you can push.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::LeftPanel,
        text: "The stats panel on the left tracks your best results per wizard archetype.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelList,
        text: "Time Travel lets you replay any level you've already reached. Earn Insight from each run, no matter the wave.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelList,
        text: "Pick a starting wave from the list, then hit Start to begin from that point.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
];

static STUDY_TAB_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "The Study is where you spend Arcane Insight to unlock spells and their talents.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::RightPanel,
        text: "The right panel is the spell web. Drag to pan around it; scroll to zoom in and out.",
        text_kbm: Some(
            "The right panel is the spell web. Click and drag to pan around it; scroll the mouse wheel to zoom.",
        ),
        anchor: PanelAnchor::CenterLeft,
    },
    TutorialStep {
        target: HighlightTarget::RightPanel,
        text: "Click a spell node to inspect it — the detail panel on the left will fill in with its description, cost, and any talents it has.",
        text_kbm: None,
        anchor: PanelAnchor::CenterLeft,
    },
];

// Fires the first time a Study spell node is selected. Continues the
// walkthrough where STUDY_TAB_STEPS leaves off, with the +/− and talent
// panels now visible to highlight.
static STUDY_SPELL_SELECTED_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::AllocAdjustControls,
        text: "Use the + and - buttons to commit Insight toward unlocking this spell, then press Commit.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::TalentList,
        text: "Once a spell is unlocked, its talents appear here. Talents are passive perks that change how the spell behaves — pick the ones that match your build.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
];

// Wizard Tower: UI is centered vertically. Level/Insight at top-center, buttons below.
static WIZARD_TOWER_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Welcome to your Wizard Tower! This is your home base between battles.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::LevelDisplay,
        text: "Here you can see your current level. Higher levels mean tougher enemies.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::InsightDisplay,
        text: "Arcane Insight is your currency for researching new spells. Earn it from battles and achievements.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::StudySpellsButton,
        text: "Click here to research and unlock new spells.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::StartBattleButton,
        text: "When you're ready, click here to start the next battle!",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
];

// Time Travel: Appears on the right side of the Wizard Tower screen.
static TIME_TRAVEL_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "You've unlocked Time Travel! Replay past levels to earn more Arcane Insight.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelList,
        text: "Select a level from this list to revisit it.",
        text_kbm: None,
        anchor: PanelAnchor::TopLeft,
    },
    TutorialStep {
        target: HighlightTarget::TimeTravelButton,
        text: "Then click here to start the battle at the selected level.",
        text_kbm: None,
        anchor: PanelAnchor::TopLeft,
    },
];

// Study: Spell graph fills most of screen, detail panel on the right.
static STUDY_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::SpellGraphArea,
        text: "This is your spell web. Each node is a spell you can research.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Click and drag to pan around. Scroll to zoom in and out.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::DetailPanel,
        text: "Click a spell node to see its details and research cost.",
        text_kbm: None,
        anchor: PanelAnchor::BottomLeft,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Drag the slider to allocate Arcane Insight toward researching a spell.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::CommitButton,
        text: "Click Commit to finalize your research. Insight is spent when you commit.",
        text_kbm: None,
        anchor: PanelAnchor::BottomRight,
    },
];

// InGame: Action bar bottom-left, mana bottom-right, king health middle,
// wave display top-right, spell/cauldron buttons top-left.
static IN_GAME_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Enemies are attacking! Protect the King using your spells.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ManaBar,
        text: "This is your mana bar. Spells cost mana, and it regenerates over time.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ActionBar,
        text: "This is your action bar where you can assign spells for quick selection. Press keys 1-5 to select a spell, then click the battlefield to cast.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::KingHealthBar,
        text: "Watch the King's health. If the King falls, you lose!",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::WaveDisplay,
        text: "Enemies arrive in waves. Survive all waves to win.",
        text_kbm: None,
        anchor: PanelAnchor::BottomLeft,
    },
    TutorialStep {
        target: HighlightTarget::SpellBookButton,
        text: "Need a different spell? Open your spell book here to swap hotkeys.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::CauldronButton,
        text: "Open the cauldron here to brew powerful buffs from ingredients dropped by enemies.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
];

// SpellBook: List on left, detail on right, hotkey slots at bottom.
static SPELL_BOOK_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::SpellList,
        text: "Browse your available spells here. Click one to prime it — the menu stays open so you can keep exploring.",
        text_kbm: None,
        anchor: PanelAnchor::TopRight,
    },
    TutorialStep {
        target: HighlightTarget::SpellDetail,
        text: "Read descriptions to understand each spell's effects, damage, and range.",
        text_kbm: None,
        anchor: PanelAnchor::TopLeft,
    },
    TutorialStep {
        target: HighlightTarget::HotkeySlots,
        text: "Click a hotkey slot (1-5) to assign the primed spell to your action bar.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press {east} to close the Spell Book when you're done.",
        text_kbm: Some("Click the close button (or press Esc) to leave the Spell Book."),
        anchor: PanelAnchor::Center,
    },
];

// Cauldron: Ingredients in center, brew button at bottom.
static CAULDRON_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "The Cauldron lets you brew buffs from ingredients dropped by enemies.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Select up to 3 ingredients. Different combinations create different effects.",
        text_kbm: None,
        anchor: PanelAnchor::BottomCenter,
    },
    TutorialStep {
        target: HighlightTarget::BrewButton,
        text: "Click Brew to start. Brews take time but provide powerful temporary buffs.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
];

// Controller menu navigation primer. Text-only; covers focus nav, confirm,
// back, tab switching, and pause.
static CONTROLLER_MENUS_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "Controller detected! Here's how to get around menus.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press {east} to go back or close a menu.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::TabRow,
        text: "Press {lb} or {rb} to cycle between tabs.",
        text_kbm: None,
        // Keep the window centered — moving it up would cover the
        // highlighted tab row.
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press {start} at any time during battle to pause.",
        text_kbm: None,
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
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "{lstick} aims. A thin line from your staff shows where your spell will land.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::ActionBar,
        text: "Hold {rstick} in a direction and press {rt} to pick that spell from the action bar.",
        text_kbm: None,
        anchor: PanelAnchor::TopCenter,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Once a spell is primed, tap or hold {rt} to cast it.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press {lt} to cancel an in-progress cast.",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::SpellBookButton,
        text: "Press {west} to open the Spell Book and swap spells.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::CauldronButton,
        text: "Press {north} to open the Cauldron and brew buffs.",
        text_kbm: None,
        anchor: PanelAnchor::CenterRight,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "Press {start} to pause. That's everything — good luck!",
        text_kbm: None,
        anchor: PanelAnchor::Center,
    },
];

// Mouse + keyboard menu navigation primer. Brief — most KBM users already
// know how to click. Fires on the first Wizard Tower visit while KBM is the
// active input device.
static KBM_MENUS_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::None,
        text: "",
        text_kbm: Some("Welcome! Click any button to interact, hover over things for tooltips."),
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::None,
        text: "",
        text_kbm: Some(
            "Press Esc at any time to back out of a menu or return to the previous screen.",
        ),
        anchor: PanelAnchor::Center,
    },
    TutorialStep {
        target: HighlightTarget::TabRow,
        text: "",
        text_kbm: Some(
            "Click the tabs at the top to switch between Roguelite, Endless, and Study.",
        ),
        anchor: PanelAnchor::Center,
    },
];

// Wizard select walkthrough — fires the first time the player opens the
// "Switch Wizard" panel. Highlights the wizard card grid.
static WIZARD_SELECT_STEPS: &[TutorialStep] = &[
    TutorialStep {
        target: HighlightTarget::WizardSelectGrid,
        text: "Pick your wizard archetype here. Each card shows the wizard's name, role, and signature spells.",
        text_kbm: None,
        anchor: PanelAnchor::CenterLeft,
    },
    TutorialStep {
        target: HighlightTarget::WizardSelectGrid,
        text: "Locked wizards (greyed out) need to be unlocked first — usually by clearing certain levels or earning achievements.",
        text_kbm: None,
        anchor: PanelAnchor::CenterLeft,
    },
    TutorialStep {
        target: HighlightTarget::WizardSelectGrid,
        text: "Scroll the grid to see every wizard. Pick one to set them as your active archetype, then back out to start a run.",
        text_kbm: Some(
            "Scroll with the mouse wheel to see every wizard. Click one to set them as your active archetype, then close the panel to start a run.",
        ),
        anchor: PanelAnchor::CenterLeft,
    },
];
