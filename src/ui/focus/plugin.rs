//! Plugin registering focus navigation systems.

use bevy::prelude::*;

use super::resources::{FocusNavInhibit, FocusedEntity, PreModalFocus, ScreenFocusMemory};
use super::systems::{
    ScrollAnimation, animate_scroll, auto_refocus, autoscroll_to_focused, clear_focus_on_back,
    clear_focus_on_device_switch, focus_navigation, override_focused_interaction,
    restore_focus_on_screen_change, right_stick_scroll, tab_cycle, track_modal_focus_restore,
    update_focus_memory,
};
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::state::InGameState;

pub(crate) struct FocusPlugin;

fn gamepad_active(active: Res<ActiveInputDevice>) -> bool {
    active.is_gamepad()
}

/// Focus navigation is disabled during active gameplay (`InGameState::Running`):
/// any forced `Interaction::Hovered` on a HUD button would cause
/// `block_spell_input_on_button_interaction` to silently block every cast.
fn focus_enabled(in_game: Option<Res<State<InGameState>>>) -> bool {
    in_game
        .map(|s| *s.get() != InGameState::Running)
        .unwrap_or(true)
}

/// Focus navigation is enabled when neither a screen-driven cursor mode
/// (Study spell-web reticle) nor a tutorial overlay is active. Tutorial
/// overlays must remain navigable so the player can press Next/Skip even
/// while standing on a screen that would otherwise own the sticks.
fn nav_enabled(
    inhibit: Option<Res<FocusNavInhibit>>,
    active_tutorial: Option<Res<crate::ui::tutorial::resources::ActiveTutorial>>,
) -> bool {
    inhibit.is_none() || active_tutorial.is_some()
}

fn no_active_tutorial(active: Option<Res<crate::ui::tutorial::resources::ActiveTutorial>>) -> bool {
    active.is_none()
}

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusedEntity>()
            .init_resource::<ScreenFocusMemory>()
            .init_resource::<PreModalFocus>()
            // track_modal_focus_restore runs BEFORE auto_refocus so a restored
            // pre-modal focus survives auto_refocus's validity check.
            // restore_focus_on_screen_change runs BEFORE auto_refocus so the
            // saved position takes precedence over the top-leftmost default.
            .add_systems(
                Update,
                (
                    clear_focus_on_device_switch.run_if(resource_changed::<ActiveInputDevice>),
                    update_focus_memory.run_if(gamepad_active),
                    track_modal_focus_restore
                        .run_if(gamepad_active)
                        .run_if(focus_enabled),
                    restore_focus_on_screen_change
                        .run_if(gamepad_active)
                        .run_if(focus_enabled),
                    auto_refocus
                        .run_if(gamepad_active)
                        .run_if(focus_enabled)
                        .run_if(nav_enabled),
                    focus_navigation
                        .run_if(gamepad_active)
                        .run_if(focus_enabled)
                        .run_if(nav_enabled),
                    // Bumpers cycle tabs regardless of `FocusNavInhibit` so
                    // tab switching still works when a full-screen cursor
                    // mode (e.g. Study spell web) is active. Tab cycling is
                    // disabled while a tutorial overlay is up so the player
                    // can't accidentally swap tabs and trigger a different
                    // tab's tutorial mid-walkthrough.
                    tab_cycle
                        .run_if(gamepad_active)
                        .run_if(focus_enabled)
                        .run_if(no_active_tutorial),
                    clear_focus_on_back.run_if(focus_enabled),
                    autoscroll_to_focused
                        .run_if(gamepad_active)
                        .run_if(focus_enabled)
                        .run_if(nav_enabled),
                )
                    .chain(),
            )
            // Must run before autoscroll so a stick lingering past the
            // deadzone can't strip the freshly-inserted `ScrollAnimation`.
            .add_systems(
                Update,
                right_stick_scroll
                    .before(autoscroll_to_focused)
                    .run_if(gamepad_active)
                    .run_if(focus_enabled)
                    .run_if(nav_enabled),
            )
            // Smooth scroll animation — only runs while ScrollAnimation tweens are active.
            .add_systems(
                Update,
                animate_scroll.run_if(any_with_component::<ScrollAnimation>),
            )
            // The Interaction override must run in PreUpdate AFTER
            // `correct_ui_interaction_for_barrel`. Running in Update causes
            // a per-frame REST↔HOVER oscillation because `button_interaction`
            // only re-reads `Interaction` in PreUpdate.
            .add_systems(
                PreUpdate,
                override_focused_interaction
                    .after(crate::game::crt_effect::correct_ui_interaction_for_barrel)
                    .run_if(gamepad_active)
                    .run_if(focus_enabled)
                    .run_if(nav_enabled),
            )
            .add_systems(
                OnEnter(InGameState::Running),
                |mut focused: ResMut<FocusedEntity>| focused.0 = None,
            );
    }
}
