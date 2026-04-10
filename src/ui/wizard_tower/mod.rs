mod components;
mod constants;
mod endless_tab;
mod graph;
mod layout;
mod materials;
mod plugin;
mod roguelite_tab;
mod study_tab;
mod wizard_cards;

pub use plugin::WizardTowerPlugin;

// Re-exports for tutorial system
pub(crate) use components::{
    InsightDisplay, LevelDisplay, SpellGraphArea, StudyButtonAction, StudyDetailPanel,
    TimeTravelContainer, WizardTowerButtonAction,
};
pub(crate) use layout::WizardTowerTab;
