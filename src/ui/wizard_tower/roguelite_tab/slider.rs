use bevy::prelude::*;

use crate::game::game_mode::components::RogueliteModifiers;
use crate::game::input::messages::MouseClicked;
use crate::ui::constants::SLIDER_TRACK_WIDTH;
use crate::ui::systems::{DragTrack, SliderRowConfig, spawn_slider_row};

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

/// Sets a modifier when its track is clicked or dragged, snapped to its step.
pub(crate) fn slider_interaction(
    slider_tracks: Query<(Ref<DragTrack>, &ModifierSliderTrack)>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    for (drag, track) in &slider_tracks {
        let Some(fraction) = DragTrack::moved(&drag) else {
            continue;
        };
        let min = track.value.min_value();
        let max = track.value.max_value();
        let new_value = (min + fraction * (max - min)).clamp(min, max);

        // Snap to nearest step
        let step = track.value.step();
        let snapped = (new_value / step).round() * step;
        let snapped = snapped.clamp(min, max);

        if (track.value.get(&modifiers) - snapped).abs() > f32::EPSILON {
            track.value.set(&mut modifiers, snapped);
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
