use bevy::prelude::*;

use crate::ui::systems::{spawn_button, spawn_title_with_shadow};

use super::super::components::*;
use super::super::constants::*;

fn setup(mut commands: Commands, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    commands.insert_resource(CompendiumState::default());

    let content = spawn_page_container(
        &mut commands,
        OnCompendiumScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );

    commands.entity(content).with_children(|parent| {
        // Header row: title left, Back button right
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                ..default()
            })
            .with_children(|header| {
                spawn_title_with_shadow(
                    header,
                    "Compendium",
                    TITLE_FONT_SIZE,
                    TEXT_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                spawn_button(
                    header,
                    "Back",
                    (BackButton, crate::ui::focus::NoGamepadFocus),
                    &crate::ui::main_menu::BACK_BUTTON_STYLE,
                );
            });

        // Main content: left detail + right tabbed panel
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            })
            .with_children(|main| {
                // Left detail panel
                spawn_detail_panel(main);

                // Right panel: tabs + content
                spawn_right_panel(main);
            });
    });
}

pub(super) fn spawn_detail_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
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
            BackgroundColor(DETAIL_BG),
            BorderColor::all(DETAIL_BORDER),
            DetailPanel,
            crate::ui::focus::GamepadScrollTarget,
        ))
        .with_children(|panel| {
            // Detail icon (hidden by default, shown when an item is selected).
            // Centered horizontally and pushed down so the corona stays inside
            // the panel. Locked silhouettes get the white corona via BoxShadow.
            panel.spawn((
                ImageNode::default(),
                Node {
                    width: Val::Px(DETAIL_ICON_SIZE),
                    height: Val::Px(DETAIL_ICON_SIZE),
                    align_self: AlignSelf::Center,
                    margin: UiRect {
                        top: Val::Px(DETAIL_ICON_TOP_MARGIN),
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    display: Display::None,
                    ..default()
                },
                BoxShadow(vec![]),
                DetailIcon,
            ));

            panel.spawn((
                Text::new("Select an item"),
                TextFont::from_font_size(DETAIL_NAME_FONT_SIZE),
                TextColor(TEXT_COLOR),
                DetailTitle,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_CATEGORY_FONT_SIZE),
                TextColor(DESCRIPTION_COLOR),
                DetailCategory,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                TextColor(UNLOCKED_COLOR),
                DetailDescription,
            ));

            // Container for the colored "Status effects" section. Populated
            // on the fly in `update_detail_panel`.
            panel.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(Val::Px(8.0)),
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                DetailStatusContainer,
            ));

            panel.spawn((
                Text::new(""),
                TextFont::from_font_size(DETAIL_FLAVOR_FONT_SIZE),
                TextColor(LOCKED_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                DetailFlavor,
            ));

            // Level history container (used by stats tab, hidden by default)
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    display: Display::None,
                    ..default()
                },
                LevelHistoryContainer,
            ));
        });
}

pub(super) fn spawn_right_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Percent(RIGHT_PANEL_PERCENT),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RightPanelContent,
        ))
        .with_children(|right| {
            // Tab bar
            right
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                })
                .with_children(|tabs| {
                    for tab in CompendiumTab::all() {
                        let is_active = *tab == CompendiumTab::Spells;
                        let (bg, border) = if is_active {
                            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                        } else {
                            (INACTIVE_TAB_BG, TAB_BORDER)
                        };

                        tabs.spawn((
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
                            crate::ui::components::ButtonColors {
                                background: bg,
                                border,
                            },
                            TabButton(*tab),
                            crate::ui::focus::Focusable,
                            crate::ui::focus::TabFocusable,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(tab.label()),
                                TextFont::from_font_size(TAB_FONT_SIZE),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    }
                });

            // Scrollable content area
            right
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    ScrollPosition::default(),
                    crate::ui::focus::GamepadScrollTarget,
                    ScrollableCompendiumContainer,
                    BackgroundColor(SECTION_BG),
                ))
                .with_children(|scroll| {
                    scroll.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(SECTION_PADDING)),
                            ..default()
                        },
                        ItemsContainer,
                    ));
                });
        });
}

pub(crate) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

pub(crate) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}
