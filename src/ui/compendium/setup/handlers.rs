use bevy::prelude::*;

use super::super::components::*;

// ---------------------------------------------------------------------------
// Tab switching
// ---------------------------------------------------------------------------

pub(crate) fn handle_tab_click(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    tab_query: Query<&TabButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && state.active_tab != tab_btn.0
        {
            state.active_tab = tab_btn.0;
            state.selected_item = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Item selection
// ---------------------------------------------------------------------------

pub(crate) fn handle_item_click(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    item_query: Query<&ItemButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(item_btn) = item_query.get(event.button) {
            state.selected_item = Some(item_btn.0.clone());
        }
    }
}

/// Updates ButtonActive markers on item buttons when the selection changes.
/// Separate from rebuild_on_state_change to avoid the system parameter limit.
pub(crate) fn update_item_active_state(
    mut commands: Commands,
    state: Res<CompendiumState>,
    item_buttons: Query<(Entity, &ItemButton)>,
) {
    if !state.is_changed() {
        return;
    }
    for (entity, item_btn) in &item_buttons {
        if state.selected_item.as_ref() == Some(&item_btn.0) {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
        }
    }
}

pub(crate) fn handle_toggle_save_run(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    toggle_query: Query<&ToggleSaveRunButton>,
    mut state: ResMut<CompendiumState>,
) {
    for event in button_clicked.read() {
        if let Ok(toggle_btn) = toggle_query.get(event.button) {
            crate::config::save_data::toggle_roguelite_run_saved(toggle_btn.0);
            // Force rebuild by re-setting the same state (triggers is_changed)
            state.set_changed();
        }
    }
}

pub(crate) fn handle_copy_seed(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    copy_query: Query<&CopySeedButton>,
) {
    for event in button_clicked.read() {
        if let Ok(copy_btn) = copy_query.get(event.button)
            && let Ok(mut clipboard) = arboard::Clipboard::new()
        {
            let _ = clipboard.set_text(copy_btn.0.to_string());
        }
    }
}
