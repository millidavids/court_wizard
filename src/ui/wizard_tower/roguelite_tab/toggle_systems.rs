use bevy::prelude::*;

use crate::config::save_data;
use crate::game::input::messages::MouseClicked;
use crate::ui::components::{ButtonActive, ButtonColors};

use super::components::{
    ConfirmUnlockAction, ConfirmUnlockPopup, ExpandedToggles, PendingToggles,
    ToggleDescriptionNode, ToggleExpandButton, ToggleRowContainer, ToggleUnlockButton,
};
use super::constants::{TOGGLE_OFF_BG, TOGGLE_OFF_BORDER, TOGGLE_ON_BG, TOGGLE_ON_BORDER};
use super::toggle_spawn::spawn_unlock_popup;

/// Toggles the expand/collapse state of a toggle modifier's description.
pub(crate) fn toggle_expand_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut expanded: ResMut<ExpandedToggles>,
    mut descriptions: Query<(&ToggleDescriptionNode, &mut Node)>,
    expand_btn_entities: Query<(Entity, &ToggleExpandButton)>,
) {
    for event in button_clicked.read() {
        let Ok(btn) = expand_buttons.get(event.button) else {
            continue;
        };
        let toggle = btn.0;

        let is_expanded = expanded.0.contains(&toggle);
        if is_expanded {
            expanded.0.remove(&toggle);
        } else {
            expanded.0.insert(toggle);
        }
        let now_expanded = !is_expanded;

        // Update description visibility
        for (desc, mut node) in &mut descriptions {
            if desc.0 == toggle {
                node.display = if now_expanded {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // Toggle ButtonActive on the expand button
        for (entity, expand_btn) in &expand_btn_entities {
            if expand_btn.0 == toggle {
                if now_expanded {
                    commands.entity(entity).insert(ButtonActive);
                } else {
                    commands.entity(entity).remove::<ButtonActive>();
                }
            }
        }
    }
}

/// Toggles ON/OFF for unlocked toggle modifiers by clicking the row itself.
/// For locked toggles, clicking the row opens the unlock confirmation popup.
pub(crate) fn toggle_row_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    row_query: Query<&ToggleRowContainer>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    existing_popup: Query<Entity, With<ConfirmUnlockPopup>>,
) {
    for event in button_clicked.read() {
        // Skip if the click was on the expand button
        if expand_buttons.get(event.button).is_ok() {
            continue;
        }

        let Ok(row) = row_query.get(event.button) else {
            continue;
        };
        let toggle = row.0;
        let is_unlocked = save_data::is_toggle_unlocked(toggle);

        if !is_unlocked {
            if existing_popup.is_empty() {
                spawn_unlock_popup(&mut commands, toggle);
            }
            continue;
        }

        // Unlocked -- toggle on/off
        pending_toggles.toggle(toggle);
        let now_enabled = pending_toggles.is_enabled(toggle);

        let (bg, border) = if now_enabled {
            (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
        } else {
            (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
        };

        for (entity, container, mut bg_color, mut border_color, mut btn_colors) in
            &mut row_containers
        {
            if container.0 == toggle {
                bg_color.0 = bg;
                *border_color = BorderColor::all(border);
                btn_colors.background = bg;
                btn_colors.border = border;

                if now_enabled {
                    commands.entity(entity).insert(ButtonActive);
                } else {
                    commands.entity(entity).remove::<ButtonActive>();
                }
            }
        }
    }
}

/// Handles confirm/cancel actions in the unlock confirmation popup.
pub(crate) fn handle_unlock_confirmation(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&ConfirmUnlockAction>,
    popup_query: Query<Entity, With<ConfirmUnlockPopup>>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    unlock_cost_texts: Query<(Entity, &ToggleUnlockButton)>,
) {
    for event in button_clicked.read() {
        let Ok(action) = action_query.get(event.button) else {
            continue;
        };

        match action {
            ConfirmUnlockAction::Confirm(toggle) => {
                let toggle = *toggle;
                if save_data::unlock_toggle(toggle) {
                    pending_toggles.enabled.push(toggle);

                    for (entity, container, mut bg_color, mut border_color, mut btn_colors) in
                        &mut row_containers
                    {
                        if container.0 == toggle {
                            bg_color.0 = TOGGLE_ON_BG;
                            *border_color = BorderColor::all(TOGGLE_ON_BORDER);
                            commands.entity(entity).insert(ButtonActive);
                            btn_colors.background = TOGGLE_ON_BG;
                            btn_colors.border = TOGGLE_ON_BORDER;
                        }
                    }

                    for (entity, unlock_btn) in &unlock_cost_texts {
                        if unlock_btn.0 == toggle {
                            commands.entity(entity).try_despawn();
                            break;
                        }
                    }
                }
            }
            ConfirmUnlockAction::Cancel => {}
        }

        // Dismiss popup
        for entity in &popup_query {
            commands.entity(entity).try_despawn();
        }
    }
}
