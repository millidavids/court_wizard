use bevy::prelude::*;

use super::resources::{
    MultiplayerPanelData, RightPanelView, WizardTowerLeftPanel, WizardTowerRightPanel,
    WizardTowerTab,
};

/// When the active tab changes, despawn children of both panels and rebuild
/// with the appropriate tab's content.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rebuild_panels_on_tab_change(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    right_panel_view: Res<RightPanelView>,
    left_panel: Query<Entity, With<WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<WizardTowerRightPanel>>,
    mut config: ResMut<crate::config::GameConfig>,
    roguelite_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    pending_toggles: Option<Res<super::super::super::roguelite_tab::PendingToggles>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    battle_insight: Res<crate::game::resources::BattleInsightData>,
    asset_server: Res<AssetServer>,
    mut progress_materials: ResMut<Assets<super::super::super::materials::RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<super::super::super::materials::ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<super::super::super::materials::StarSkyMaterial>>,
    multiplayer: MultiplayerPanelData,
) {
    // The run_if condition already gates this system to only run when
    // tab, right_panel_view, or RogueliteRunState changes/removes.
    // No additional early-return needed.

    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };

    // Despawn children of both panels
    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();

    // If showing wizard cards, build the card grid instead of tab content
    if *right_panel_view == RightPanelView::WizardSelect {
        commands.init_resource::<super::super::super::wizard_cards::SelectedWizard>();
        // Hide the (MP-unsupported) Psychopath card when switching wizards from
        // the Multiplayer tab; SP tabs still show it.
        // Wizard selection for multiplayer happens on the VS tab now; filter the
        // grid to MP-supported wizards there (and on the connection tab, which
        // can still open the grid). SP tabs show the full roster.
        let exclude_mp_unsupported =
            matches!(*tab, WizardTowerTab::Multiplayer | WizardTowerTab::Vs);
        super::super::super::wizard_cards::build_wizard_card_grid(
            &mut commands,
            right_entity,
            exclude_mp_unsupported,
        );
        return;
    }

    // Whether a co-op guest is connected (and ready) — for gating the host's mode
    // start buttons. `None` = no guest present → normal solo behaviour;
    // `Some(false)` = guest connected but not ready → show "Guest Not Ready";
    // `Some(true)` = guest ready (and has picked a wizard) → enable co-op start.
    let guest_pending = compute_guest_pending(&multiplayer.connection, &multiplayer.lobby);

    match *tab {
        WizardTowerTab::Roguelite => {
            if let Some(ref run_state) = roguelite_run {
                // Active run: show run info + continue/end buttons
                super::super::super::roguelite_tab::build_roguelite_active_run_right_panel(
                    &mut commands,
                    right_entity,
                    guest_pending,
                );
                super::super::super::roguelite_tab::build_roguelite_active_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    run_state,
                );
            } else {
                // No active run: init resources and show modifier UI
                super::super::super::roguelite_tab::init_roguelite_tab_resources(
                    &mut commands,
                    &mut config,
                    roguelite_modifiers.as_deref(),
                    pending_toggles.as_deref(),
                    active_toggles.as_deref(),
                );
                let seed_text = config.seed.map(|s| s.to_string()).unwrap_or_default();
                let mods = roguelite_modifiers.as_deref().cloned().unwrap_or_default();
                let pt = pending_toggles.as_deref().cloned().unwrap_or_default();
                super::super::super::roguelite_tab::build_roguelite_no_run_right_panel(
                    &mut commands,
                    right_entity,
                    &mods,
                    &pt,
                    &seed_text,
                    guest_pending,
                );
                super::super::super::roguelite_tab::build_roguelite_no_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    &mods,
                    &pt,
                );
            }
        }
        WizardTowerTab::Endless => {
            super::super::super::endless_tab::build_endless_right_panel(
                &mut commands,
                right_entity,
                &config,
                guest_pending,
            );
            super::super::super::endless_tab::build_endless_left_panel(
                &mut commands,
                left_entity,
                config.wizard_type,
                &config,
            );
        }
        WizardTowerTab::Study => {
            super::super::super::study_tab::build_study_panels(
                &mut commands,
                right_entity,
                left_entity,
                &battle_insight,
                &asset_server,
                &mut progress_materials,
                &mut ring_materials,
                &mut star_sky_materials,
            );
        }
        WizardTowerTab::Multiplayer => {
            // Connection screen only (host/join + connected status). The versus
            // duel setup lives on the VS tab. Mid-tab lobby changes are handled
            // by `rebuild_multiplayer_on_lobby_change` in plugin.rs.
            super::super::super::multiplayer_tab::panels::build_multiplayer_panels(
                &mut commands,
                left_entity,
                right_entity,
                &multiplayer.lobby,
                &multiplayer.connection,
                multiplayer.steam_client.is_some(),
                false, // connection tab
                multiplayer.host_selection.as_deref(),
            );
        }
        WizardTowerTab::Vs => {
            // 1v1 duel setup: wizard pick + ready + start (only meaningful once
            // connected; otherwise it shows a "connect first" hint).
            super::super::super::multiplayer_tab::panels::build_multiplayer_panels(
                &mut commands,
                left_entity,
                right_entity,
                &multiplayer.lobby,
                &multiplayer.connection,
                multiplayer.steam_client.is_some(),
                true, // VS tab
                multiplayer.host_selection.as_deref(),
            );
        }
    }
}

fn compute_guest_pending(
    connection: &crate::networking::resources::NetworkConnection,
    lobby: &super::super::super::multiplayer_tab::MultiplayerLobby,
) -> Option<bool> {
    use super::super::super::multiplayer_tab::state::LobbyPhase;
    if !connection.has_connected_guest() {
        return None;
    }
    match &lobby.phase {
        LobbyPhase::WizardSelect {
            opponent_ready,
            opponent_wizard: Some(_),
            ..
        } => Some(*opponent_ready),
        // A guest is connected but hasn't sent its wizard pick yet (the brief
        // window right after connecting). Treat it as "present, not ready" so the
        // host's start button shows "Guest Not Ready" instead of a live solo-start
        // button — otherwise a click in that window starts a solo game and strands
        // the guest. (`opponent_ready` can never be true before `opponent_wizard`
        // is `Some`, so the enabled path is unaffected.)
        LobbyPhase::WizardSelect { .. } => Some(false),
        _ => None,
    }
}
