use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;

use super::layout::{WizardTowerLeftPanel, WizardTowerRightPanel, WizardTowerTab};
use super::multiplayer_tab::{CoopHostSelection, MultiplayerLobby};

// ---------------------------------------------------------------------------
// Multiplayer panel rebuild (driven by lobby/connection change)
// ---------------------------------------------------------------------------

/// Rebuilds the multiplayer left+right panels when `MultiplayerLobby` or
/// `NetworkConnection` changes while the Multiplayer tab is active.
///
/// This is a sibling to `rebuild_panels_on_tab_change` — that system fires
/// on tab-switch; this one fires when the lobby phase advances mid-tab.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild_multiplayer_on_lobby_change(
    mut commands: Commands,
    left_panel: Query<Entity, With<WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<WizardTowerRightPanel>>,
    lobby: Res<MultiplayerLobby>,
    connection: Res<NetworkConnection>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    tab: Option<Res<WizardTowerTab>>,
    host_selection: Option<Res<CoopHostSelection>>,
) {
    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };

    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();

    let for_vs_tab = tab.is_some_and(|t| *t == WizardTowerTab::Vs);
    super::multiplayer_tab::panels::build_multiplayer_panels(
        &mut commands,
        left_entity,
        right_entity,
        &lobby,
        &connection,
        steam_client.is_some(),
        for_vs_tab,
        host_selection.as_deref(),
    );
}
