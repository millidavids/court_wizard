//! Key-capture overlay, binding update systems, and confirmation popup.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;

use super::super::components::{
    ButtonColors, ConfirmationAction, ConfirmationPopup, KeyBindingButton, KeyBindingText,
    KeyCaptureAction, KeyCaptureOverlay, KeyCaptureState, OnSettingsScreen, PendingConflict,
    SettingsButtonAction,
};
use super::super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH, BUTTON_FONT_SIZE,
    DANGER_BUTTON_BACKGROUND, DANGER_BUTTON_BORDER, LABEL_FONT_SIZE, MARGIN, MARGIN_SMALL,
    OPTION_BUTTON_HEIGHT, OPTION_BUTTON_WIDTH, POPUP_BOX_BG, POPUP_OVERLAY_BG, SECTION_FONT_SIZE,
    SELECTED_BORDER, TEXT_COLOR,
};
use crate::config::input_bindings::{is_bindable_key, key_display_name, key_name};

/// Opens key capture overlay when a binding button is clicked.
pub fn key_binding_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&KeyBindingButton>,
    mut capture_state: ResMut<KeyCaptureState>,
    capture_query: Query<&KeyCaptureOverlay>,
) {
    if !capture_query.is_empty() {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(binding) = button_query.get(event.button) {
            capture_state.active = Some(KeyCaptureAction {
                context: binding.context,
                action: binding.action,
                pending_conflict: None,
            });
            spawn_capture_overlay(&mut commands);
        }
    }
}

/// Captures key input for rebinding, with unbind and conflict confirmation support.
pub fn capture_key_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut capture_state: ResMut<KeyCaptureState>,
    overlay_query: Query<Entity, With<KeyCaptureOverlay>>,
    mut bindings: ResMut<crate::config::InputBindings>,
) {
    let Some(ref mut action) = capture_state.active else {
        return;
    };
    let context = action.context;
    let action_id = action.action;

    // Escape cancels
    if keyboard.just_pressed(KeyCode::Escape) {
        capture_state.active = None;
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Backspace unbinds
    if keyboard.just_pressed(KeyCode::Backspace) {
        bindings.set(context, action_id, None);
        capture_state.active = None;
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    let pressed_key = keyboard.get_just_pressed().find(|k| is_bindable_key(**k));
    let Some(&key) = pressed_key else {
        return;
    };

    // Check if confirming a pending conflict (same key pressed again)
    if let Some(ref pending) = action.pending_conflict
        && pending.key == key
    {
        // User confirmed — swap: unbind conflicting, bind new
        bindings.set(
            pending.conflicting_context,
            pending.conflicting_action,
            None,
        );
        bindings.set(context, action_id, Some(key));
        capture_state.active = None;
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Check for conflict
    if let Some(conflict_display) = bindings.would_conflict(key, context, action_id) {
        if let Some((conf_ctx, conf_action)) = bindings.find_conflict(key, context, action_id) {
            // Enter confirmation mode
            let key_label = key_name(key);
            action.pending_conflict = Some(PendingConflict {
                key,
                conflicting_context: conf_ctx,
                conflicting_action: conf_action,
            });
            // Respawn overlay with warning
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            spawn_capture_overlay_with_warning(
                &mut commands,
                &format!(
                    "{key_label} is already bound to {conflict_display}.\nPress {key_label} again to swap, or press another key."
                ),
            );
        }
        return;
    }

    // No conflict — apply immediately
    bindings.set(context, action_id, Some(key));
    capture_state.active = None;
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }
}

/// Returns true when no key capture is active (used as run condition for escape navigation).
pub fn key_capture_inactive(capture_state: Res<KeyCaptureState>) -> bool {
    capture_state.active.is_none()
}

fn spawn_capture_overlay(commands: &mut Commands) {
    spawn_capture_overlay_inner(commands, None);
}

fn spawn_capture_overlay_with_warning(commands: &mut Commands, warning: &str) {
    spawn_capture_overlay_inner(commands, Some(warning));
}

fn spawn_capture_overlay_inner(commands: &mut Commands, warning: Option<&str>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            KeyCaptureOverlay,
            OnSettingsScreen,
            crate::ui::focus::ModalOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(MARGIN_SMALL),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(SELECTED_BORDER),
                ))
                .with_children(|popup| {
                    if let Some(warning) = warning {
                        popup.spawn((
                            Text::new(warning),
                            TextFont::from_font_size(LABEL_FONT_SIZE),
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        ));
                    } else {
                        popup.spawn((
                            Text::new("Press a key..."),
                            TextFont::from_font_size(SECTION_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                        ));
                    }
                    popup.spawn((
                        Text::new("(Backspace to unbind, Escape to cancel)"),
                        TextFont::from_font_size(LABEL_FONT_SIZE),
                        TextColor(Color::hsla(0.0, 0.0, 0.5, 1.0)),
                    ));
                });
        });
}

/// Updates binding button text when InputBindings changes.
pub fn update_key_binding_text(
    bindings: Res<crate::config::InputBindings>,
    mut texts: Query<(&mut Text, &KeyBindingText)>,
) {
    if !bindings.is_changed() {
        return;
    }
    for (mut text, binding_text) in &mut texts {
        let key = bindings.get(binding_text.context, binding_text.action);
        text.0 = key_display_name(key).to_string();
    }
}

/// Spawns a confirmation popup overlay in the center of the screen.
pub(crate) fn spawn_confirmation_popup(
    commands: &mut Commands,
    action: SettingsButtonAction,
    message: &str,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            ConfirmationPopup,
            OnSettingsScreen,
            crate::ui::focus::ModalOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(DANGER_BUTTON_BORDER),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(message),
                        TextFont::from_font_size(SECTION_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(MARGIN),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(OPTION_BUTTON_WIDTH),
                                    height: Val::Px(OPTION_BUTTON_HEIGHT),
                                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BorderColor::all(DANGER_BUTTON_BORDER),
                                BackgroundColor(DANGER_BUTTON_BACKGROUND),
                                ButtonColors {
                                    background: DANGER_BUTTON_BACKGROUND,
                                    border: DANGER_BUTTON_BORDER,
                                },
                                ConfirmationAction::Confirm(action),
                                crate::ui::focus::Focusable,
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Confirm"),
                                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });

                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(OPTION_BUTTON_WIDTH),
                                    height: Val::Px(OPTION_BUTTON_HEIGHT),
                                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER),
                                BackgroundColor(BUTTON_BACKGROUND),
                                ButtonColors {
                                    background: BUTTON_BACKGROUND,
                                    border: BUTTON_BORDER,
                                },
                                ConfirmationAction::Cancel,
                                crate::ui::focus::Focusable,
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Cancel"),
                                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                        });
                });
        });
}
