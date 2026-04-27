use bevy::prelude::*;

/// Marker component for entities that belong to the compendium screen.
#[derive(Component)]
pub(super) struct OnCompendiumScreen;

/// Marker component for the back button.
#[derive(Component)]
pub(super) struct BackButton;

/// Marker component for the scrollable content container.
#[derive(Component)]
pub(super) struct ScrollableCompendiumContainer;

/// The active compendium tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum CompendiumTab {
    #[default]
    Spells,
    Ingredients,
    Units,
    Wizards,
    Achievements,
    Stats,
    Endless,
    Roguelite,
}

impl CompendiumTab {
    pub fn all() -> &'static [CompendiumTab] {
        &[
            CompendiumTab::Spells,
            CompendiumTab::Ingredients,
            CompendiumTab::Units,
            CompendiumTab::Wizards,
            CompendiumTab::Achievements,
            CompendiumTab::Stats,
            CompendiumTab::Endless,
            CompendiumTab::Roguelite,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            CompendiumTab::Spells => "Spells",
            CompendiumTab::Ingredients => "Ingredients",
            CompendiumTab::Units => "Units",
            CompendiumTab::Wizards => "Wizards",
            CompendiumTab::Achievements => "Achievements",
            CompendiumTab::Stats => "Stats",
            CompendiumTab::Endless => "Endless",
            CompendiumTab::Roguelite => "Roguelite",
        }
    }
}

/// Component on tab buttons to identify which tab they switch to.
#[derive(Component)]
pub(super) struct TabButton(pub CompendiumTab);

/// Component on item buttons to identify what they select.
#[derive(Component)]
pub(super) struct ItemButton(pub CompendiumItemId);

/// Identifies a selectable item in the compendium.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CompendiumItemId {
    Spell(String),
    Ingredient(String),
    Unit(String),
    Wizard(String),
    Achievement(String),
    /// A roguelite run, identified by its `started_at` timestamp.
    RogueliteRun(u64),
    /// Endless stats for a specific wizard type (debug name).
    EndlessWizardType(String),
}

/// Resource tracking the active tab and selected item.
#[derive(Resource)]
pub(super) struct CompendiumState {
    pub active_tab: CompendiumTab,
    pub selected_item: Option<CompendiumItemId>,
    /// Previous tab, used to detect tab-only changes without rebuilding items on selection.
    /// `None` means the items list has not been rendered yet — first frame builds it.
    pub prev_tab: Option<CompendiumTab>,
}

impl Default for CompendiumState {
    fn default() -> Self {
        Self {
            active_tab: CompendiumTab::Spells,
            selected_item: None,
            prev_tab: None,
        }
    }
}

/// Marker for the detail panel container.
#[derive(Component)]
pub(super) struct DetailPanel;

/// Marker for the spell icon in the detail panel.
#[derive(Component)]
pub(super) struct DetailIcon;

/// Marker for detail panel text fields.
#[derive(Component)]
pub(super) struct DetailTitle;

#[derive(Component)]
pub(super) struct DetailCategory;

#[derive(Component)]
pub(super) struct DetailDescription;

#[derive(Component)]
pub(super) struct DetailFlavor;

/// Marker for the right panel content area (tabs + items).
#[derive(Component)]
pub(super) struct RightPanelContent;

/// Marker for the items grid container inside the scrollable area.
#[derive(Component)]
pub(super) struct ItemsContainer;

/// Marker for the level history container in the detail panel (stats tab).
#[derive(Component)]
pub(super) struct LevelHistoryContainer;

/// Button to toggle save/unsave on a roguelite run. Stores the run's `started_at` timestamp.
#[derive(Component)]
pub(super) struct ToggleSaveRunButton(pub u64);

/// Button to copy a seed value to the clipboard.
#[derive(Component)]
pub(super) struct CopySeedButton(pub u64);
