use bevy::prelude::*;

use crate::config::save_data::load_unified_save;
use crate::game::units::wizard::components::Spell;

/// Returns all currently unlocked spells from save data.
pub(crate) fn get_unlocked_spells() -> Vec<Spell> {
    let save = load_unified_save();
    let unlocked_names: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    Spell::all()
        .iter()
        .filter(|s| unlocked_names.iter().any(|n| n.as_str() == s.save_key()))
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Which tab is currently active in the wizard tower hub.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum WizardTowerTab {
    #[default]
    Endless,
    Roguelite,
    /// Peer-to-peer connection screen (host/join + connected status).
    Multiplayer,
    /// 1v1 duel setup (wizard pick + ready + start). Disabled until connected.
    Vs,
    Study,
}

impl WizardTowerTab {
    pub fn all() -> &'static [WizardTowerTab] {
        // Study is last so it can be right-justified (separated from the
        // game-mode tabs) by a flex spacer inserted before it in the tab row.
        &[
            WizardTowerTab::Endless,
            WizardTowerTab::Roguelite,
            WizardTowerTab::Vs,
            WizardTowerTab::Multiplayer,
            WizardTowerTab::Study,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            WizardTowerTab::Roguelite => "Roguelite",
            WizardTowerTab::Endless => "Endless",
            WizardTowerTab::Study => "Study",
            WizardTowerTab::Multiplayer => "Multiplayer",
            WizardTowerTab::Vs => "VS",
        }
    }

    /// Whether this tab is disabled at spawn time. The VS tab is gated on a live
    /// connection — `update_tab_active_state` toggles `DisabledTab` on it as the
    /// connection state changes — but it starts disabled (no connection yet).
    pub fn is_disabled(&self) -> bool {
        matches!(self, WizardTowerTab::Vs)
    }
}

/// What the right panel is currently showing.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum RightPanelView {
    /// The default view for the active tab.
    #[default]
    TabContent,
    /// Wizard type selection grid.
    WizardSelect,
}

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker for the left panel container.
#[derive(Component)]
pub(crate) struct WizardTowerLeftPanel;

/// Marker for the right panel container.
#[derive(Component)]
pub(crate) struct WizardTowerRightPanel;

/// Identifies which tab a button corresponds to.
#[derive(Component)]
pub(crate) struct WizardTowerTabButton(pub WizardTowerTab);

/// Marker on the row container holding all top-level tab buttons, so the
/// tutorial system can highlight the whole row at once.
#[derive(Component)]
pub(crate) struct WizardTowerTabRow;

/// Marker for the header "<name> connected" badge (shown while a multiplayer
/// connection is live). Updated by [`update_mp_connected_indicator`].
#[derive(Component)]
pub(crate) struct MpConnectedIndicator;

/// Shows/updates the header multiplayer-connected badge: hidden when
/// disconnected; green "`<steam name>` connected" (or generic "MP connected")
/// when a connection is live.
pub(crate) fn update_mp_connected_indicator(
    connection: Res<crate::networking::resources::NetworkConnection>,
    peer_info: Option<Res<crate::game::multiplayer::coop::CoopPeerInfo>>,
    mut query: Query<(&mut Text, &mut Visibility), With<MpConnectedIndicator>>,
) {
    let connected = connection.state == crate::networking::resources::ConnectionState::Connected;
    let want_vis = if connected {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for (mut text, mut visibility) in &mut query {
        // Write through Bevy change-detection only when the value actually changes,
        // so this every-frame system doesn't dirty the node (and re-layout) each tick.
        if *visibility != want_vis {
            *visibility = want_vis;
        }
        if connected {
            let desired = peer_info
                .as_ref()
                .and_then(|p| p.name.clone())
                .map(|n| format!("{n} connected"))
                .unwrap_or_else(|| "MP connected".to_string());
            if text.0 != desired {
                text.0 = desired;
            }
        }
    }
}

/// Multiplayer lobby + connection state, bundled so `rebuild_panels_on_tab_change`
/// stays under Bevy's 16-parameter system limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MultiplayerPanelData<'w> {
    pub lobby: Res<'w, super::super::super::multiplayer_tab::MultiplayerLobby>,
    pub connection: Res<'w, crate::networking::resources::NetworkConnection>,
    pub steam_client: Option<Res<'w, bevy_steamworks::Client>>,
    pub host_selection: Option<Res<'w, super::super::super::multiplayer_tab::CoopHostSelection>>,
}
