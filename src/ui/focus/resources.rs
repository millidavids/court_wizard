//! Focus navigation resources.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::state::{AppState, InGameState, MenuState, MetaGameState, PauseMenuState};

/// The currently focused UI entity when navigating with a gamepad.
#[derive(Resource, Default, Debug)]
pub(crate) struct FocusedEntity(pub Option<Entity>);

/// Composite key identifying a "screen" for focus memory. Derived from the
/// currently-active state tuple; most-specific state wins. Using a typed
/// `Copy` key instead of a formatted `String` avoids a per-frame allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ScreenKey {
    pub app: AppState,
    pub in_game: Option<InGameState>,
    pub pause: Option<PauseMenuState>,
    pub menu: Option<MenuState>,
    pub meta: Option<MetaGameState>,
}

/// Remembers the last-focused screen-space position per screen. When the
/// player returns to a previously-visited screen, focus snaps to the
/// focusable nearest the saved position.
#[derive(Resource, Default, Debug)]
pub(crate) struct ScreenFocusMemory(pub HashMap<ScreenKey, Vec2>);

/// Stashed focus entity captured when a `ModalOverlay` first appears.
/// Restored to `FocusedEntity` after the modal closes so focus returns to
/// where it was before the modal opened.
#[derive(Resource, Default, Debug)]
pub(crate) struct PreModalFocus(pub Option<Entity>);

/// When this resource exists, focus navigation (D-pad/stick button jumping),
/// right-stick scroll, and focus-driven autoscroll are all suppressed.
/// Inserted by screens that drive a custom pointer-style input (e.g. the
/// Study tab's spell-web cursor) and need to own the sticks exclusively.
#[derive(Resource, Default, Debug)]
pub(crate) struct FocusNavInhibit;
