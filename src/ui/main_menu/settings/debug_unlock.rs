//! Dev-only "Unlock Everything" button for the Gameplay settings tab.
//!
//! Spawned `Visibility::Hidden` and revealed by the global F2 debug toggle
//! (`crate::game::debug_ui::DebugUiVisible` via `sync_marker_visibility`), just
//! like the study-tab "+10000 Insight" and action-bar infinite-mana buttons.
//! Clicking it unlocks all game content for testing. The whole module is gated
//! behind `#[cfg(debug_assertions)]` at its `mod` declaration, so none of this
//! is compiled into release builds.

use bevy::prelude::*;

use crate::game::input::messages::MouseClicked;
use crate::ui::components::ButtonStyle;
use crate::ui::focus::NoGamepadFocus;
use crate::ui::notification::{NotificationEntry, NotificationQueue};
use crate::ui::systems::spawn_button;

/// Marker on the wrapper node — the `sync_marker_visibility` target driven by F2.
#[derive(Component)]
pub(crate) struct UnlockEverythingButton;

/// Marker on the inner clickable button — what the click handler matches against.
/// `MouseClicked.button` is the entity that owns `Button`/`Interaction` (the one
/// `spawn_button` spawns), so the click marker must live there, not on the wrapper.
#[derive(Component)]
pub(crate) struct UnlockEverythingAction;

/// Orange styling to make the debug button visually distinct from normal settings buttons.
const ORANGE_DEBUG_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 40.0,
    border_width: 3.0,
    font_size: 14.0,
    background: Color::srgba(0.55, 0.30, 0.05, 0.85),
    border: Color::srgb(1.0, 0.55, 0.1),
    text_color: Color::srgb(1.0, 0.92, 0.78),
    text_shadow: true,
};

/// Spawn the hidden orange button into the Gameplay tab column. Absolutely
/// positioned so it has zero layout impact whether shown or hidden (a flex child
/// with `Visibility::Hidden` would still reserve its row). Mirrors the study-tab
/// `DebugInsightButton` block.
pub(crate) fn spawn_unlock_everything_button(section: &mut ChildSpawnerCommands) {
    section
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                right: Val::Px(12.0),
                ..default()
            },
            Visibility::Hidden,
            UnlockEverythingButton,
        ))
        .with_children(|wrapper| {
            spawn_button(
                wrapper,
                "Unlock Everything",
                // `NoGamepadFocus` overrides the `Focusable` that `spawn_button`
                // always adds, keeping the hidden button out of gamepad traversal.
                (UnlockEverythingAction, NoGamepadFocus),
                &ORANGE_DEBUG_BUTTON_STYLE,
            );
        });
}

/// Unlock all content when the button is clicked, then toast confirmation.
pub(crate) fn unlock_everything_button_action(
    mut clicks: MessageReader<MouseClicked>,
    actions: Query<&UnlockEverythingAction>,
    mut queue: ResMut<NotificationQueue>,
) {
    for event in clicks.read() {
        if actions.get(event.button).is_ok() {
            crate::config::save_data::unlock_everything_for_testing();
            queue.push(NotificationEntry::Toast {
                message: "All content unlocked (debug)",
            });
        }
    }
}
