use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::super::components::ButtonColors;
use super::super::constants::{
    DOT_LEADER_COLOR, SLIDER_BORDER_WIDTH, SLIDER_BUTTON_BG, SLIDER_BUTTON_BORDER_COLOR,
    SLIDER_BUTTON_FONT_SIZE, SLIDER_BUTTON_SIZE, SLIDER_GAP, SLIDER_LABEL_FONT_SIZE,
    SLIDER_TRACK_BG, SLIDER_TRACK_WIDTH, TEXT_PRIMARY,
};

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
    pub text_component: TText,
    pub down_button: TDownButton,
    pub up_button: TUpButton,
    pub slider_track: TSliderTrack,
    pub slider_fill: TSliderFill,
    pub slider_handle: TSliderHandle,
}

/// Spawns a dot-leader filler that expands to fill available horizontal space.
pub(crate) fn spawn_dot_leader(parent: &mut ChildSpawnerCommands, font_size: f32) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_width: Val::Px(0.0),
            overflow: Overflow::clip(),
            margin: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        })
        .with_child((
            Text::new("......................................................................"),
            TextFont::from_font_size(font_size),
            TextColor(DOT_LEADER_COLOR),
        ));
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
            align_items: AlignItems::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                TextColor(TEXT_PRIMARY),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
            ));

            spawn_dot_leader(row, SLIDER_LABEL_FONT_SIZE);

            // Controls
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
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
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        crate::ui::focus::Focusable,
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
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_TRACK_BG),
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
                                border_radius: BorderRadius {
                                    top_left: Val::Px(6.0),
                                    bottom_left: Val::Px(6.0),
                                    top_right: Val::Px(0.0),
                                    bottom_right: Val::Px(0.0),
                                },
                                ..default()
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
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
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
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        crate::ui::focus::Focusable,
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
