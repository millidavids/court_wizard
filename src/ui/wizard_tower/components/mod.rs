mod panel_components;
mod tab_components;

// ---------------------------------------------------------------------------
// pub(crate) re-exports (items needed by tutorial system and outside wizard_tower)
// ---------------------------------------------------------------------------

pub(crate) use panel_components::SelectedStudySpell;
pub(crate) use panel_components::SpellGraphArea;
pub(crate) use tab_components::InsightDisplay;
pub(crate) use tab_components::LevelDisplay;
pub(crate) use tab_components::StudyAllocAdjustButton;
pub(crate) use tab_components::StudyButtonAction;
pub(crate) use tab_components::StudyDetailPanel;
pub(crate) use tab_components::TalentCard;
pub(crate) use tab_components::TimeTravelContainer;
pub(crate) use tab_components::WizardTowerButtonAction;

// ---------------------------------------------------------------------------
// pub(super) re-exports (items used by wizard_tower sibling modules)
// ---------------------------------------------------------------------------

// Shared / main screen
pub(super) use tab_components::ArcaneRuneBackground;
pub(super) use tab_components::ArcaneRuneText;
pub(super) use tab_components::OnMainScreen;
pub(super) use tab_components::OnWizardTowerScreen;
pub(super) use tab_components::PendingInsightDisplay;
pub(super) use tab_components::SelectedTimeTravelLevel;
pub(super) use tab_components::StudyInsightDisplay;
pub(super) use tab_components::TimeTravelLevelButton;
pub(super) use tab_components::TimeTravelSection;
pub(super) use tab_components::TimeTravelSelectedDisplay;

#[cfg(debug_assertions)]
pub(super) use tab_components::WizardTowerUiContent;

// Study detail panel
pub(super) use tab_components::AllocTarget;
pub(super) use tab_components::StudyAllocationFill;
pub(super) use tab_components::StudyAllocationHandle;
pub(super) use tab_components::StudyAllocationSlider;
pub(super) use tab_components::StudyAllocationText;
pub(super) use tab_components::StudyProgressFill;
pub(super) use tab_components::TalentDescriptionText;
pub(super) use tab_components::TalentProgressBarFill;

// Graph resources and components
pub(super) use panel_components::FreeNode;
pub(super) use panel_components::GraphBounds;
pub(super) use panel_components::GraphDragState;
pub(super) use panel_components::GraphNodeLabel;
pub(super) use panel_components::GraphViewAnimation;
pub(super) use panel_components::GraphViewState;
pub(super) use panel_components::InsightAllocation;
pub(super) use panel_components::InsightBonusAllocationFill;
pub(super) use panel_components::InsightBonusAllocationText;
pub(super) use panel_components::InsightBonusNode;
pub(super) use panel_components::InsightBonusProgressFill;
pub(super) use panel_components::InsightBonusRings;
pub(super) use panel_components::InsightBonusSlider;
pub(super) use panel_components::InsightBonusSliderHandle;
pub(super) use panel_components::InsightConstellationAnchor;
pub(super) use panel_components::InsightConstellationEdge;
pub(super) use panel_components::SelectedInsightBonus;
pub(super) use panel_components::SpellGraphEdge;
pub(super) use panel_components::SpellGraphNode;
