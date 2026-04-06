//! Shared wizard select UI elements used by both single-player and multiplayer screens.
//!
//! Contains card spawning, detail panel helpers, and shared styling constants
//! that are identical between the two wizard select screens.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::ui::components::ButtonColors;
use crate::ui::constants::{DETAIL_BG, DETAIL_BORDER, GOLD_ACCENT, TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::spawn_title_with_shadow;

// ===== Shared Components =====

/// Marker for the detail panel name text.
#[derive(Component)]
pub(super) struct DetailName;

/// Marker for the detail panel description text.
#[derive(Component)]
pub(super) struct DetailDescription;

/// Marker for the detail panel status text.
#[derive(Component)]
pub(super) struct DetailStatus;

/// Marker for a grid card, storing which wizard type it represents.
#[derive(Component)]
pub(super) struct WizardCard(pub WizardType);

/// Resource tracking which wizard type is currently previewed in the detail panel.
#[derive(Resource)]
pub(super) struct SelectedWizardPreview(pub WizardType);

// ===== Shared Styling Constants =====

/// Font size for the wizard select title text.
pub(super) const TITLE_FONT_SIZE: f32 = 29.0;

/// Font size for the subtitle text.
pub(super) const SUBTITLE_FONT_SIZE: f32 = 10.0;

/// Primary text color.
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Subdued text color for secondary elements.
pub(super) const SUBTITLE_COLOR: Color = TEXT_MUTED;

/// Width of the left panel (title + detail + buttons).
pub(super) const LEFT_PANEL_WIDTH: f32 = 300.0;

/// Margin between elements.
pub(super) const MARGIN: f32 = 20.0;

/// Total number of grid slots (4x4).
pub(super) const GRID_SLOTS: usize = 16;

/// Gap between grid cards in pixels.
pub(super) const CARD_GAP: f32 = 8.0;

/// Number of columns in the wizard grid.
pub(super) const GRID_COLUMNS: usize = 4;

/// Card width in pixels.
pub(super) const CARD_WIDTH: f32 = 210.0;

/// Card height in pixels.
pub(super) const CARD_HEIGHT: f32 = 140.0;

/// Card border width in pixels.
pub(super) const CARD_BORDER_WIDTH: f32 = 1.0;

/// Card border radius in pixels.
pub(super) const CARD_BORDER_RADIUS: f32 = 4.0;

/// Font size for wizard name on cards.
pub(super) const CARD_NAME_FONT_SIZE: f32 = 14.0;

/// Font size for wizard description on cards.
pub(super) const CARD_DESC_FONT_SIZE: f32 = 10.0;

/// Background color for unlocked wizard cards.
pub(super) const CARD_BG: Color = Color::hsla(220.0, 0.08, 0.11, 0.75);

/// Border color for unlocked wizard cards.
pub(super) const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 0.6);

/// Border color for the selected/active wizard card — gold accent.
pub(super) const CARD_BORDER_SELECTED: Color = GOLD_ACCENT;

/// Color for wizard type short description text on cards.
pub(super) const DESCRIPTION_COLOR: Color = TEXT_MUTED;

/// Accent color for wizard name text — slightly warm white.
pub(super) const CARD_NAME_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);

/// Detail panel border width.
pub(super) const DETAIL_BORDER_WIDTH: f32 = 1.0;

/// Detail panel border radius.
pub(super) const DETAIL_BORDER_RADIUS: f32 = 6.0;

/// Detail panel background color (from global palette).
pub(super) const DETAIL_PANEL_BG: Color = DETAIL_BG;

/// Detail panel border color (from global palette).
pub(super) const DETAIL_PANEL_BORDER: Color = DETAIL_BORDER;

/// Font size for the wizard name in the detail panel.
pub(super) const DETAIL_NAME_FONT_SIZE: f32 = 18.0;

/// Font size for the long description in the detail panel.
pub(super) const DETAIL_DESC_FONT_SIZE: f32 = 10.0;

/// Color for the long description text.
pub(super) const DETAIL_DESC_COLOR: Color = Color::hsla(0.0, 0.0, 0.58, 1.0);

/// Font size for status text in the detail panel.
pub(super) const DETAIL_STATUS_FONT_SIZE: f32 = 10.0;

/// Background color for locked (unavailable) wizard cards.
pub(super) const LOCKED_CARD_BG: Color = Color::hsla(220.0, 0.05, 0.065, 0.6);

/// Border color for locked wizard cards.
pub(super) const LOCKED_CARD_BORDER: Color = Color::hsla(220.0, 0.05, 0.12, 0.5);

/// Text color for locked wizard cards.
pub(super) const LOCKED_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.20, 1.0);

/// Separator line color.
pub(super) const SEPARATOR_COLOR: Color = Color::hsla(40.0, 0.15, 0.15, 1.0);

// ===== Shared Spawn Functions =====

/// Spawns an unlocked wizard card with the given button action component.
pub(super) fn spawn_wizard_card(
    parent: &mut ChildSpawnerCommands,
    wizard_type: WizardType,
    is_selected: bool,
    action: impl Component,
) {
    let border_color = if is_selected {
        CARD_BORDER_SELECTED
    } else {
        CARD_BORDER
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(CARD_BG),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
            ButtonColors {
                background: CARD_BG,
                border: border_color,
            },
            action,
            WizardCard(wizard_type),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(wizard_type.display_name()),
                TextFont::from_font_size(CARD_NAME_FONT_SIZE),
                TextColor(CARD_NAME_COLOR),
                TextLayout::new_with_justify(Justify::Center),
            ));

            card.spawn((
                Text::new(wizard_type.locked_description()),
                TextFont::from_font_size(CARD_DESC_FONT_SIZE),
                TextColor(DESCRIPTION_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(CARD_WIDTH - 20.0),
                    ..default()
                },
            ));
        });
}

/// Spawns a locked wizard card showing flavor text, not interactive.
pub(super) fn spawn_locked_wizard_card(parent: &mut ChildSpawnerCommands, wizard_type: WizardType) {
    parent
        .spawn((
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(LOCKED_CARD_BG),
            BorderColor::all(LOCKED_CARD_BORDER),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(wizard_type.locked_description()),
                TextFont::from_font_size(CARD_DESC_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(CARD_WIDTH - 20.0),
                    ..default()
                },
            ));
        });
}

/// Spawns a locked/unavailable card placeholder.
pub(super) fn spawn_locked_card(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Px(CARD_WIDTH),
                height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(LOCKED_CARD_BG),
            BorderColor::all(LOCKED_CARD_BORDER),
            BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new("???"),
                TextFont::from_font_size(CARD_NAME_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
        });
}

/// Spawns the title group (heading + subtitle + separator line).
pub(super) fn spawn_title_group(parent: &mut ChildSpawnerCommands, title: &str, subtitle: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|title_group| {
            spawn_title_with_shadow(
                title_group,
                title,
                TITLE_FONT_SIZE,
                TEXT_COLOR,
                Node::default(),
            );
            title_group.spawn((
                Text::new(subtitle.to_string()),
                TextFont::from_font_size(SUBTITLE_FONT_SIZE),
                TextColor(SUBTITLE_COLOR),
            ));
        });

    // Separator
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(SEPARATOR_COLOR),
    ));
}

/// Spawns the detail panel container with shared styling.
/// `build_contents` receives the card's ChildSpawnerCommands to add custom content.
pub(super) fn spawn_detail_panel_container(
    parent: &mut ChildSpawnerCommands,
    build_contents: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(DETAIL_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(DETAIL_PANEL_BG),
            BorderColor::all(DETAIL_PANEL_BORDER),
            BorderRadius::all(Val::Px(DETAIL_BORDER_RADIUS)),
        ))
        .with_children(build_contents);
}

/// Spawns the top section of the detail panel (wizard name + long description).
pub(super) fn spawn_detail_panel_top(parent: &mut ChildSpawnerCommands, wizard_type: WizardType) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|top| {
            top.spawn((
                Text::new(wizard_type.display_name()),
                TextFont::from_font_size(DETAIL_NAME_FONT_SIZE),
                TextColor(CARD_NAME_COLOR),
                DetailName,
            ));

            top.spawn((
                Text::new(wizard_type.long_description()),
                TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                TextColor(DETAIL_DESC_COLOR),
                Node {
                    max_width: Val::Px(LEFT_PANEL_WIDTH - 36.0),
                    ..default()
                },
                DetailDescription,
            ));
        });
}

/// Updates the detail panel text when previewing a different wizard.
pub(super) fn update_detail_panel_text(
    wizard_type: WizardType,
    detail_name: &mut Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDescription>,
            Without<DetailStatus>,
        ),
    >,
    detail_desc: &mut Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailStatus>,
        ),
    >,
) {
    if let Ok(mut name_text) = detail_name.single_mut() {
        **name_text = wizard_type.display_name().to_string();
    }
    if let Ok(mut desc_text) = detail_desc.single_mut() {
        **desc_text = wizard_type.long_description().to_string();
    }
}

/// Updates card border highlights when selection changes.
pub(super) fn update_card_borders(
    selected: WizardType,
    card_borders: &mut Query<(&WizardCard, &mut BorderColor, &mut ButtonColors)>,
) {
    for (card, mut border, mut colors) in card_borders.iter_mut() {
        let new_border = if card.0 == selected {
            CARD_BORDER_SELECTED
        } else {
            CARD_BORDER
        };
        *border = BorderColor::all(new_border);
        colors.border = new_border;
    }
}

/// Returns the grid container node for the right side of the wizard select screen.
pub(super) fn grid_container_node() -> Node {
    let grid_width =
        (CARD_WIDTH * GRID_COLUMNS as f32) + (CARD_GAP * (GRID_COLUMNS - 1) as f32);
    Node {
        flex_grow: 1.0,
        flex_shrink: 0.0,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::FlexEnd,
        align_content: AlignContent::FlexStart,
        column_gap: Val::Px(CARD_GAP),
        row_gap: Val::Px(CARD_GAP),
        min_width: Val::Px(grid_width),
        max_width: Val::Px(grid_width),
        ..default()
    }
}
