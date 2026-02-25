use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::archetypes::arcanorouter::{
    ArcanoRouterState, SliderAdjustMessage, SliderType,
};

/// Returns the color for a slider type
fn slider_color(slider_type: SliderType) -> Color {
    match slider_type {
        SliderType::Range => RANGE_COLOR,
        SliderType::Mana => MANA_COLOR,
        SliderType::Power => POWER_COLOR,
        SliderType::Speed => SPEED_COLOR,
    }
}

/// Returns the label text for a slider type
fn slider_label(slider_type: SliderType) -> &'static str {
    match slider_type {
        SliderType::Range => "RANGE",
        SliderType::Mana => "MANA",
        SliderType::Power => "POWER",
        SliderType::Speed => "SPEED",
    }
}

/// Spawns the Arcanorouter display with 4 vertical sliders
pub(super) fn spawn_arcanorouter_display(mut commands: Commands, state: Res<ArcanoRouterState>) {
    // Root container - absolute positioned at bottom center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_OFFSET),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            OnGameplayScreen,
            ArcanoRouterDisplay,
        ))
        .with_children(|parent| {
            // Container for all sliders
            parent
                .spawn((
                    Node {
                        column_gap: Val::Px(SLIDER_GAP),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(BACKGROUND_COLOR),
                ))
                .with_children(|sliders_parent| {
                    // Spawn each slider
                    for slider_type in SliderType::all() {
                        let allocation = state.get_allocation(slider_type);
                        let color = slider_color(slider_type);

                        sliders_parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(2.0),
                                ..default()
                            })
                            .with_children(|slider_parent| {
                                // Bar container (background + fill)
                                slider_parent
                                    .spawn((
                                        Node {
                                            width: Val::Px(SLIDER_WIDTH),
                                            height: Val::Px(SLIDER_HEIGHT),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::FlexEnd, // Fill from bottom
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BAR_BACKGROUND_COLOR),
                                        BorderColor::all(color),
                                        BorderRadius::all(Val::Px(4.0)),
                                        SliderBar,
                                    ))
                                    .with_children(|bar_parent| {
                                        // Fill bar (grows upward from bottom)
                                        let fill_height = (allocation / 200.0) * SLIDER_HEIGHT;
                                        bar_parent.spawn((
                                            Node {
                                                width: Val::Percent(100.0),
                                                height: Val::Px(fill_height),
                                                ..default()
                                            },
                                            BackgroundColor(color),
                                            BorderRadius::all(Val::Px(2.0)),
                                            SliderFill { slider_type },
                                        ));
                                    });

                                // Label text at bottom
                                slider_parent.spawn((
                                    Text::new(slider_label(slider_type)),
                                    TextFont::from_font_size(LABEL_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                    }
                });
        });
}

/// Updates slider visual fills based on current allocations
pub(super) fn update_slider_visuals(
    state: Res<ArcanoRouterState>,
    mut fill_query: Query<(&SliderFill, &mut Node)>,
) {
    // Only update if state changed
    if !state.is_changed() {
        return;
    }

    // Update fill bar heights
    for (fill, mut node) in fill_query.iter_mut() {
        let allocation = state.get_allocation(fill.slider_type);
        let fill_height = (allocation / 200.0) * SLIDER_HEIGHT;
        node.height = Val::Px(fill_height);
    }
}

/// Handles mouse and keyboard input for slider adjustment
pub(super) fn handle_slider_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<SliderAdjustMessage>,
) {
    // Keyboard controls: 1-4 to select, arrow keys to adjust
    // For now, we'll use simple keyboard controls:
    // Q/A = Range, W/S = Mana, E/D = Power, R/F = Speed

    let sensitivity = 10.0; // Change by 10% per key press

    if keyboard.just_pressed(KeyCode::KeyQ) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Range,
            delta: sensitivity,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyA) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Range,
            delta: -sensitivity,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Mana,
            delta: sensitivity,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Mana,
            delta: -sensitivity,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Power,
            delta: sensitivity,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Power,
            delta: -sensitivity,
        });
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Speed,
            delta: sensitivity,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        writer.write(SliderAdjustMessage {
            slider: SliderType::Speed,
            delta: -sensitivity,
        });
    }
}
