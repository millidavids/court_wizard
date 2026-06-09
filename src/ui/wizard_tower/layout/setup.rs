//! Wizard tower layout: panel setup, tab clicks, panel rebuilds.

use super::decorations::*;
use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::config::save_data::load_unified_save;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::KillStats;
use crate::game::units::wizard::components::Spell;
use crate::state::AppState;
use crate::ui::components::{ButtonActive, ButtonColors, ButtonFront};
use crate::ui::systems::spawn_button;

use super::super::components::{OnWizardTowerScreen, WizardTowerButtonAction};
use super::super::constants::*;
use super::super::materials::{ArcaneRuneData, ArcaneRuneMaterial};

/// Returns all currently unlocked spells from save data.
pub(super) fn get_unlocked_spells() -> Vec<Spell> {
    let save = load_unified_save();
    let unlocked_names: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    Spell::all()
        .iter()
        .filter(|s| unlocked_names.contains(&format!("{:?}", s)))
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

/// Marker for tabs that are disabled and cannot be clicked.
#[derive(Component)]
pub(crate) struct DisabledTab;

// ---------------------------------------------------------------------------
// Layout setup
// ---------------------------------------------------------------------------

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

/// Spawns the full wizard tower tabbed layout. Tab row:
/// `[Endless] [Roguelite] [VS] [Multiplayer] ........... [Study]   [<- Back]`
/// (Study is right-justified; VS is disabled until a connection is live).
#[allow(clippy::too_many_arguments)]
pub(crate) fn setup_wizard_tower_layout(
    mut commands: Commands,
    mut config: ResMut<crate::config::GameConfig>,
    mut active_save: ResMut<crate::config::ActiveSave>,
    existing_tab: Option<Res<WizardTowerTab>>,
    existing_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
    asset_server: Res<AssetServer>,
    mut rune_materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    // Preserve tab if already set (e.g. returning from a level sets the
    // appropriate tab before transitioning to MetaGame). Otherwise default.
    if existing_tab.is_none() {
        commands.insert_resource(WizardTowerTab::default());
    }
    commands.insert_resource(RightPanelView::default());

    if active_save.0.is_none() {
        crate::config::save_data::load_or_create_wizard(
            config.wizard_type,
            &mut config,
            &mut active_save,
        );
    }

    // Restore dormant roguelite run from disk if no in-memory run exists.
    // This allows resuming a run after closing the game or switching modes.
    if existing_run.is_none()
        && let Some(saved_run) = crate::config::save_data::load_current_roguelite_run(&active_save)
    {
        // Restore game mode and run state resources
        commands.insert_resource(crate::game::game_mode::components::GameMode::Roguelite);
        commands.insert_resource(crate::game::game_mode::components::RogueliteRunState {
            started_at: saved_run.started_at,
            level_stats: saved_run.level_stats,
        });
        if let Some(mods) = saved_run.modifiers {
            commands.insert_resource(mods);
        }
        let toggles =
            crate::game::game_mode::components::ActiveToggles::from_ids(&saved_run.active_toggles);
        commands.insert_resource(toggles);
        if let Some(seed) = saved_run.seed {
            commands.insert_resource(crate::game::seeded_rng::resources::GameSeed(seed));
        }
        // Ensure config reflects the run's state
        config.current_level = saved_run.current_level;
        config.wizard_type = saved_run.wizard_type;
        // Auto-select the roguelite tab
        commands.insert_resource(WizardTowerTab::Roguelite);
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
        ))
        .with_children(|root| {
            let unlocked_count = get_unlocked_spells().len() as f32;
            let rune_mat = rune_materials.add(ArcaneRuneMaterial {
                data: ArcaneRuneData {
                    color: RUNE_COLOR.to_linear(),
                    time: 0.0,
                    opacity: RUNE_OPACITY,
                    unlocked_count,
                    _padding: 0.0,
                },
            });
            root.spawn((
                MaterialNode(rune_mat),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
                super::super::components::ArcaneRuneBackground,
            ));

            // Orbiting spell name text around the rune circles
            spawn_arcane_rune_text(root, &asset_server);

            // Header row: title left, Back button right
            let mut header_cmd = root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                },
                ZIndex(10),
            ));
            #[cfg(debug_assertions)]
            header_cmd.insert(super::super::components::WizardTowerUiContent);
            header_cmd.with_children(|header| {
                crate::ui::systems::spawn_title_with_shadow(
                    header,
                    "Wizard Tower",
                    TITLE_FONT_SIZE,
                    TITLE_COLOR,
                    Node::default(),
                );
                // Small "<name> connected" badge — shown (green) while a
                // multiplayer connection is live so the host can confirm the
                // partner is present without opening the Multiplayer tab.
                header.spawn((
                    Text::new(""),
                    TextFont::from_font_size(16.0),
                    TextColor(crate::ui::constants::SUCCESS_COLOR),
                    Visibility::Hidden,
                    Node {
                        margin: UiRect::left(Val::Px(16.0)),
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    MpConnectedIndicator,
                ));
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                spawn_button(
                    header,
                    "Back",
                    (
                        WizardTowerButtonAction::ReturnToMenu,
                        crate::ui::focus::NoGamepadFocus,
                    ),
                    &BACK_BUTTON_STYLE,
                );
            });

            // Content area: left panel + right column (tabs + right panel)
            let mut content_cmd = root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            });
            #[cfg(debug_assertions)]
            content_cmd.insert(super::super::components::WizardTowerUiContent);
            content_cmd.with_children(|content| {
                // GlobalZIndex(1) keeps the panel (and descendants) above the
                // rune backdrop in a stable stacking context, even across
                // child despawn/respawn cycles.
                content.spawn((
                    Node {
                        width: Val::Percent(LEFT_PANEL_PERCENT),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(SECTION_PADDING)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        overflow: Overflow::scroll_y(),
                        flex_shrink: 0.0,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ScrollPosition::default(),
                    crate::ui::focus::GamepadScrollTarget,
                    Interaction::None,
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    GlobalZIndex(1),
                    WizardTowerLeftPanel,
                ));

                // Right column: tab bar + right panel
                content
                    .spawn(Node {
                        width: Val::Percent(RIGHT_PANEL_PERCENT),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(MARGIN_SMALL),
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|right_col| {
                        // Tab bar inside the right column
                        right_col
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    border: UiRect::all(Val::Px(0.0)),
                                    ..default()
                                },
                                WizardTowerTabRow,
                            ))
                            .with_children(|tab_row| {
                                let initial_tab =
                                    existing_tab.as_deref().copied().unwrap_or_default();
                                for tab in WizardTowerTab::all() {
                                    // Push Study to the right edge of the row — it
                                    // isn't a game-mode tab, so it gets visual
                                    // separation from Endless/Roguelite/Multiplayer.
                                    if *tab == WizardTowerTab::Study {
                                        tab_row.spawn(Node {
                                            flex_grow: 1.0,
                                            ..default()
                                        });
                                    }

                                    let is_active = *tab == initial_tab;
                                    let is_disabled = tab.is_disabled();

                                    let (bg, border) = if is_active {
                                        (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                                    } else {
                                        (INACTIVE_TAB_BG, TAB_BORDER)
                                    };

                                    let text_color = if is_disabled {
                                        DISABLED_TAB_TEXT
                                    } else {
                                        TEXT_COLOR
                                    };

                                    let mut tab_btn = tab_row.spawn((
                                        Button,
                                        Node {
                                            height: Val::Px(TAB_HEIGHT),
                                            padding: UiRect::horizontal(Val::Px(TAB_PADDING_H)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderColor::all(border),
                                        ButtonColors {
                                            background: bg,
                                            border,
                                        },
                                        WizardTowerTabButton(*tab),
                                        crate::ui::focus::Focusable,
                                        crate::ui::focus::TabFocusable,
                                    ));

                                    if is_disabled {
                                        tab_btn.insert(DisabledTab);
                                    }

                                    tab_btn.with_children(|btn| {
                                        btn.spawn((
                                            Text::new(tab.label()),
                                            TextFont::from_font_size(TAB_FONT_SIZE),
                                            TextColor(text_color),
                                        ));
                                    });
                                }
                            });

                        // Right panel — scrollable content below tabs.
                        // GlobalZIndex(1) — see WizardTowerLeftPanel.
                        right_col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                border: UiRect::all(Val::Px(1.0)),
                                overflow: Overflow::scroll_y(),
                                flex_grow: 1.0,
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            ScrollPosition::default(),
                            crate::ui::focus::GamepadScrollTarget,
                            Interaction::None,
                            BackgroundColor(SECTION_BG),
                            BorderColor::all(DETAIL_BORDER),
                            GlobalZIndex(1),
                            WizardTowerRightPanel,
                        ));
                    });
            });
        });
}

// ---------------------------------------------------------------------------
// Tab click handling
// ---------------------------------------------------------------------------

/// Updates the active tab when a tab button is clicked. Disabled tabs are ignored.
pub(crate) fn handle_tab_click(
    mut button_clicked: MessageReader<MouseClicked>,
    tab_query: Query<&WizardTowerTabButton, Without<DisabledTab>>,
    mut tab_resource: ResMut<WizardTowerTab>,
    mut right_panel_view: ResMut<RightPanelView>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && *tab_resource != tab_btn.0
        {
            *tab_resource = tab_btn.0;
            // Reset to default tab content view when switching tabs
            *right_panel_view = RightPanelView::TabContent;
        }
    }
}

// ---------------------------------------------------------------------------
// Back button handling
// ---------------------------------------------------------------------------

/// Handles the back button to return to the main menu.
pub(crate) fn handle_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(WizardTowerButtonAction::ReturnToMenu) = button_query.get(event.button) {
            return_to_main_menu(
                &mut next_app_state,
                &mut kill_stats,
                &mut active_save,
                &mut channel_change,
            );
        }
    }
}

/// Handles Escape key / gamepad back to return to the main menu from the wizard tower.
#[allow(clippy::too_many_arguments)]
pub(crate) fn escape_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<crate::game::input::gamepad::messages::MenuBackPressed>,
    tab: Option<Res<WizardTowerTab>>,
    selected_spell: Option<Res<super::super::components::SelectedStudySpell>>,
    selected_bonus: Option<Res<super::super::components::SelectedInsightBonus>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if !keyboard.just_pressed(KeyCode::Escape) && !back_pressed {
        return;
    }
    // On Study with a selection, `study_back_to_cursor` consumes the back
    // to deselect; swallow it here so we don't also exit to main menu.
    let study_consumed = tab.as_deref().is_some_and(|t| *t == WizardTowerTab::Study)
        && (selected_spell.as_deref().is_some_and(|s| s.0.is_some())
            || selected_bonus.as_deref().is_some_and(|s| s.0.is_some()));
    if study_consumed {
        return;
    }
    return_to_main_menu(
        &mut next_app_state,
        &mut kill_stats,
        &mut active_save,
        &mut channel_change,
    );
}

fn return_to_main_menu(
    next_app_state: &mut ResMut<NextState<AppState>>,
    kill_stats: &mut ResMut<KillStats>,
    active_save: &mut ResMut<ActiveSave>,
    channel_change: &mut MessageWriter<ChannelChangeMessage>,
) {
    channel_change.write(ChannelChangeMessage);
    kill_stats.reset();
    active_save.0 = None;
    next_app_state.set(AppState::MainMenu);
}

// ---------------------------------------------------------------------------
// Panel rebuild on tab change
// ---------------------------------------------------------------------------

/// Multiplayer lobby + connection state, bundled so `rebuild_panels_on_tab_change`
/// stays under Bevy's 16-parameter system limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MultiplayerPanelData<'w> {
    lobby: Res<'w, super::super::multiplayer_tab::MultiplayerLobby>,
    connection: Res<'w, crate::networking::resources::NetworkConnection>,
    steam_client: Option<Res<'w, bevy_steamworks::Client>>,
    host_selection: Option<Res<'w, super::super::multiplayer_tab::CoopHostSelection>>,
}

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
    pending_toggles: Option<Res<super::super::roguelite_tab::PendingToggles>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    battle_insight: Res<crate::game::resources::BattleInsightData>,
    asset_server: Res<AssetServer>,
    mut progress_materials: ResMut<Assets<super::super::materials::RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<super::super::materials::ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<super::super::materials::StarSkyMaterial>>,
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
        commands.init_resource::<super::super::wizard_cards::SelectedWizard>();
        // Hide the (MP-unsupported) Psychopath card when switching wizards from
        // the Multiplayer tab; SP tabs still show it.
        // Wizard selection for multiplayer happens on the VS tab now; filter the
        // grid to MP-supported wizards there (and on the connection tab, which
        // can still open the grid). SP tabs show the full roster.
        let exclude_mp_unsupported =
            matches!(*tab, WizardTowerTab::Multiplayer | WizardTowerTab::Vs);
        super::super::wizard_cards::build_wizard_card_grid(
            &mut commands,
            right_entity,
            exclude_mp_unsupported,
        );
        return;
    }

    // Whether a co-op guest is connected and (if so) whether they're ready — gates
    // the host's start buttons ("Guest Not Ready" until the guest readies). `None`
    // when no guest is present, so solo behaviour is unchanged.
    let guest_pending = compute_guest_pending(&multiplayer.connection, &multiplayer.lobby);

    match *tab {
        WizardTowerTab::Roguelite => {
            if let Some(ref run_state) = roguelite_run {
                // Active run: show run info + continue/end buttons
                super::super::roguelite_tab::build_roguelite_active_run_right_panel(
                    &mut commands,
                    right_entity,
                    guest_pending,
                );
                super::super::roguelite_tab::build_roguelite_active_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    run_state,
                );
            } else {
                // No active run: init resources and show modifier UI
                super::super::roguelite_tab::init_roguelite_tab_resources(
                    &mut commands,
                    &mut config,
                    roguelite_modifiers.as_deref(),
                    pending_toggles.as_deref(),
                    active_toggles.as_deref(),
                );
                let seed_text = config.seed.map(|s| s.to_string()).unwrap_or_default();
                let mods = roguelite_modifiers.as_deref().cloned().unwrap_or_default();
                let pt = pending_toggles.as_deref().cloned().unwrap_or_default();
                super::super::roguelite_tab::build_roguelite_no_run_right_panel(
                    &mut commands,
                    right_entity,
                    &mods,
                    &pt,
                    &seed_text,
                    guest_pending,
                );
                super::super::roguelite_tab::build_roguelite_no_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    &mods,
                    &pt,
                );
            }
        }
        WizardTowerTab::Endless => {
            super::super::endless_tab::build_endless_right_panel(
                &mut commands,
                right_entity,
                &config,
                guest_pending,
            );
            super::super::endless_tab::build_endless_left_panel(
                &mut commands,
                left_entity,
                config.wizard_type,
                &config,
            );
        }
        WizardTowerTab::Study => {
            super::super::study_tab::build_study_panels(
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
            super::super::multiplayer_tab::panels::build_multiplayer_panels(
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
            super::super::multiplayer_tab::panels::build_multiplayer_panels(
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

/// Whether a co-op guest is connected (and ready) — for gating the host's mode
/// start buttons. `None` = no guest present → normal solo behaviour;
/// `Some(false)` = guest connected but not ready → show "Guest Not Ready";
/// `Some(true)` = guest ready (and has picked a wizard) → enable co-op start.
fn compute_guest_pending(
    connection: &crate::networking::resources::NetworkConnection,
    lobby: &super::super::multiplayer_tab::MultiplayerLobby,
) -> Option<bool> {
    use super::super::multiplayer_tab::state::LobbyPhase;
    use crate::networking::resources::{ConnectionState, PeerRole};
    if connection.state != ConnectionState::Connected || connection.role != Some(PeerRole::Host) {
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

// ---------------------------------------------------------------------------
// Tab visual state
// ---------------------------------------------------------------------------

/// Updates tab button visuals to reflect the currently active tab.
pub(crate) fn update_tab_active_state(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    connection: Res<crate::networking::resources::NetworkConnection>,
    tab_buttons: Query<(
        Entity,
        &WizardTowerTabButton,
        &Children,
        Has<DisabledTab>,
        Has<ButtonActive>,
        Has<crate::ui::focus::GamepadFocused>,
    )>,
    mut button_colors: Query<&mut ButtonColors>,
    mut front_q: Query<
        (&mut BackgroundColor, &Children),
        (With<ButtonFront>, Without<ButtonColors>),
    >,
    mut tab_text: Query<&mut TextColor>,
) {
    use crate::networking::resources::{ConnectionState, PeerRole};
    let connected = connection.state == ConnectionState::Connected;
    // A connected GUEST is locked to the Multiplayer (+ Study) screen: it does
    // everything from there, and locking the mode tabs stops it accidentally
    // starting a solo game. The HOST drives the mode tabs as normal.
    let is_guest_connected = connected && connection.role == Some(PeerRole::Guest);

    for (entity, tab_btn, children, is_disabled, has_active, focused) in &tab_buttons {
        // One coherent desired-enabled state per tab.
        let desired_enabled = match tab_btn.0 {
            // VS needs a connection, and is hidden from a connected guest.
            WizardTowerTab::Vs => connected && !is_guest_connected,
            // Mode-start tabs lock for a connected guest.
            WizardTowerTab::Endless | WizardTowerTab::Roguelite => !is_guest_connected,
            // The guest's home + Study are always reachable.
            WizardTowerTab::Multiplayer | WizardTowerTab::Study => true,
        };
        if desired_enabled && is_disabled {
            commands.entity(entity).remove::<DisabledTab>();
        } else if !desired_enabled && !is_disabled {
            commands.entity(entity).insert(DisabledTab);
        }

        let is_active = tab_btn.0 == *tab;
        // Keep the `ButtonActive` marker in sync (queried by other systems).
        if is_active && !has_active {
            commands.entity(entity).insert(ButtonActive);
        } else if !is_active && has_active {
            commands.entity(entity).remove::<ButtonActive>();
        }

        // Desired visuals, priority: disabled (greyed) > active > inactive. A
        // disabled tab greys its background too, not just its label, so it clearly
        // reads as unavailable; an enabled tab (e.g. VS once a host connects) gets
        // the normal active/inactive styling.
        let (bg, border, label_color) = if !desired_enabled {
            (DISABLED_TAB_BG, DISABLED_TAB_BORDER, DISABLED_TAB_TEXT)
        } else if is_active {
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, TEXT_COLOR)
        } else {
            (INACTIVE_TAB_BG, TAB_BORDER, TEXT_COLOR)
        };

        // Update the wrapper's `ButtonColors` — the source the 3D-button systems
        // (`sync_front_face_colors`) read to repaint the visible front face.
        if let Ok(mut colors) = button_colors.get_mut(entity) {
            if colors.background != bg {
                colors.background = bg;
            }
            if colors.border != border {
                colors.border = border;
            }
        }

        // The visible surface is the 3D `ButtonFront` child, and the label is
        // reparented into it. Paint the front-face background directly so it updates
        // immediately and for the active tab (which `sync_front_face_colors` skips) —
        // except when gamepad-focused, where the focus tint owns the background — and
        // recolour the (now nested) label text. All writes are change-guarded.
        let want_bg = crate::ui::systems::opaque(bg);
        for child in children.iter() {
            // Pre-3D-conversion frame: the text is still a direct child.
            if let Ok(mut text_color) = tab_text.get_mut(child)
                && text_color.0 != label_color
            {
                *text_color = TextColor(label_color);
            }
            if let Ok((mut front_bg, front_children)) = front_q.get_mut(child) {
                if !focused && front_bg.0 != want_bg {
                    *front_bg = BackgroundColor(want_bg);
                }
                for grandchild in front_children.iter() {
                    if let Ok(mut text_color) = tab_text.get_mut(grandchild)
                        && text_color.0 != label_color
                    {
                        *text_color = TextColor(label_color);
                    }
                }
            }
        }
    }
}
