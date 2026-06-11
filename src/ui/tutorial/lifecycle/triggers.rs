use bevy::prelude::*;

use crate::config::GameConfig;
use crate::state::InGameState;

use super::super::components::{HighlightOverlay, TutorialOverlay};
use super::super::definitions::{TutorialId, TutorialModality};
use super::super::resources::{ActiveTutorial, TutorialProgress};
use super::highlight::remove_all_highlights;
use super::overlay::despawn_overlay;

/// Starts a tutorial if it hasn't been completed and tutorials are enabled.
/// If another tutorial is already active, appends this one to
/// `PendingTutorials` so it plays as soon as the current one finishes.
/// `paused_gameplay` is honored only for the immediately-started case;
/// queued tutorials run un-paused (they're already deep enough into a
/// session that pausing again would be jarring).
fn try_start_tutorial(
    commands: &mut Commands,
    tutorial: TutorialId,
    progress: &TutorialProgress,
    config: &GameConfig,
    active: Option<&ActiveTutorial>,
    pending: Option<&super::super::resources::PendingTutorials>,
    paused_gameplay: bool,
) -> bool {
    if !config.tutorials_enabled {
        return false;
    }
    if progress.is_completed(&tutorial) {
        return false;
    }
    if let Some(active) = active {
        if active.tutorial == tutorial {
            return false;
        }
        let _ = pending;
        commands.queue(move |world: &mut bevy::prelude::World| {
            let mut q = world
                .get_resource_or_insert_with::<super::super::resources::PendingTutorials>(
                    Default::default,
                );
            if !q.queue.contains(&tutorial) {
                q.queue.push_back(tutorial);
            }
        });
        return false;
    }

    commands.insert_resource(ActiveTutorial {
        tutorial,
        step: 0,
        paused_gameplay,
    });
    true
}

// Old `trigger_wizard_tower_tutorial`, `trigger_time_travel_tutorial`, and
// `trigger_study_tutorial` were removed when the tower walkthrough was split
// into per-tab tutorials. Their `TutorialId` variants stay in the enum so
// existing saves that completed those overlays don't get them re-shown under
// any future revival.

pub(crate) fn trigger_in_game_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if try_start_tutorial(
        &mut commands,
        TutorialId::InGameIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        true,
    ) {
        next_in_game_state.set(InGameState::Tutorial);
    }
}

pub(crate) fn trigger_spell_book_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::SpellBookIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

pub(crate) fn trigger_cauldron_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::CauldronIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Pops the next tutorial off `PendingTutorials` and inserts it as the new
/// `ActiveTutorial` whenever no tutorial is currently active. Skips entries
/// the player has already completed (e.g. via Skip while another was queued).
pub(crate) fn drain_pending_tutorials(
    mut commands: Commands,
    active: Option<Res<ActiveTutorial>>,
    mut pending: Option<ResMut<super::super::resources::PendingTutorials>>,
    progress: Res<TutorialProgress>,
) {
    if active.is_some() {
        return;
    }
    let Some(pending) = pending.as_deref_mut() else {
        return;
    };
    while let Some(next) = pending.queue.pop_front() {
        if progress.is_completed(&next) {
            continue;
        }
        commands.insert_resource(ActiveTutorial {
            tutorial: next,
            step: 0,
            paused_gameplay: false,
        });
        return;
    }
}

/// Mouse + keyboard menu navigation primer. Mirrors the controller variant —
/// fires once per save when the player enters the Wizard Tower while
/// mouse/keyboard is the active input device.
pub(crate) fn trigger_kbm_menus_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
    active_input: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
) {
    if active_input.is_gamepad() {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::KbmMenusIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Dismisses any active tutorial whose `modality` doesn't match the current
/// input device, so a controller-only or KBM-only walkthrough disappears the
/// moment the player switches inputs. Does NOT mark the tutorial complete —
/// the matching counterpart (or the same tutorial when the player switches
/// back) is still allowed to fire.
pub(crate) fn enforce_tutorial_modality(
    mut commands: Commands,
    active: Option<Res<ActiveTutorial>>,
    active_input: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    let Some(active) = active else { return };
    let modality = active.tutorial.modality();
    let mismatch = match modality {
        TutorialModality::Any => false,
        TutorialModality::MouseKeyboard => active_input.is_gamepad(),
    };
    if !mismatch {
        return;
    }

    remove_all_highlights(&mut commands, &mut highlighted);
    despawn_overlay(&mut commands, &overlay_query);
    if active.paused_gameplay
        && let Some(next_state) = next_in_game_state.as_mut()
    {
        next_state.set(InGameState::Running);
    }
    commands.remove_resource::<ActiveTutorial>();
}

/// Study spell-selected walkthrough. Fires when a Study spell becomes the
/// selected one (detail panel populated), so the +/− and talent walkthrough
/// only shows after the relevant UI is on screen.
pub(crate) fn trigger_study_spell_selected_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
    selected: Res<crate::ui::wizard_tower::SelectedStudySpell>,
) {
    let Some(spell) = selected.0 else {
        return;
    };
    // Only meaningful for a *locked* spell — the +/− and talents don't apply
    // to spells that come unlocked by default (e.g. Magic Missile).
    if crate::ui::wizard_tower::is_spell_unlocked(spell) {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::StudySpellSelectedIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Wizard-select walkthrough. Fires the first time the player opens the
/// "Switch Wizard" panel (RightPanelView::WizardSelect).
pub(crate) fn trigger_wizard_select_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
    view: Res<crate::ui::wizard_tower::RightPanelView>,
) {
    if *view != crate::ui::wizard_tower::RightPanelView::WizardSelect {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::WizardSelectIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Per-tab tutorial trigger. Watches the `WizardTowerTab` resource and fires
/// the matching walkthrough the first time the player opens each tab. Runs
/// only when no tutorial is currently active so it queues nicely behind any
/// in-flight overlay (e.g. controller menus primer).
pub(crate) fn trigger_wizard_tower_tab_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::super::resources::PendingTutorials>>,
    tab: Res<crate::ui::wizard_tower::WizardTowerTab>,
) {
    use crate::ui::wizard_tower::WizardTowerTab;
    let tutorial = match *tab {
        WizardTowerTab::Roguelite => TutorialId::RogueliteTabIntro,
        WizardTowerTab::Endless => TutorialId::EndlessTabIntro,
        WizardTowerTab::Study => TutorialId::StudyTabIntro,
        WizardTowerTab::Multiplayer | WizardTowerTab::Vs => return,
    };
    try_start_tutorial(
        &mut commands,
        tutorial,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}
