//! Focus navigation marker components.

use bevy::prelude::*;

/// Marks a UI entity as a candidate for gamepad focus navigation.
///
/// Any `Button` entity with this component is reachable via D-pad / stick
/// navigation when the gamepad is the active input device. Nearest-neighbor
/// navigation uses screen-space node centers.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Focusable;

/// Marks a focusable entity as a "tab" — LB/RB cycle through the set of
/// tab-focusables. Tabs are excluded from D-pad / left-stick navigation
/// so that focus stays on body panels.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TabFocusable;

/// Marks a button that should NOT participate in gamepad D-pad focus
/// navigation. Used for header "Back" buttons, which are reachable on
/// controller only via the B/East button (`MenuBackPressed`).
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct NoGamepadFocus;

/// Marker placed on the currently gamepad-focused button. Distinguishes
/// gamepad focus from mouse hover so visual systems can style it differently
/// (purple tint on the 3D front face). Inserted/removed by the focus system.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct GamepadFocused;

/// Marks a scrollable container that the right stick should scroll when the
/// gamepad is active. Used for text-heavy screens (Manual, Credits) and
/// detail panels where the user wants to scroll through content without
/// having to move focus through every item.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct GamepadScrollTarget;

/// Marks a flat-style focusable (no 3D `ButtonFront` child) that should have
/// its own `BackgroundColor` tinted purple while gamepad-focused. Stores the
/// base color so the tint can be cleanly removed on unfocus.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct FocusableFlatBackground {
    pub base: Color,
}

/// Marks the root entity of a modal/popup overlay. While any entity with
/// this marker exists, gamepad focus is restricted to descendants of a
/// modal — the main-screen focusables behind it become unreachable until
/// the modal closes.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ModalOverlay;

/// Marks a focusable that should allow cross-row Left/Right navigation.
/// Used for grid layouts (e.g. wizard cards) where expanding one item can
/// push its column's siblings out of alignment with other columns — Left/Right
/// should still jump to the adjacent column by picking the nearest-Y
/// candidate rather than hard-gating on row alignment.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct CrossRowHorizontalNav;

/// Marks a container whose bounds should be used (instead of the focused
/// descendant's own bounds) when auto-scrolling the focus into view. Used
/// for cards/rows where the focusable button sits near the bottom of a
/// taller visual container — without this marker, scroll would stop as soon
/// as the button is visible, leaving the card's header clipped above the
/// viewport. The autoscroll walks up from the focus and uses the nearest
/// ancestor carrying this marker (if any) as the rect to reveal.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ScrollRevealBounds;
