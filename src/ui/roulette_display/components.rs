use bevy::prelude::*;

/// Marker for the roulette display root entity (used for cleanup on pause/unpause).
#[derive(Component)]
pub(crate) struct RouletteDisplayRoot;

/// Marker for the spinning wheel mesh/entity that rotates.
#[derive(Component)]
pub(super) struct RouletteWheelMesh;

/// Marker for the pointer/indicator at the top of the wheel.
#[derive(Component)]
pub(super) struct RoulettePointer;

/// Marker for the text showing which spell was selected above the wheel.
#[derive(Component)]
pub(super) struct RouletteSelectedText;

/// Marker for the "Press SPACE to spin" prompt text.
#[derive(Component)]
pub(super) struct RoulettePromptText;

/// Sibling image node for the spin prompt, shown (in place of the text glyph)
/// when an official Steam Input glyph is available for the spin action.
#[derive(Component)]
pub(super) struct RoulettePromptGlyphImage;

/// Timer for fading the selected spell name display.
#[derive(Component)]
pub(super) struct SelectedSpellFadeTimer {
    pub elapsed: f32,
    pub duration: f32,
}
