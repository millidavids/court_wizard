use bevy::prelude::*;

/// Marker for the wrapper holding the Debug Level +/- buttons. Toggled by
/// the global F2 debug-UI flag (see `crate::game::debug_ui`).
#[cfg(debug_assertions)]
#[derive(Component)]
pub(crate) struct DebugLevelButtons;

// ---------------------------------------------------------------------------
// Action enum
// ---------------------------------------------------------------------------

/// Actions for buttons within the endless tab.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EndlessAction {
    ContinuePlay,
    SwitchWizardType,
    StartTimeTravel,
    #[cfg(debug_assertions)]
    DebugIncreaseLevel,
    #[cfg(debug_assertions)]
    DebugDecreaseLevel,
}
