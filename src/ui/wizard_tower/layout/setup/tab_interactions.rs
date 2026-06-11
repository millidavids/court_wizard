use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::KillStats;
use crate::state::AppState;
use crate::ui::components::{ButtonActive, ButtonColors, ButtonFront};

use super::super::super::components::WizardTowerButtonAction;
use super::super::super::constants::*;
use super::resources::{DisabledTab, RightPanelView, WizardTowerTab, WizardTowerTabButton};

// ---------------------------------------------------------------------------
// Tab click handling
// ---------------------------------------------------------------------------

/// Updates the active tab when a tab button is clicked. Disabled tabs are ignored.
pub(crate) fn handle_tab_click(
    mut button_clicked: MessageReader<MouseClicked>,
    tab_query: Query<&WizardTowerTabButton, Without<DisabledTab>>,
    mut tab_resource: ResMut<WizardTowerTab>,
    mut right_panel_view: ResMut<RightPanelView>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && *tab_resource != tab_btn.0
        {
            *tab_resource = tab_btn.0;
            // Reset to default tab content view when switching tabs
            *right_panel_view = RightPanelView::TabContent;
        }
    }
}

// ---------------------------------------------------------------------------
// Back button handling
// ---------------------------------------------------------------------------

/// Handles the back button to return to the main menu.
pub(crate) fn handle_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(WizardTowerButtonAction::ReturnToMenu) = button_query.get(event.button) {
            return_to_main_menu(
                &mut next_app_state,
                &mut kill_stats,
                &mut active_save,
                &mut channel_change,
            );
        }
    }
}

/// Handles Escape key / gamepad back to return to the main menu from the wizard tower.
#[allow(clippy::too_many_arguments)]
pub(crate) fn escape_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<crate::game::input::gamepad::messages::MenuBackPressed>,
    tab: Option<Res<WizardTowerTab>>,
    selected_spell: Option<Res<super::super::super::components::SelectedStudySpell>>,
    selected_bonus: Option<Res<super::super::super::components::SelectedInsightBonus>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if !keyboard.just_pressed(KeyCode::Escape) && !back_pressed {
        return;
    }
    // On Study with a selection, `study_back_to_cursor` consumes the back
    // to deselect; swallow it here so we don't also exit to main menu.
    let study_consumed = tab.as_deref().is_some_and(|t| *t == WizardTowerTab::Study)
        && (selected_spell.as_deref().is_some_and(|s| s.0.is_some())
            || selected_bonus.as_deref().is_some_and(|s| s.0.is_some()));
    if study_consumed {
        return;
    }
    return_to_main_menu(
        &mut next_app_state,
        &mut kill_stats,
        &mut active_save,
        &mut channel_change,
    );
}

fn return_to_main_menu(
    next_app_state: &mut ResMut<NextState<AppState>>,
    kill_stats: &mut ResMut<KillStats>,
    active_save: &mut ResMut<ActiveSave>,
    channel_change: &mut MessageWriter<ChannelChangeMessage>,
) {
    channel_change.write(ChannelChangeMessage);
    kill_stats.reset();
    active_save.0 = None;
    next_app_state.set(AppState::MainMenu);
}

// ---------------------------------------------------------------------------
// Tab visual state
// ---------------------------------------------------------------------------

/// Updates tab button visuals to reflect the currently active tab.
pub(crate) fn update_tab_active_state(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    connection: Res<crate::networking::resources::NetworkConnection>,
    tab_buttons: Query<(
        Entity,
        &WizardTowerTabButton,
        &Children,
        Has<DisabledTab>,
        Has<ButtonActive>,
        Has<crate::ui::focus::GamepadFocused>,
    )>,
    mut button_colors: Query<&mut ButtonColors>,
    mut front_q: Query<
        (&mut BackgroundColor, &Children),
        (With<ButtonFront>, Without<ButtonColors>),
    >,
    mut tab_text: Query<&mut TextColor>,
) {
    use crate::networking::resources::{ConnectionState, PeerRole};
    let connected = connection.state == ConnectionState::Connected;
    // A connected GUEST is locked to the Multiplayer (+ Study) screen: it does
    // everything from there, and locking the mode tabs stops it accidentally
    // starting a solo game. The HOST drives the mode tabs as normal.
    let is_guest_connected = connected && connection.role == Some(PeerRole::Guest);

    for (entity, tab_btn, children, is_disabled, has_active, focused) in &tab_buttons {
        // One coherent desired-enabled state per tab.
        let desired_enabled = match tab_btn.0 {
            // VS needs a connection, and is hidden from a connected guest.
            WizardTowerTab::Vs => connected && !is_guest_connected,
            // Mode-start tabs lock for a connected guest.
            WizardTowerTab::Endless | WizardTowerTab::Roguelite => !is_guest_connected,
            // The guest's home + Study are always reachable.
            WizardTowerTab::Multiplayer | WizardTowerTab::Study => true,
        };
        if desired_enabled && is_disabled {
            commands.entity(entity).remove::<DisabledTab>();
        } else if !desired_enabled && !is_disabled {
            commands.entity(entity).insert(DisabledTab);
        }

        let is_active = tab_btn.0 == *tab;
        // Keep the `ButtonActive` marker in sync (queried by other systems).
        if is_active && !has_active {
            commands.entity(entity).insert(ButtonActive);
        } else if !is_active && has_active {
            commands.entity(entity).remove::<ButtonActive>();
        }

        // Desired visuals, priority: disabled (greyed) > active > inactive. A
        // disabled tab greys its background too, not just its label, so it clearly
        // reads as unavailable; an enabled tab (e.g. VS once a host connects) gets
        // the normal active/inactive styling.
        let (bg, border, label_color) = if !desired_enabled {
            (DISABLED_TAB_BG, DISABLED_TAB_BORDER, DISABLED_TAB_TEXT)
        } else if is_active {
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, TEXT_COLOR)
        } else {
            (INACTIVE_TAB_BG, TAB_BORDER, TEXT_COLOR)
        };

        // Update the wrapper's `ButtonColors` — the source the 3D-button systems
        // (`sync_front_face_colors`) read to repaint the visible front face.
        if let Ok(mut colors) = button_colors.get_mut(entity) {
            if colors.background != bg {
                colors.background = bg;
            }
            if colors.border != border {
                colors.border = border;
            }
        }

        // The visible surface is the 3D `ButtonFront` child, and the label is
        // reparented into it. Paint the front-face background directly so it updates
        // immediately and for the active tab (which `sync_front_face_colors` skips) —
        // except when gamepad-focused, where the focus tint owns the background — and
        // recolour the (now nested) label text. All writes are change-guarded.
        let want_bg = crate::ui::systems::opaque(bg);
        for child in children.iter() {
            // Pre-3D-conversion frame: the text is still a direct child.
            if let Ok(mut text_color) = tab_text.get_mut(child)
                && text_color.0 != label_color
            {
                *text_color = TextColor(label_color);
            }
            if let Ok((mut front_bg, front_children)) = front_q.get_mut(child) {
                if !focused && front_bg.0 != want_bg {
                    *front_bg = BackgroundColor(want_bg);
                }
                for grandchild in front_children.iter() {
                    if let Ok(mut text_color) = tab_text.get_mut(grandchild)
                        && text_color.0 != label_color
                    {
                        *text_color = TextColor(label_color);
                    }
                }
            }
        }
    }
}
