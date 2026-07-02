use bevy::prelude::*;

use crate::ui::systems::spawn_button;

use super::super::super::components::{OnWizardTowerScreen, WizardTowerButtonAction};
use super::super::super::constants::*;
use super::super::super::materials::{ArcaneRuneData, ArcaneRuneMaterial};
use super::super::decorations::*;
use super::resources::{
    MpConnectedIndicator, RightPanelView, WizardTowerLeftPanel, WizardTowerRightPanel,
    WizardTowerTab, WizardTowerTabButton, WizardTowerTabRow, get_unlocked_spells,
};
use crate::ui::components::ButtonColors;

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
                super::super::super::components::ArcaneRuneBackground,
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
            header_cmd.insert(super::super::super::components::WizardTowerUiContent);
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
            content_cmd.insert(super::super::super::components::WizardTowerUiContent);
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
                                        tab_btn.insert(crate::ui::focus::DisabledTab);
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
