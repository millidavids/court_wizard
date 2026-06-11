use bevy::prelude::*;

use super::layout::{RightPanelView, WizardTowerTab};

pub(super) fn study_tab_active(tab: Option<Res<WizardTowerTab>>) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Study)
}

/// True when the right panel is showing its normal tab content (not the shared
/// wizard-card grid). Shared by the per-tab run-condition helpers below.
pub(super) fn is_tab_content(view: &Option<Res<RightPanelView>>) -> bool {
    view.as_ref()
        .is_some_and(|v| **v == RightPanelView::TabContent)
}

pub(super) fn roguelite_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Roguelite) && is_tab_content(&view)
}

pub(super) fn endless_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    tab.is_some_and(|t| *t == WizardTowerTab::Endless) && is_tab_content(&view)
}

/// True when the host is on a co-op-startable game-mode tab (Endless or Roguelite)
/// showing normal content. Gates the lobby-change rebuild that flips the host's
/// start button "Guest Not Ready" ↔ enabled when the guest readies.
pub(super) fn game_mode_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    tab.is_some_and(|t| matches!(*t, WizardTowerTab::Endless | WizardTowerTab::Roguelite))
        && is_tab_content(&view)
}

/// True only when the Multiplayer tab is showing its normal content — not the
/// shared wizard-card grid. Gates `rebuild_multiplayer_on_lobby_change` so a
/// lobby/connection change can't clobber the open grid.
pub(super) fn multiplayer_tab_active(
    tab: Option<Res<WizardTowerTab>>,
    view: Option<Res<RightPanelView>>,
) -> bool {
    // Both the connection (Multiplayer) tab and the VS tab render lobby-driven
    // content, so a lobby/connection change must rebuild either one.
    tab.is_some_and(|t| matches!(*t, WizardTowerTab::Multiplayer | WizardTowerTab::Vs))
        && is_tab_content(&view)
}
