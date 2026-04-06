use bevy::prelude::*;

use crate::game::units::wizard::archetypes::runes::resources::Rune;

/// Marker for the root rune display container.
#[derive(Component)]
pub(super) struct RuneDisplayRoot;

/// Marker for the text showing the current rune sequence above the buttons.
#[derive(Component)]
pub(super) struct RuneSequenceText;

/// Component that marks a rune button and stores which rune it represents.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct RuneButton {
    pub(super) rune: Rune,
}

/// Marker for the activated spell name text shown above the rune display.
#[derive(Component)]
pub(super) struct ActivatedSpellText;

/// Shadow duplicate of RuneSequenceText.
#[derive(Component)]
pub(super) struct RuneSequenceTextShadow;

/// Shadow duplicate of ActivatedSpellText.
#[derive(Component)]
pub(super) struct ActivatedSpellTextShadow;

/// Timer component for spell name fade-out animation.
#[derive(Component)]
pub(super) struct SpellNameFadeTimer {
    pub(super) elapsed: f32,
    pub(super) duration: f32,
}
