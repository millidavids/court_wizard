use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::KillStats;
use crate::state::AppState;
use crate::ui::components::{ButtonActive, ButtonColors, ButtonFront};
use crate::ui::systems::spawn_button;

use super::components::{OnWizardTowerScreen, WizardTowerButtonAction};
use super::constants::*;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Which tab is currently active in the wizard tower hub.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum WizardTowerTab {
    #[default]
    Roguelite,
    Endless,
    Study,
    Multiplayer,
}

impl WizardTowerTab {
    pub fn all() -> &'static [WizardTowerTab] {
        &[
            WizardTowerTab::Roguelite,
            WizardTowerTab::Endless,
            WizardTowerTab::Study,
            WizardTowerTab::Multiplayer,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            WizardTowerTab::Roguelite => "Roguelite",
            WizardTowerTab::Endless => "Endless",
            WizardTowerTab::Study => "Study",
            WizardTowerTab::Multiplayer => "Multiplayer",
        }
    }

    /// Whether this tab is disabled and cannot be clicked.
    pub fn is_disabled(&self) -> bool {
        matches!(self, WizardTowerTab::Multiplayer)
    }
}

/// What the right panel is currently showing.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum RightPanelView {
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
pub(super) struct WizardTowerLeftPanel;

/// Marker for the right panel container.
#[derive(Component)]
pub(super) struct WizardTowerRightPanel;

/// Identifies which tab a button corresponds to.
#[derive(Component)]
pub(super) struct WizardTowerTabButton(pub WizardTowerTab);

/// Marker for tabs that are disabled and cannot be clicked.
#[derive(Component)]
pub(super) struct DisabledTab;

// ---------------------------------------------------------------------------
// Layout setup
// ---------------------------------------------------------------------------

/// Spawns the full wizard tower tabbed layout.
///
/// ```text
/// +---------------------------------------------------------------+
/// | [Roguelite] [Endless] [Study] [Multiplayer*]       [<- Back]  |
/// +--------------------+------------------------------------------+
/// |                    |                                            |
/// |   Left Panel       |       Right Panel                         |
/// |   (33% width)      |       (67% width)                         |
/// |                    |                                            |
/// +--------------------+------------------------------------------+
/// ```
pub(super) fn setup_wizard_tower_layout(
    mut commands: Commands,
    mut config: ResMut<crate::config::GameConfig>,
    mut active_save: ResMut<crate::config::ActiveSave>,
    existing_tab: Option<Res<WizardTowerTab>>,
    existing_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
) {
    // Preserve tab if already set (e.g. returning from a level sets the
    // appropriate tab before transitioning to MetaGame). Otherwise default.
    if existing_tab.is_none() {
        commands.insert_resource(WizardTowerTab::default());
    }
    commands.insert_resource(RightPanelView::default());

    // Ensure a wizard save is loaded so config has valid data (level, etc.).
    // When entering from Landing with no active run, ActiveSave is None.
    if active_save.0.is_none() {
        crate::config::save_data::load_wizard_type_into_config(
            config.wizard_type,
            &mut config,
            &mut active_save,
        );
    }

    // Restore dormant roguelite run from disk if no in-memory run exists.
    // This allows resuming a run after closing the game or switching modes.
    if existing_run.is_none()
        && let Some(saved_run) =
            crate::config::save_data::load_current_roguelite_run(&active_save)
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
        let toggles = crate::game::game_mode::components::ActiveToggles::from_ids(
            &saved_run.active_toggles,
        );
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
            // Header row: title left, Back button right
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                },
                ZIndex(10),
            ))
            .with_children(|header| {
                crate::ui::systems::spawn_title_with_shadow(
                    header,
                    "Wizard Tower",
                    TITLE_FONT_SIZE,
                    TITLE_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                spawn_button(
                    header,
                    "Back",
                    WizardTowerButtonAction::ReturnToMenu,
                    &BACK_BUTTON_STYLE,
                );
            });

            // Content area: left panel + right column (tabs + right panel)
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            })
            .with_children(|content| {
                // Left panel — ZIndex ensures it renders above graph edges
                // that may visually leak due to UiTransform rotation bypassing clip.
                content.spawn((
                    Node {
                        width: Val::Percent(LEFT_PANEL_PERCENT),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(SECTION_PADDING)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        overflow: Overflow::scroll_y(),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ScrollPosition::default(),
                    Interaction::None,
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    BorderRadius::all(Val::Px(6.0)),
                    ZIndex(10),
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
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(4.0),
                                ..default()
                            })
                            .with_children(|tab_row| {
                                let initial_tab = existing_tab.as_deref().copied().unwrap_or_default();
                                for tab in WizardTowerTab::all() {
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
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderColor::all(border),
                                        BorderRadius::all(Val::Px(4.0)),
                                        ButtonColors {
                                            background: bg,
                                            border,
                                        },
                                        WizardTowerTabButton(*tab),
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

                        // Right panel — scrollable content below tabs
                        right_col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                border: UiRect::all(Val::Px(1.0)),
                                overflow: Overflow::scroll_y(),
                                flex_grow: 1.0,
                                ..default()
                            },
                            ScrollPosition::default(),
                            Interaction::None,
                            BackgroundColor(SECTION_BG),
                            BorderColor::all(DETAIL_BORDER),
                            BorderRadius::all(Val::Px(6.0)),
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
pub(super) fn handle_tab_click(
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
pub(super) fn handle_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(WizardTowerButtonAction::ReturnToMenu) = button_query.get(event.button) {
            channel_change.write(ChannelChangeMessage);
            kill_stats.reset();
            active_save.0 = None;
            next_app_state.set(AppState::MainMenu);
        }
    }
}

// ---------------------------------------------------------------------------
// Panel rebuild on tab change
// ---------------------------------------------------------------------------

/// When the active tab changes, despawn children of both panels and rebuild
/// with the appropriate tab's content.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild_panels_on_tab_change(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    right_panel_view: Res<RightPanelView>,
    left_panel: Query<Entity, With<WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<WizardTowerRightPanel>>,
    mut config: ResMut<crate::config::GameConfig>,
    roguelite_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    pending_toggles: Option<Res<super::roguelite_tab::PendingToggles>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    battle_insight: Res<crate::game::resources::BattleInsightData>,
    asset_server: Res<AssetServer>,
    mut progress_materials: ResMut<Assets<super::materials::RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<super::materials::ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<super::materials::StarSkyMaterial>>,
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
        commands.init_resource::<super::wizard_cards::ExpandedWizard>();
        commands.init_resource::<super::wizard_cards::SelectedWizard>();
        super::wizard_cards::build_wizard_card_grid(&mut commands, right_entity);
        return;
    }

    match *tab {
        WizardTowerTab::Roguelite => {
            if let Some(ref run_state) = roguelite_run {
                // Active run: show run info + continue/end buttons
                super::roguelite_tab::build_roguelite_active_run_right_panel(
                    &mut commands,
                    right_entity,
                );
                super::roguelite_tab::build_roguelite_active_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    run_state,
                );
            } else {
                // No active run: init resources and show modifier UI
                super::roguelite_tab::init_roguelite_tab_resources(
                    &mut commands,
                    &mut config,
                    roguelite_modifiers.as_deref(),
                    pending_toggles.as_deref(),
                    active_toggles.as_deref(),
                );
                let seed_text = config
                    .seed
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let mods = roguelite_modifiers
                    .as_deref()
                    .cloned()
                    .unwrap_or_default();
                let pt = pending_toggles
                    .as_deref()
                    .cloned()
                    .unwrap_or_default();
                super::roguelite_tab::build_roguelite_no_run_right_panel(
                    &mut commands,
                    right_entity,
                    &mods,
                    &pt,
                    &seed_text,
                );
                super::roguelite_tab::build_roguelite_no_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    &mods,
                    &pt,
                );
            }
        }
        WizardTowerTab::Endless => {
            super::endless_tab::build_endless_right_panel(
                &mut commands,
                right_entity,
                &config,
            );
            super::endless_tab::build_endless_left_panel(
                &mut commands,
                left_entity,
                config.wizard_type,
                &config,
            );
        }
        WizardTowerTab::Study => {
            super::study_tab::build_study_panels(
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
            // Disabled tab - should not be reachable
        }
    }
}

// ---------------------------------------------------------------------------
// Tab visual state
// ---------------------------------------------------------------------------

/// Updates tab button visuals to reflect the currently active tab.
pub(super) fn update_tab_active_state(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    tab_buttons: Query<(Entity, &WizardTowerTabButton, &Children, Has<DisabledTab>, Has<ButtonActive>)>,
    mut tab_bg: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
    mut front_bg: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<ButtonFront>, Without<ButtonColors>),
    >,
) {
    for (entity, tab_btn, children, _is_disabled, has_active) in &tab_buttons {
        let is_active = tab_btn.0 == *tab;

        // Skip if the active/inactive state already matches
        if is_active == has_active && !tab.is_changed() {
            continue;
        }

        let (bg, border) = if is_active {
            commands.entity(entity).insert(ButtonActive);
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            commands.entity(entity).remove::<ButtonActive>();
            (INACTIVE_TAB_BG, TAB_BORDER)
        };

        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_bg.get_mut(entity) {
            *bg_color = bg.into();
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }

        // Update the 3D front face child
        for child in children.iter() {
            if let Ok((mut front_bg_color, mut front_border_color)) = front_bg.get_mut(child) {
                *front_bg_color = crate::ui::systems::opaque(bg).into();
                *front_border_color = BorderColor::all(border);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

/// Removes wizard tower tab resources when exiting the meta-game.
pub(super) fn cleanup_wizard_tower_tab_resources(mut commands: Commands) {
    commands.remove_resource::<WizardTowerTab>();
    commands.remove_resource::<RightPanelView>();
}
