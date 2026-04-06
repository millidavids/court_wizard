//! Shared UI systems used across all menus and screens.

use bevy::ecs::relationship::Relationship;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use bevy::ui::RelativeCursorPosition;

use super::components::{ButtonColors, ButtonStyle};
use super::constants::{
    CONTENT_BG, CONTENT_BORDER, DETAIL_BG, DETAIL_BORDER, DETAIL_PADDING, LEFT_PANEL_WIDTH,
    LIST_BG, LIST_BORDER, OVERLAY_BG, PANEL_BORDER_RADIUS, SCROLL_BG, SCROLL_BORDER,
    SCROLL_SHADOW_COLOR, SHADOW_COLOR, SLIDER_BORDER_WIDTH, SLIDER_BUTTON_BG,
    SLIDER_BUTTON_BORDER_COLOR, SLIDER_BUTTON_FONT_SIZE, SLIDER_BUTTON_SIZE, SLIDER_GAP,
    SLIDER_LABEL_FONT_SIZE, SLIDER_TRACK_WIDTH, TEXT_PRIMARY, TEXT_SHADOW_COLOR, TWO_PANEL_GAP,
    TWO_PANEL_PADDING,
};
use super::styles::{item_hovered, item_pressed};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseClicked;
use crate::state::{InGameState, MenuState, MultiplayerGameState, PauseMenuState};

/// Scales a font size down based on text width to fit within a constrained area.
///
/// Returns `base_font` when `max_width <= min_chars`, scaling linearly down to
/// `base_font * min_scale` when `max_width >= max_chars`.
pub(crate) fn scale_font_by_text_width(
    max_width: f32,
    min_chars: f32,
    max_chars: f32,
    min_scale: f32,
    base_font: f32,
) -> f32 {
    let t = ((max_width - min_chars) / (max_chars - min_chars)).clamp(0.0, 1.0);
    base_font * (1.0 - t * (1.0 - min_scale))
}

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub struct ButtonPressedDown;

/// Run condition that returns true if there are any MouseClicked messages.
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().next().is_some()
}

/// Tracks button press state and sends click events.
///
/// This system handles the core button click detection:
/// - Marks buttons as pressed when interaction becomes Pressed
/// - Sends MouseClicked event when interaction changes from Pressed to non-Pressed (either Hovered or None)
/// - Only sends click event if the button was previously marked as pressed down
///
/// This works for both mouse (Pressed → Hovered → None) and touch (Pressed → None).
pub fn button_click_detection(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, Option<&ButtonPressedDown>),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_clicked: MessageWriter<MouseClicked>,
) {
    for (entity, interaction, pressed_down) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered | Interaction::None => {
                // If button was pressed down and is now released, send click event
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                    button_clicked.write(MouseClicked { button: entity });
                }
            }
        }
    }
}

/// Handles button interaction visual feedback for all buttons with `ButtonColors`.
///
/// Updates button background and border colors based on the current
/// interaction state (None, Hovered, or Pressed).
pub fn button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, colors, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = item_pressed(colors.background).into();
                *border_color = BorderColor::all(item_pressed(colors.border));
            }
            Interaction::Hovered => {
                *bg_color = item_hovered(colors.background).into();
                *border_color = BorderColor::all(item_hovered(colors.border));
            }
            Interaction::None => {
                *bg_color = colors.background.into();
                *border_color = BorderColor::all(colors.border);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Page container (shared by settings, progress, instructions, and overlays)
// ---------------------------------------------------------------------------

/// Returns the shared styling bundle for inner scrollable areas.
/// Callers should also add `ScrollPosition::default()` and their marker component.
pub(crate) fn scroll_area_style() -> (BackgroundColor, BorderColor, BorderRadius, BoxShadow) {
    (
        BackgroundColor(SCROLL_BG),
        BorderColor::all(SCROLL_BORDER),
        BorderRadius::all(Val::Px(4.0)),
        BoxShadow::new(
            SCROLL_SHADOW_COLOR,
            Val::Px(0.0),
            Val::Px(2.0),
            Val::Px(2.0),
            Val::Px(8.0),
        ),
    )
}

/// Standard content node for page containers (column, centered, with scroll clipping).
pub fn default_content_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(20.0)),
        border: UiRect::all(Val::Px(1.0)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Returns the standard content Node for a two-panel page layout (left detail + right list).
/// Pass this to `spawn_page_container()` as the `content_node`.
pub fn two_panel_content_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TWO_PANEL_PADDING)),
        column_gap: Val::Px(TWO_PANEL_GAP),
        border: UiRect::all(Val::Px(1.0)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Spawns the standard left detail panel (300px fixed, gold-bordered box).
/// Returns the **inner detail box** entity — add your content as children of this.
pub fn spawn_left_detail_panel(parent: &mut ChildSpawnerCommands) -> Entity {
    let mut detail_box_id = Entity::PLACEHOLDER;
    parent
        .spawn(Node {
            width: Val::Px(LEFT_PANEL_WIDTH),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            row_gap: Val::Px(16.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|left| {
            detail_box_id = left
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(DETAIL_PADDING)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_grow: 1.0,
                        ..default()
                    },
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                ))
                .id();
        });
    detail_box_id
}

/// Like `spawn_left_detail_panel`, but makes the detail box scrollable.
/// Inserts the given `marker` component and `ScrollPosition` for scroll handling.
pub fn spawn_scrollable_left_detail_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
) -> Entity {
    let detail_box = spawn_left_detail_panel(parent);
    parent.commands().entity(detail_box).insert((
        marker,
        ScrollPosition::default(),
    ));
    parent
        .commands()
        .entity(detail_box)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.overflow = Overflow::scroll_y();
        });
    detail_box
}

/// Spawns the standard right scrollable panel (flex-grow, dark background with scroll).
/// `marker` is attached for screen-specific queries (e.g. scroll handling).
/// Returns the scrollable content entity — add your content as children of this.
pub fn spawn_right_scroll_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    inner_direction: FlexDirection,
    inner_gap: f32,
) -> Entity {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                flex_direction: inner_direction,
                row_gap: Val::Px(inner_gap),
                column_gap: Val::Px(inner_gap),
                overflow: Overflow::scroll_y(),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
            ScrollPosition::default(),
            marker,
        ))
        .id()
}

/// Spawns a full-screen page with a semi-transparent overlay and a styled
/// content container inside it. Returns the content container entity so the
/// caller can add children.
///
/// `content_node` controls the inner container layout (flex direction, padding,
/// gaps, overflow, etc.). Use `default_content_node()` for the standard look.
/// The border color, background, border-radius, and shadow are applied
/// automatically.
///
/// When `pause_menu` is true, `GlobalZIndex(500)` is added so the page
/// renders above in-game UI.
pub fn spawn_page_container<M: Component>(
    commands: &mut Commands,
    screen_marker: M,
    pause_menu: bool,
    content_node: Node,
) -> Entity {
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(OVERLAY_BG),
        screen_marker,
    ));

    if pause_menu {
        root.insert(GlobalZIndex(500));
    }

    let root_id = root.id();

    let content_id = commands
        .spawn((
            content_node,
            BackgroundColor(CONTENT_BG),
            BorderColor::all(CONTENT_BORDER),
            BorderRadius::all(Val::Px(6.0)),
            BoxShadow::new(
                SHADOW_COLOR,
                Val::Px(0.0),
                Val::Px(4.0),
                Val::Px(4.0),
                Val::Px(12.0),
            ),
        ))
        .id();

    commands.entity(root_id).add_child(content_id);

    content_id
}

// ---------------------------------------------------------------------------
// Shared Escape key handling
// ---------------------------------------------------------------------------

/// Handles back button click to return to the main menu landing screen.
///
/// Shared across all menu sub-screens (changelog, credits, instructions, etc.).
pub fn handle_back_to_landing(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&super::components::BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Handles Escape key to return to the main menu landing screen.
pub fn escape_to_landing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::Landing);
    }
}

/// Handles Escape key to return to the pause menu main screen.
pub fn escape_to_pause_main(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(PauseMenuState::Main);
    }
}

/// Handles Escape key to return to running gameplay state (SP and/or MP).
pub fn escape_to_running(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::Running);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::Running);
        }
    }
}

/// Consumes the mouse button state on exit to prevent click bleed-through.
pub fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
}

// ---------------------------------------------------------------------------
// Shared scroll handling
// ---------------------------------------------------------------------------

/// Handles mouse wheel scrolling for any scrollable container marked with `T`.
///
/// Walks up the entity hierarchy from the hovered entity to find a scrollable
/// parent, then adjusts its `ScrollPosition` based on the mouse wheel delta.
pub fn handle_scroll<T: Component>(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<(&mut ScrollPosition, &ComputedNode), With<T>>,
    parent_query: Query<&ChildOf>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    'event: for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        continue 'event;
                    }

                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared button spawning
// ---------------------------------------------------------------------------

/// Generic screen cleanup system that despawns all entities with the given marker component.
///
/// Use as `cleanup_screen::<OnMyScreen>` when registering `OnExit` systems.
pub fn cleanup_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

/// Spawns a styled button as a child of the given parent.
///
/// # Arguments
///
/// * `parent` - The parent entity to spawn the button under
/// * `text` - The button label text
/// * `action` - Any component to attach as the button's action identifier
/// * `style` - The `ButtonStyle` configuration for dimensions and colors
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Component,
    style: &ButtonStyle,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height),
                border: UiRect::all(Val::Px(style.border_width)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(style.border),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(style.background),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            action,
        ))
        .with_children(|button| {
            if style.text_shadow {
                spawn_shadowed_text(
                    button,
                    text,
                    style.font_size,
                    style.text_color,
                    Node::default(),
                );
            } else {
                button.spawn((
                    Text::new(text),
                    TextFont::from_font_size(style.font_size),
                    TextColor(style.text_color),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            }
        });
}

/// Spawns text with a drop shadow inside the given parent.
/// Uses a relative wrapper with an absolute-positioned shadow behind the main text.
/// Offset scales with font size (font_size / 20).
fn spawn_shadowed_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    let offset = font_size / 20.0;
    parent
        .spawn(Node {
            position_type: PositionType::Relative,
            ..node
        })
        .with_children(|wrapper| {
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(TEXT_SHADOW_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(offset),
                    top: Val::Px(offset),
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Spawns a title text with a drop shadow effect.
pub fn spawn_title_with_shadow(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    spawn_shadowed_text(parent, text, font_size, text_color, node);
}

// ── Shared Slider ─────────────────────────────────────────────────────────

/// Configuration for spawning a slider row.
pub(crate) struct SliderRowConfig<
    'a,
    TText,
    TDownButton,
    TUpButton,
    TSliderTrack,
    TSliderFill,
    TSliderHandle,
> {
    pub label: &'a str,
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub label_width: f32,
    pub text_component: TText,
    pub down_button: TDownButton,
    pub up_button: TUpButton,
    pub slider_track: TSliderTrack,
    pub slider_fill: TSliderFill,
    pub slider_handle: TSliderHandle,
}

/// Spawns a slider row with label, decrease/increase buttons, track, fill, handle, and value text.
/// Shared by settings and roguelite modifier screens.
pub(crate) fn spawn_slider_row<
    TText: Component,
    TDownButton: Component,
    TUpButton: Component,
    TSliderTrack: Component,
    TSliderFill: Component,
    TSliderHandle: Component,
>(
    parent: &mut ChildSpawnerCommands,
    config: SliderRowConfig<
        '_,
        TText,
        TDownButton,
        TUpButton,
        TSliderTrack,
        TSliderFill,
        TSliderHandle,
    >,
) {
    let SliderRowConfig {
        label,
        current_value,
        min_value,
        max_value,
        label_width,
        text_component,
        down_button,
        up_button,
        slider_track,
        slider_fill,
        slider_handle,
    } = config;

    let range = max_value - min_value;
    let normalized = if range > 0.0 {
        (current_value - min_value) / range
    } else {
        0.0
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(SLIDER_GAP),
            ..default()
        })
        .with_children(|row| {
            // Label (min_width ensures consistent alignment regardless of text length)
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                TextColor(TEXT_PRIMARY),
                Node {
                    min_width: Val::Px(label_width),
                    width: Val::Px(label_width),
                    ..default()
                },
            ));

            // Controls
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(SLIDER_GAP),
                ..default()
            })
            .with_children(|controls| {
                // Decrease button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        down_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Slider track
                controls
                    .spawn((
                        Node {
                            width: Val::Px(SLIDER_TRACK_WIDTH),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(6.0)),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                        slider_track,
                    ))
                    .with_children(|track| {
                        // Slider fill
                        track.spawn((
                            Node {
                                width: Val::Percent(normalized * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BorderRadius {
                                top_left: Val::Px(6.0),
                                bottom_left: Val::Px(6.0),
                                top_right: Val::Px(0.0),
                                bottom_right: Val::Px(0.0),
                            },
                            BackgroundColor(SLIDER_BUTTON_BORDER_COLOR),
                            slider_fill,
                        ));

                        // Slider handle (offset by -2px to center the 4px wide bar)
                        track.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0),
                                top: Val::Px(-4.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(2.0)),
                            BackgroundColor(Color::WHITE),
                            BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            slider_handle,
                        ));
                    });

                // Increase button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        up_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u32)),
                    TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                    TextColor(TEXT_PRIMARY),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text_component,
                ));
            });
        });
}
