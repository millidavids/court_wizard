//! Tutorial entity tagging for highlight overlays.

use bevy::prelude::*;

use crate::ui::cauldron_menu::CauldronMenuButtonAction;
use crate::ui::in_game::{HudButtonAction, KingHealthBarFill, ManaBarFill, WaveDisplay};
use crate::ui::spell_book::{DetailName, HotkeySlotButton, ScrollableSpellList};
use crate::ui::wizard_tower::{
    InsightDisplay, LevelDisplay, SpellGraphArea, StudyButtonAction, StudyDetailPanel,
    TimeTravelContainer, WizardTowerButtonAction,
};

use super::components::TutorialHighlightable;
use super::definitions::HighlightTarget;

// ---------------------------------------------------------------------------
// Trigger systems
// ---------------------------------------------------------------------------

/// Starts a tutorial if it hasn't been completed and tutorials are enabled.
/// If another tutorial is already active, appends this one to
/// `PendingTutorials` so it plays as soon as the current one finishes.
/// `paused_gameplay` is honored only for the immediately-started case;
/// queued tutorials run un-paused (they're already deep enough into a
/// session that pausing again would be jarring).
#[allow(clippy::too_many_arguments)]
pub(super) fn tag_wizard_tower_entities(
    mut commands: Commands,
    level_displays: Query<Entity, (With<LevelDisplay>, Without<TutorialHighlightable>)>,
    insight_displays: Query<Entity, (With<InsightDisplay>, Without<TutorialHighlightable>)>,
    _wt_buttons: Query<(Entity, &WizardTowerButtonAction), Without<TutorialHighlightable>>,
    tt_containers: Query<Entity, (With<TimeTravelContainer>, Without<TutorialHighlightable>)>,
    tab_rows: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::WizardTowerTabRow>,
            Without<TutorialHighlightable>,
        ),
    >,
    wizard_grids: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::WizardCardScrollContainer>,
            Without<TutorialHighlightable>,
        ),
    >,
    left_panels: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::WizardTowerLeftPanel>,
            Without<TutorialHighlightable>,
        ),
    >,
    right_panels: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::WizardTowerRightPanel>,
            Without<TutorialHighlightable>,
        ),
    >,
    alloc_buttons: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::StudyAllocAdjustButton>,
            Without<TutorialHighlightable>,
        ),
    >,
    talents: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::TalentCard>,
            Without<TutorialHighlightable>,
        ),
    >,
    modifier_panels: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::RogueliteScrollableLeft>,
            Without<TutorialHighlightable>,
        ),
    >,
    seed_inputs: Query<
        Entity,
        (
            With<crate::ui::wizard_tower::SeedInputBox>,
            Without<TutorialHighlightable>,
        ),
    >,
) {
    for entity in &level_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::LevelDisplay,
        });
    }
    for entity in &insight_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::InsightDisplay,
        });
    }
    for entity in &tt_containers {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::TimeTravelList,
        });
    }
    for entity in &tab_rows {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::TabRow,
        });
    }
    for entity in &wizard_grids {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::WizardSelectGrid,
        });
    }
    for entity in &left_panels {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::LeftPanel,
        });
    }
    for entity in &right_panels {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::RightPanel,
        });
    }
    for entity in &alloc_buttons {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::AllocAdjustControls,
        });
    }
    for entity in &talents {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::TalentList,
        });
    }
    for entity in &modifier_panels {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::RogueliteModifierPanel,
        });
    }
    for entity in &seed_inputs {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SeedInput,
        });
    }
}

/// Tags Study screen UI entities with TutorialHighlightable.
pub(super) fn tag_study_entities(
    mut commands: Commands,
    spell_graph_areas: Query<Entity, (With<SpellGraphArea>, Without<TutorialHighlightable>)>,
    detail_panels: Query<Entity, (With<StudyDetailPanel>, Without<TutorialHighlightable>)>,
    study_buttons: Query<(Entity, &StudyButtonAction), Without<TutorialHighlightable>>,
) {
    for entity in &spell_graph_areas {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellGraphArea,
        });
    }
    for entity in &detail_panels {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::DetailPanel,
        });
    }
    for (entity, action) in &study_buttons {
        let target = match action {
            StudyButtonAction::Commit => HighlightTarget::CommitButton,
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => continue,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
}

/// Tags InGame HUD entities with TutorialHighlightable.
pub(super) fn tag_in_game_entities(
    mut commands: Commands,
    mana_bars: Query<Entity, (With<ManaBarFill>, Without<TutorialHighlightable>)>,
    king_health_bars: Query<Entity, (With<KingHealthBarFill>, Without<TutorialHighlightable>)>,
    wave_displays: Query<Entity, (With<WaveDisplay>, Without<TutorialHighlightable>)>,
    hud_buttons: Query<(Entity, &HudButtonAction), Without<TutorialHighlightable>>,
    action_bar_slots: Query<
        Entity,
        (
            With<crate::ui::action_bar::ActionBarSlot>,
            Without<TutorialHighlightable>,
        ),
    >,
) {
    for entity in &mana_bars {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::ManaBar,
        });
    }
    for entity in &king_health_bars {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::KingHealthBar,
        });
    }
    for entity in &wave_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::WaveDisplay,
        });
    }
    for (entity, action) in &hud_buttons {
        let target = match action {
            HudButtonAction::OpenSpellBook => HighlightTarget::SpellBookButton,
            HudButtonAction::OpenCauldronMenu => HighlightTarget::CauldronButton,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
    // Highlight each slot individually rather than the full-screen
    // ActionBarRoot container — the root spans the whole HUD bottom strip
    // and would dwarf the actual buttons.
    for entity in &action_bar_slots {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::ActionBar,
        });
    }
}

/// Tags Spell Book UI entities with TutorialHighlightable.
pub(super) fn tag_spell_book_entities(
    mut commands: Commands,
    spell_lists: Query<Entity, (With<ScrollableSpellList>, Without<TutorialHighlightable>)>,
    detail_names: Query<Entity, (With<DetailName>, Without<TutorialHighlightable>)>,
    hotkey_buttons: Query<Entity, (With<HotkeySlotButton>, Without<TutorialHighlightable>)>,
) {
    for entity in &spell_lists {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellList,
        });
    }
    for entity in &detail_names {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellDetail,
        });
    }
    if let Some(entity) = hotkey_buttons.iter().next() {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::HotkeySlots,
        });
    }
}

/// Tags Cauldron menu UI entities with TutorialHighlightable.
pub(super) fn tag_cauldron_entities(
    mut commands: Commands,
    cauldron_buttons: Query<(Entity, &CauldronMenuButtonAction), Without<TutorialHighlightable>>,
) {
    for (entity, action) in &cauldron_buttons {
        let target = match action {
            CauldronMenuButtonAction::StartBrew => HighlightTarget::BrewButton,
            _ => continue,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
}
