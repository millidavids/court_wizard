use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::game::game_mode::components::RogueliteModifiers;
use crate::game::input::messages::MouseClicked;
use crate::ui::constants::SLIDER_TRACK_WIDTH;
use crate::ui::systems::{SliderRowConfig, spawn_slider_row};

use super::components::{
    ModifierSliderDownButton, ModifierSliderFill, ModifierSliderHandle, ModifierSliderText,
    ModifierSliderTrack, ModifierSliderUpButton, ModifierSliderValue,
};

/// Spawns a single modifier slider using the shared slider row helper.
pub(super) fn spawn_modifier_slider(
    parent: &mut ChildSpawnerCommands,
    slider_value: ModifierSliderValue,
    modifiers: &RogueliteModifiers,
) {
    let current_value = slider_value.get(modifiers);

    spawn_slider_row(
        parent,
        SliderRowConfig {
            label: slider_value.label(),
            current_value,
            min_value: slider_value.min_value(),
            max_value: slider_value.max_value(),
            text_component: ModifierSliderText {
                value: slider_value,
            },
            down_button: ModifierSliderDownButton {
                value: slider_value,
            },
            up_button: ModifierSliderUpButton {
                value: slider_value,
            },
            slider_track: ModifierSliderTrack {
                value: slider_value,
            },
            slider_fill: ModifierSliderFill {
                value: slider_value,
            },
            slider_handle: ModifierSliderHandle {
                value: slider_value,
                is_dragging: false,
            },
        },
    );
}

/// Handles slider +/- button clicks.
pub(crate) fn slider_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    down_buttons: Query<&ModifierSliderDownButton>,
    up_buttons: Query<&ModifierSliderUpButton>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    for event in button_clicked.read() {
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut modifiers, new_value);
        } else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut modifiers, new_value);
        }
    }
}

pub(crate) fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut ModifierSliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &ModifierSliderTrack)>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    // Stop dragging when mouse is released
    if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
        for (_interaction, mut slider_handle) in &mut slider_handles {
            slider_handle.is_dragging = false;
        }
        return;
    }

    // Check if track was clicked (start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, _cursor_pos, track) in &slider_tracks {
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }

        // Also start dragging if the handle itself was clicked
        for (interaction, mut slider_handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                slider_handle.is_dragging = true;
            }
        }
    }

    // While dragging, use the track's RelativeCursorPosition to set the value.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging && let Some(pos) = cursor_pos.normalized {
            let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

            let min = track.value.min_value();
            let max = track.value.max_value();
            let range = max - min;
            let new_value = (min + normalized * range).clamp(min, max);

            // Snap to nearest step
            let step = track.value.step();
            let snapped = (new_value / step).round() * step;
            let snapped = snapped.clamp(min, max);

            if (track.value.get(&modifiers) - snapped).abs() > f32::EPSILON {
                track.value.set(&mut modifiers, snapped);
            }
        }
    }
}

/// Updates slider text displays when modifiers change.
pub(crate) fn update_slider_text(
    modifiers: Res<RogueliteModifiers>,
    mut slider_texts: Query<(&mut Text, &ModifierSliderText)>,
) {
    if modifiers.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&modifiers);
            text.0 = format!("{}%", (value * 100.0) as u32);
        }
    }
}

/// Updates slider fill widths and handle positions when modifiers change.
pub(crate) fn update_sliders(
    modifiers: Res<RogueliteModifiers>,
    mut slider_fills: Query<(&mut Node, &ModifierSliderFill), Without<ModifierSliderHandle>>,
    mut slider_handles: Query<(&mut Node, &ModifierSliderHandle), Without<ModifierSliderFill>>,
) {
    if modifiers.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&modifiers);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&modifiers);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0);
        }
    }
}
