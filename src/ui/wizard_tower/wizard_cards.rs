//! Expandable wizard card grid shared by Roguelite and Endless tabs.
//!
//! Provides a 3-column grid of wizard type cards that can be expanded
//! to show full details. Used by both the Roguelite and Endless tabs
//! when the player clicks "Change Wizard Type" / "Switch Wizard Type".

use bevy::prelude::*;
use bevy::ui::ComputedNode;

use crate::config::WizardType;
use crate::config::save_data::load_unified_save;
use crate::game::input::messages::MouseClicked;
use crate::ui::components::ButtonActive;
use crate::ui::constants::{ACTIVE_ACCENT, TEXT_DISABLED, TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::layout::RightPanelView;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GRID_COLUMNS: usize = 3;
const CARD_GAP: f32 = 8.0;
const CARD_BORDER_WIDTH: f32 = 1.0;
const CARD_BORDER_RADIUS: f32 = 4.0;
const CARD_NAME_FONT_SIZE: f32 = 15.0;
const CARD_DESC_FONT_SIZE: f32 = 10.0;
const CARD_LONG_DESC_FONT_SIZE: f32 = 10.0;
const CARD_STATUS_FONT_SIZE: f32 = 10.0;
const CARD_BG: Color = Color::hsla(220.0, 0.08, 0.11, 0.75);
const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 0.6);
const CARD_BORDER_SELECTED: Color = ACTIVE_ACCENT;
const LOCKED_CARD_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.6);
const LOCKED_CARD_BORDER: Color = Color::hsla(25.0, 0.10, 0.12, 0.5);
const LOCKED_TEXT_COLOR: Color = Color::hsla(30.0, 0.06, 0.45, 1.0);

const SELECT_BUTTON_STYLE: crate::ui::components::ButtonStyle =
    crate::ui::components::ButtonStyle {
        width: 80.0,
        height: 30.0,
        border_width: 2.0,
        font_size: 12.0,
        background: crate::ui::constants::BUTTON_BG,
        border: crate::ui::constants::BUTTON_BORDER,
        text_color: TEXT_PRIMARY,
        text_shadow: true,
    };

const EXPAND_BUTTON_STYLE: crate::ui::components::ButtonStyle =
    crate::ui::components::ButtonStyle {
        width: 80.0,
        height: 30.0,
        border_width: 2.0,
        font_size: 12.0,
        background: Color::hsla(220.0, 0.08, 0.15, 0.75),
        border: Color::hsla(0.0, 0.0, 0.30, 0.6),
        text_color: TEXT_MUTED,
        text_shadow: true,
    };

/// Height for the expand animation (pixels per second).
const EXPAND_ANIM_SPEED: f32 = 800.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker on the root container of a wizard card.
#[derive(Component)]
pub(super) struct WizardCardContainer(pub WizardType);

/// Marker for each currently-expanded wizard card. Multiple cards can
/// carry this marker simultaneously — expansions are independent per card.
#[derive(Component)]
pub(super) struct ExpandedWizardCard;

/// Resource tracking which wizard is currently selected.
#[derive(Resource)]
pub(super) struct SelectedWizard(pub WizardType);

impl Default for SelectedWizard {
    fn default() -> Self {
        Self(WizardType::BoringOleMage)
    }
}

/// Actions for wizard card buttons.
#[derive(Component, Clone, Copy)]
pub(super) enum WizardCardAction {
    Select(WizardType),
    ToggleExpand(WizardType),
}

/// Marker for the expandable content section inside a card.
#[derive(Component)]
pub(super) struct CardExpandSection;

// ---------------------------------------------------------------------------
// Grid builder
// ---------------------------------------------------------------------------

/// Spawns a 3-column wizard card grid inside the given parent entity.
/// Cards are arranged in columns so expanding a card only pushes down
/// cards in the same column.
/// Marker for the scrollable wizard-card grid container so the generic
/// `handle_scroll` system can find it on mouse wheel events. Re-exported
/// `pub(crate)` so the tutorial system can also tag it for highlighting.
#[derive(Component)]
pub(crate) struct WizardCardScrollContainer;

pub(super) fn build_wizard_card_grid(
    commands: &mut Commands,
    parent: Entity,
    // True in the multiplayer "Switch Wizard" view: Psychopath isn't supported in
    // MP, so hide its card there. SP tabs pass false (it's selectable there).
    exclude_mp_unsupported: bool,
) {
    let save = load_unified_save();
    let unlocked_names: Vec<String> = save
        .map(|s| s.player.unlocked_content.wizard_types)
        .unwrap_or_default();

    let all_wizards: Vec<WizardType> = WizardType::all()
        .iter()
        .copied()
        .filter(|wt| !exclude_mp_unsupported || *wt != WizardType::Psychopath)
        .collect();

    commands.entity(parent).with_children(|grid_parent| {
        // Scrollable row container holding 3 columns
        grid_parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(CARD_GAP),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                crate::ui::focus::GamepadScrollTarget,
                WizardCardScrollContainer,
            ))
            .with_children(|row| {
                // Create 3 columns
                for col in 0..GRID_COLUMNS {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(CARD_GAP),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|column| {
                        // Distribute wizards across columns
                        for (i, wizard_type) in all_wizards.iter().enumerate() {
                            if i % GRID_COLUMNS != col {
                                continue;
                            }
                            let is_unlocked = unlocked_names
                                .iter()
                                .any(|name| name.as_str() == wizard_type.save_key());
                            if is_unlocked {
                                spawn_compact_card(column, *wizard_type);
                            } else {
                                spawn_locked_card(column, *wizard_type);
                            }
                        }
                    });
                }
            });
    });
}

/// Spawns a wizard card with always-visible header/buttons and a
/// collapsible middle section that animates open/closed.
fn spawn_compact_card(parent: &mut ChildSpawnerCommands, wizard_type: WizardType) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(CARD_BG),
            BorderColor::all(CARD_BORDER),
            WizardCardContainer(wizard_type),
            crate::ui::focus::ScrollRevealBounds,
        ))
        .with_children(|card| {
            // Wizard name (always visible)
            card.spawn((
                Text::new(wizard_type.display_name()),
                TextFont::from_font_size(CARD_NAME_FONT_SIZE),
                TextColor(ACTIVE_ACCENT),
            ));

            // Short description (always visible)
            card.spawn((
                Text::new(wizard_type.description()),
                TextFont::from_font_size(CARD_DESC_FONT_SIZE),
                TextColor(TEXT_MUTED),
            ));

            // Expandable section (starts at 0 height, clipped)
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                CardExpandSection,
            ))
            .with_children(|section| {
                // Separator
                section.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(CARD_BORDER),
                ));

                // Long description
                section.spawn((
                    Text::new(wizard_type.long_description()),
                    TextFont::from_font_size(CARD_LONG_DESC_FONT_SIZE),
                    TextColor(TEXT_MUTED),
                ));

                // Status
                let status = get_wizard_status(wizard_type);
                section.spawn((
                    Text::new(status),
                    TextFont::from_font_size(CARD_STATUS_FONT_SIZE),
                    TextColor(TEXT_DISABLED),
                    Node {
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                ));
            });

            // Button row (always visible)
            card.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                margin: UiRect::top(Val::Px(6.0)),
                padding: UiRect::bottom(Val::Px(6.0)),
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    "Select",
                    (
                        WizardCardAction::Select(wizard_type),
                        crate::ui::focus::CrossRowHorizontalNav,
                    ),
                    &SELECT_BUTTON_STYLE,
                );
                spawn_button(
                    row,
                    "Expand",
                    (
                        WizardCardAction::ToggleExpand(wizard_type),
                        crate::ui::focus::CrossRowHorizontalNav,
                    ),
                    &EXPAND_BUTTON_STYLE,
                );
            });
        });
}

/// Spawns a locked wizard card (grayed out, no buttons).
fn spawn_locked_card(parent: &mut ChildSpawnerCommands, wizard_type: WizardType) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(80.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                row_gap: Val::Px(4.0),
                border_radius: BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(LOCKED_CARD_BG),
            BorderColor::all(LOCKED_CARD_BORDER),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(wizard_type.locked_description()),
                TextFont::from_font_size(CARD_DESC_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

// ---------------------------------------------------------------------------
// Card interaction systems
// ---------------------------------------------------------------------------

/// Handles wizard card button actions (Select, ToggleExpand). Expansions
/// are per-card: toggling a card's Expand button never collapses sibling
/// cards, so multiple can be open at once.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_wizard_card_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<(Entity, &WizardCardAction)>,
    mut selected: ResMut<SelectedWizard>,
    mut right_panel_view: ResMut<RightPanelView>,
    mut config: ResMut<crate::config::GameConfig>,
    mut active_save: ResMut<crate::config::ActiveSave>,
    expanded_check: Query<&WizardCardContainer, With<ExpandedWizardCard>>,
    mut card_containers: Query<
        (Entity, &WizardCardContainer, &mut Node),
        Without<CardExpandSection>,
    >,
) {
    for event in button_clicked.read() {
        let Ok((btn_entity, action)) = button_query.get(event.button) else {
            continue;
        };
        match action {
            WizardCardAction::Select(wizard_type) => {
                selected.0 = *wizard_type;
                crate::config::save_data::load_or_create_wizard(
                    *wizard_type,
                    &mut config,
                    &mut active_save,
                );
                *right_panel_view = RightPanelView::TabContent;
            }
            WizardCardAction::ToggleExpand(wizard_type) => {
                let was_expanded = expanded_check.iter().any(|c| c.0 == *wizard_type);
                if was_expanded {
                    collapse_card(&mut commands, &mut card_containers, *wizard_type);
                    commands.entity(btn_entity).remove::<ButtonActive>();
                } else {
                    expand_card(&mut commands, &mut card_containers, *wizard_type);
                    commands.entity(btn_entity).insert(ButtonActive);
                }
            }
        }
    }
}

/// Starts expanding a card's expandable section.
fn expand_card(
    commands: &mut Commands,
    card_containers: &mut Query<
        (Entity, &WizardCardContainer, &mut Node),
        Without<CardExpandSection>,
    >,
    wizard_type: WizardType,
) {
    for (entity, container, _node) in card_containers.iter_mut() {
        if container.0 == wizard_type {
            commands
                .entity(entity)
                .insert((ExpandedWizardCard, BorderColor::all(CARD_BORDER_SELECTED)));
        }
    }
}

/// Starts collapsing a card's expandable section.
fn collapse_card(
    commands: &mut Commands,
    card_containers: &mut Query<
        (Entity, &WizardCardContainer, &mut Node),
        Without<CardExpandSection>,
    >,
    wizard_type: WizardType,
) {
    for (entity, container, _node) in card_containers.iter_mut() {
        if container.0 == wizard_type {
            commands.entity(entity).remove::<ExpandedWizardCard>();
            commands
                .entity(entity)
                .insert(BorderColor::all(CARD_BORDER));
        }
    }
}

/// Triggers expand/collapse animation on CardExpandSection children
/// when the parent card's ExpandedWizardCard marker changes.
///
/// The expanded target height is the section's measured content height
/// (`ComputedNode::content_size`) rather than a fixed constant, so long
/// descriptions never clip — at narrow card widths (e.g. fullscreen on a TV)
/// the text wraps into more lines and the section grows to fit it.
pub(super) fn animate_card_expand(
    time: Res<Time>,
    card_query: Query<(Has<ExpandedWizardCard>, &Children), With<WizardCardContainer>>,
    mut section_query: Query<(&mut Node, &ComputedNode), With<CardExpandSection>>,
) {
    let dt = time.delta_secs();
    for (is_expanded, children) in &card_query {
        for child in children.iter() {
            let Ok((mut node, computed)) = section_query.get_mut(child) else {
                continue;
            };

            let current = match node.height {
                Val::Px(h) => h,
                _ => 0.0,
            };

            // Collapsed and already closed (the common case for every card that
            // isn't open): nothing to animate — skip before reading ComputedNode.
            if !is_expanded && current == 0.0 {
                continue;
            }

            // `content_size` is the laid-out height of the section's children
            // (separator + long description + status) in physical pixels;
            // convert to the logical pixels `Node::height` expects.
            let inv_sf = computed.inverse_scale_factor();
            if inv_sf <= 0.0 {
                continue;
            }
            let target = if is_expanded {
                computed.content_size().y * inv_sf
            } else {
                0.0
            };

            let diff = target - current;
            let new_height = if diff.abs() < 1.0 {
                target
            } else {
                let step = EXPAND_ANIM_SPEED * dt;
                if diff > 0.0 {
                    (current + step).min(target)
                } else {
                    (current - step).max(target)
                }
            };

            // Only write when the value actually changes so collapsed cards
            // (target == current == 0) don't mark their Node dirty and force a
            // UI relayout every frame.
            if node.height != Val::Px(new_height) {
                node.height = Val::Px(new_height);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Gets a status string for a wizard type from save data.
fn get_wizard_status(wizard_type: WizardType) -> String {
    let save = load_unified_save();
    let wizard_type_name = wizard_type.save_key().to_string();

    if let Some(save) = save {
        for wizard_save in &save.wizards {
            if wizard_save.wizard_type.save_key() == wizard_type_name {
                return format!("Level {}", wizard_save.current_level);
            }
        }
    }

    "New".to_string()
}
