//! Focus navigation tuning constants.

use std::time::Duration;

/// Digital-direction threshold for left-stick navigation (stick must exceed this
/// magnitude along a single axis to register as a direction press).
pub(super) const STICK_DIRECTION_THRESHOLD: f32 = 0.55;

/// Delay between the first focus move and the start of hold-to-repeat.
/// Holding a D-pad direction or stick past this duration starts stepping
/// through focusables automatically.
pub(super) const FOCUS_REPEAT_INITIAL_DELAY: Duration = Duration::from_millis(550);

/// Interval between consecutive focus moves while a direction is held past
/// the initial delay (~4 moves per second — deliberate, not runaway).
pub(super) const FOCUS_REPEAT_INTERVAL: Duration = Duration::from_millis(260);

/// Return-to-center threshold (stick must fall below this before another
/// direction press can register, preventing runaway cursoring).
pub(super) const STICK_RESET_THRESHOLD: f32 = 0.25;

/// Two candidates within this many pixels of each other along the navigation
/// direction are considered to be in the same row/column. Pick by perpendicular
/// proximity within the group; otherwise prefer the group with the smallest
/// forward distance. This makes down-press go to the nearest row even if that
/// row has no item directly below the current focus.
pub(super) const SAME_ROW_TOLERANCE: f32 = 30.0;

/// Maximum horizontal distance between two candidates considered to be in the
/// same "panel column" for vertical navigation. When pressing Up/Down, items
/// within this X-range of the current focus are preferred over items further
/// away — so D-pad-Down in the left panel moves to the next left-panel item
/// instead of hopping to a same-Y item in the right panel.
pub(super) const PANEL_COLUMN_TOLERANCE: f32 = 200.0;

/// Pixels/second the right stick scrolls a `GamepadScrollTarget` at full deflection.
pub(super) const RIGHT_STICK_SCROLL_SPEED: f32 = 1200.0;

/// Extra padding kept between the focused item and the scrollable container's
/// edge when auto-scrolling. Prevents the focused button from sitting flush
/// against the clipping edge.
pub(super) const AUTOSCROLL_EDGE_PADDING: f32 = 24.0;

/// Minimum shaped right-stick magnitude required for `right_stick_scroll` to
/// claim control of an in-flight `ScrollAnimation`. Just past the 0.15
/// deadzone the response curve produces near-zero shaped values; a thumb
/// resting at that boundary would otherwise strip the autoscroll snap every
/// frame without producing any visible scroll movement.
pub(super) const ANIM_CANCEL_THRESHOLD: f32 = 0.1;
