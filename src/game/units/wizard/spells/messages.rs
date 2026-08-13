//! Messages broadcast by spells for other systems to react to.
//!
//! Deliberately spell-neutral. The Arcane Crystal is currently the only reader of
//! [`SpellAreaCastMessage`], but putting the message here rather than in
//! `arcane_crystal/` keeps the dependency pointing the right way: the crystal
//! module depends on other spells, not the other way round.

use bevy::prelude::*;

use super::super::components::Spell;

/// Emitted when a spell's area of effect lands on the battlefield.
///
/// Spells that produce a long-lived projectile the crystal can intercept
/// (fireball, magic missile, meteor, the beams) do **not** send this — the
/// crystal polls for those in `arcane_crystal/hits/`. This message covers the
/// spells with nothing to intercept: ground-cast zones, instant buffs, and
/// single-target casts aimed at the crystal.
#[derive(Message)]
pub(crate) struct SpellAreaCastMessage {
    /// Which spell landed.
    pub spell: Spell,
    /// Centre of the affected area in world space.
    pub position: Vec3,
    /// Radius of the affected area. `0.0` for a single-target cast that named the
    /// crystal directly, which still counts as a hit via the crystal's own
    /// collision radius.
    pub radius: f32,
    /// Caster empowerment, carried through to whatever the crystal projects.
    pub empowerment: f32,
}

/// Announces an area cast at `position`.
///
/// Goes through `Commands` rather than a `MessageWriter` parameter on purpose:
/// several casting systems already sit at Bevy's 16-parameter ceiling, and every
/// one of them already has `Commands`. Costs one frame of latency, which does not
/// matter — the crystal fires the resulting burst on its next tick either way.
pub(crate) fn announce_area_cast(
    commands: &mut Commands,
    spell: Spell,
    position: Vec3,
    radius: f32,
    empowerment: f32,
) {
    commands.write_message(SpellAreaCastMessage {
        spell,
        position,
        radius,
        empowerment,
    });
}
