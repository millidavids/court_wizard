use bevy::prelude::*;

use super::super::button_systems::{edge_color, opaque};
use super::super::components::{
    ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, ButtonStyle,
};
use super::super::constants::{
    BUTTON_3D_OFFSET_REST, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR, TEXT_SHADOW_COLOR,
};
use super::super::focus::Focusable;

/// Generic screen cleanup system that despawns all entities with the given marker component.
///
/// Use as `cleanup_screen::<OnMyScreen>` when registering `OnExit` systems.
pub fn cleanup_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
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
                TextLayout::justify(Justify::Center),
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
                TextLayout::justify(Justify::Center),
            ));
        });
}

/// Spawns a 3D pushable button with edge + front layers.
///
/// The button wrapper is transparent and contains:
/// - An **edge** child (darker bg, stays in place) that peeks through at the bottom
/// - A **front** child (button face, offset upward) that slides on interaction
///
/// The front face moves up on hover and down on press, creating a physical depth illusion.
#[allow(clippy::too_many_arguments)]
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Bundle,
    style: &ButtonStyle,
) {
    let depth = -BUTTON_3D_OFFSET_REST; // positive value = edge visible at bottom

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height + depth),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BoxShadow(vec![bevy::ui::ShadowStyle {
                color: BUTTON_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(2.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(4.0),
            }]),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            ButtonAnimState {
                current: BUTTON_3D_OFFSET_REST,
                target: BUTTON_3D_OFFSET_REST,
            },
            Focusable,
            action,
        ))
        .with_children(|wrapper| {
            // Edge layer — same size as front but offset down and slightly narrower.
            // Outline lives here so it appears to go behind the front face.
            wrapper.spawn((
                ButtonEdge,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(edge_color(style.background)),
                Outline::new(Val::Px(1.0), Val::Px(1.0), BUTTON_REST_OUTLINE),
            ));

            // Front face — the interactive surface, offset upward.
            wrapper
                .spawn((
                    ButtonFront,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(style.height),
                        border: UiRect::all(Val::Px(style.border_width)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        position_type: PositionType::Relative,
                        top: Val::Px(BUTTON_3D_OFFSET_REST),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(opaque(style.background)),
                    BorderColor::all(style.border),
                ))
                .with_children(|front| {
                    if style.text_shadow {
                        spawn_shadowed_text(
                            front,
                            text,
                            style.font_size,
                            style.text_color,
                            Node::default(),
                        );
                    } else {
                        front.spawn((
                            Text::new(text),
                            TextFont::from_font_size(style.font_size),
                            TextColor(style.text_color),
                            TextLayout::justify(Justify::Center),
                        ));
                    }
                });
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

/// Spawns a standard page header row: title on the left, spacer, back button on the right.
///
/// The back button is marked `NoGamepadFocus` — on controller the B/East button
/// handles "back" universally, so we don't want the D-pad to land on it.
pub fn spawn_page_header<B: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    font_size: f32,
    title_color: Color,
    back_action: B,
    button_style: &ButtonStyle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|header| {
            spawn_title_with_shadow(header, title, font_size, title_color, Node::default());
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_button(
                header,
                "Back",
                (back_action, super::super::focus::NoGamepadFocus),
                button_style,
            );
        });
}
