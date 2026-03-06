mod components;
mod constants;
mod graph;
mod materials;
mod plugin;
mod systems;

pub use plugin::WizardTowerPlugin;

// Re-exports for tutorial system
pub(crate) use components::{
    InsightDisplay, LevelDisplay, SpellGraphArea, StudyButtonAction, StudyDetailPanel,
    TimeTravelContainer, WizardTowerButtonAction,
};
