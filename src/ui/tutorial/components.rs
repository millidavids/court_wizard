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

/// Tracks the highlight overlay entity spawned as a child of a highlighted
/// element so it can be despawned cleanly. Replaces the older "mutate the
/// element's own border" approach so the highlight no longer changes the
/// element's layout or fights with the element's own focus/hover styling.
#[derive(Component)]
pub(super) struct HighlightOverlay {
    pub child: Entity,
}

/// Marker on the spawned overlay child that pulses with the gold glow.
#[derive(Component)]
pub(super) struct HighlightOverlayGlow;

/// Drives the glow animation timer.
#[derive(Component)]
pub(super) struct GlowAnimation {
    pub elapsed: f32,
}

/// Marker for the tutorial dialog panel.
#[derive(Component)]
pub(super) struct TutorialPanel;
