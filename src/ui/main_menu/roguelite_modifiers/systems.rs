use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::RogueliteModifiers;
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::constants::SLIDER_TRACK_WIDTH;
use crate::ui::systems::{
    default_content_node, spawn_button, spawn_page_container, spawn_slider_row,
    spawn_title_with_shadow, SliderRowConfig,
};

use super::components::{
    ModifierButtonAction, ModifierSliderDownButton, ModifierSliderFill, ModifierSliderHandle,
    ModifierSliderText, ModifierSliderTrack, ModifierSliderUpButton, ModifierSliderValue,
    OnRogueliteModifiersScreen,
};
use super::constants::*;

/// Sets up the roguelite modifiers screen UI.
pub(super) fn setup(mut commands: Commands, modifiers: Option<Res<RogueliteModifiers>>) {
    let already_exists = modifiers.is_some();
    let mods = modifiers.map(|m| m.clone()).unwrap_or_default();
    if !already_exists {
        commands.insert_resource(mods.clone());
    }

    let content = spawn_page_container(
        &mut commands,
        OnRogueliteModifiersScreen,
        false,
        default_content_node(),
    );

    commands.entity(content).with_children(|parent| {
        // Title
        spawn_title_with_shadow(
            parent,
            "Run Modifiers",
            TITLE_FONT_SIZE,
            TEXT_COLOR,
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        );

        // Subtitle
        parent.spawn((
            Text::new("Adjust difficulty before starting your run"),
            TextFont::from_font_size(SUBTITLE_FONT_SIZE),
            TextColor(DESCRIPTION_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        ));

        // Sliders
        let sliders = [
            ModifierSliderValue::GameSpeed,
            ModifierSliderValue::EnemyEffectiveness,
            ModifierSliderValue::EnemyCount,
        ];

        for slider_value in sliders {
            spawn_modifier_slider(parent, slider_value, &mods);
        }

        // Reset button
        parent
            .spawn(Node {
                margin: UiRect::top(Val::Px(MARGIN_SMALL)),
                ..default()
            })
            .with_children(|row| {
                spawn_button(
                    row,
                    "Reset All",
                    ModifierButtonAction::Reset,
                    &RESET_BUTTON_STYLE,
                );
            });

        // Button row
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(MARGIN),
                margin: UiRect::top(Val::Px(MARGIN)),
                ..default()
            })
            .with_children(|row| {
                spawn_button(row, "Back", ModifierButtonAction::Back, &BACK_BUTTON_STYLE);
                spawn_button(
                    row,
                    "Continue",
                    ModifierButtonAction::Continue,
                    &CONTINUE_BUTTON_STYLE,
                );
            });
    });
}

/// Spawns a single modifier slider using the shared slider row helper.
fn spawn_modifier_slider(
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
            label_width: 200.0,
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

/// Handles button actions on the modifiers screen.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ModifierButtonAction>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut modifiers: ResMut<RogueliteModifiers>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                ModifierButtonAction::Continue => {
                    channel_change.write(ChannelChangeMessage);
                    next_state.set(MenuState::WizardSelect);
                }
                ModifierButtonAction::Back => {
                    channel_change.write(ChannelChangeMessage);
                    next_state.set(MenuState::GameModeSelect);
                }
                ModifierButtonAction::Reset => {
                    *modifiers = RogueliteModifiers::default();
                }
            }
        }
    }
}

/// Handles Escape key to go back to game mode select.
pub(super) fn escape_to_game_mode_select(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::GameModeSelect);
    }
}

/// Handles slider +/- button clicks.
pub(super) fn slider_button_action(
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

/// Handles dragging slider handles and clicking on tracks.
pub(super) fn slider_interaction(
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

        if is_dragging {
            if let Some(pos) = cursor_pos.normalized {
                // RelativeCursorPosition.normalized: center at (0,0),
                // left edge = -0.5, right edge = 0.5
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
}

/// Updates slider text displays when modifiers change.
pub(super) fn update_slider_text(
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
pub(super) fn update_sliders(
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
