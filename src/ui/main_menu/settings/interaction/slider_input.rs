use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::GameConfig;

use super::super::components::{
    SliderAdjusted, SliderDownButton, SliderFill, SliderHandle, SliderText, SliderTrack,
    SliderUpButton,
};

/// Handles slider button clicks for increment/decrement.
pub fn slider_button_action(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    down_buttons: Query<&SliderDownButton>,
    up_buttons: Query<&SliderUpButton>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    for event in button_clicked.read() {
        // Check if it's a down button
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
        // Check if it's an up button
        else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates slider text displays when values change.
pub fn update_slider_text(
    game_config: Res<GameConfig>,
    mut slider_texts: Query<(&mut Text, &SliderText)>,
) {
    if game_config.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&game_config);
            text.0 = format!("{}%", (value * 100.0) as u8);
        }
    }
}

/// Updates slider fill widths and handle positions when values change.
pub fn update_sliders(
    game_config: Res<GameConfig>,
    mut slider_fills: Query<(&mut Node, &SliderFill), Without<SliderHandle>>,
    mut slider_handles: Query<(&mut Node, &SliderHandle), Without<SliderFill>>,
) {
    if game_config.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&game_config);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            // Normalize to 0-100% range
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&game_config);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * crate::ui::constants::SLIDER_TRACK_WIDTH - 2.0);
        }
    }
}

/// Handles dragging slider handles and clicking on tracks.
///
/// Uses the track's `RelativeCursorPosition` for both click-to-jump and drag
/// tracking. This is immune to scale factor, viewport, and CRT distortion
/// differences between mouse motion units and logical UI pixels.
pub fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut SliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &SliderTrack)>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
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
                // Start dragging the corresponding handle
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
    // This gives pixel-perfect tracking regardless of scale factor or viewport.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging && let Some(pos) = cursor_pos.normalized {
            // RelativeCursorPosition.normalized: center at (0,0),
            // left edge = -0.5, right edge = 0.5
            let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

            let min = track.value.min_value();
            let max = track.value.max_value();
            let range = max - min;
            let new_value = (min + normalized * range).clamp(min, max);

            if (track.value.get(&game_config) - new_value).abs() > f32::EPSILON {
                track.value.set(&mut game_config, new_value);
                slider_adjusted.write(SliderAdjusted);
            }
        }
    }
}
