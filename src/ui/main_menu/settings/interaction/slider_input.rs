use bevy::prelude::*;

use crate::config::GameConfig;
use crate::ui::systems::DragTrack;

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

/// Sets a slider's value when its track is clicked or dragged.
///
/// `DragTrack::fraction` follows the barrel-corrected pointer, so it is immune
/// to scale factor, viewport, and CRT distortion.
pub fn slider_interaction(
    slider_tracks: Query<(Ref<DragTrack>, &SliderTrack)>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    for (drag, track) in &slider_tracks {
        let Some(fraction) = DragTrack::moved(&drag) else {
            continue;
        };
        let min = track.value.min_value();
        let max = track.value.max_value();
        let new_value = (min + fraction * (max - min)).clamp(min, max);

        if (track.value.get(&game_config) - new_value).abs() > f32::EPSILON {
            track.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}
