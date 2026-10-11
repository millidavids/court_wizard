//! Run conditions for focus navigation systems.

use bevy::prelude::*;

use super::resources::FocusNavInhibit;
use crate::state::{InGameState, MultiplayerGameState};

pub(super) use crate::game::input::gamepad::gamepad_active;

/// Focus navigation is disabled during active gameplay (`InGameState::Running`
/// in single-player, `MultiplayerGameState::Running` in multiplayer): a
/// controller-focused HUD button would light up and A would press it while the
/// player is playing. Menus/overlays (Paused, Settings, …) keep focus
/// navigation, so only the `Running` variants disable it.
pub(super) fn focus_enabled(
    in_game: Option<Res<State<InGameState>>>,
    mp_game: Option<Res<State<MultiplayerGameState>>>,
) -> bool {
    if mp_game
        .map(|s| *s.get() == MultiplayerGameState::Running)
        .unwrap_or(false)
    {
        return false;
    }
    in_game
        .map(|s| *s.get() != InGameState::Running)
        .unwrap_or(true)
}

/// Focus navigation is enabled when neither a screen-driven cursor mode
/// (Study spell-web reticle) nor a tutorial overlay is active. Tutorial
/// overlays must remain navigable so the player can press Next/Skip even
/// while standing on a screen that would otherwise own the sticks.
pub(super) fn nav_enabled(
    inhibit: Option<Res<FocusNavInhibit>>,
    active_tutorial: Option<Res<crate::ui::tutorial::resources::ActiveTutorial>>,
) -> bool {
    inhibit.is_none() || active_tutorial.is_some()
}

pub(super) fn no_active_tutorial(
    active: Option<Res<crate::ui::tutorial::resources::ActiveTutorial>>,
) -> bool {
    active.is_none()
}
