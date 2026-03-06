//! Tutorial UI components.

use bevy::prelude::*;

use super::definitions::HighlightTarget;

/// Marker for tutorial-highlightable UI elements.
#[derive(Component)]
pub struct TutorialHighlightable {
    pub target: HighlightTarget,
}

/// Marker for the tutorial overlay root entity.
#[derive(Component)]
pub(super) struct TutorialOverlay;

/// Marker for the tutorial text display.
#[derive(Component)]
pub(super) struct TutorialText;

/// Marker for the step counter text.
#[derive(Component)]
pub(super) struct TutorialStepCounter;

/// Marker for the "Next" / "Got it" button.
#[derive(Component)]
pub(super) struct TutorialNextButton;

/// Marker for the "Skip Tutorial" button.
#[derive(Component)]
pub(super) struct TutorialSkipButton;

/// Tracks the original border state of a highlighted entity so we can restore it.
#[derive(Component)]
pub(super) struct OriginalBorder {
    pub color: Color,
    pub width: UiRect,
}

/// Drives the glow animation timer.
#[derive(Component)]
pub(super) struct GlowAnimation {
    pub elapsed: f32,
}

/// Marker for the tutorial dialog panel.
#[derive(Component)]
pub(super) struct TutorialPanel;
