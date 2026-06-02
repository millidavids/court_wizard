//! Per-wizard spell-stat tally markers for the multiplayer score screen.
//!
//! `apply_spell_damage` and `apply_spell_heal` drop these one-frame markers on
//! the unit they affect, accumulating multiple same-frame hits in place via
//! `entry()` + `and_modify` (so overlapping Meteor/Squall explosions or
//! multi-bolt Chain Lightning don't overwrite each other). The multiplayer score
//! screen's `accumulate_wizard_spell_stats` sums them into the local wizard's
//! running totals and clears them every frame.
//!
//! These types live in `units` (not `multiplayer`) so the shared combat helpers
//! that emit them don't depend on the multiplayer feature module. In
//! single-player the markers are still emitted but never consumed — the
//! accumulator runs only in multiplayer — so they are inert, fixed-size, and
//! removed automatically when the unit despawns.

use bevy::prelude::*;

/// One-frame accumulator of a wizard's spell DAMAGE dealt to a unit.
#[derive(Component)]
pub(crate) struct SpellDamageTally(pub f32);

/// One-frame accumulator of a wizard's spell HEALING applied to a unit.
#[derive(Component)]
pub(crate) struct SpellHealTally(pub f32);

/// Lets the score-screen consumer drain either tally type with one generic helper.
pub(crate) trait SpellTally: Component {
    fn amount(&self) -> f32;
}

impl SpellTally for SpellDamageTally {
    fn amount(&self) -> f32 {
        self.0
    }
}

impl SpellTally for SpellHealTally {
    fn amount(&self) -> f32 {
        self.0
    }
}
