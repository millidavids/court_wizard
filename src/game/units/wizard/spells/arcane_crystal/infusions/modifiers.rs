//! Crystal modifiers — effects that change how the crystal itself behaves,
//! layered on top of whatever it is infused with.
//!
//! These are marker components rather than an enum field because each is read by a
//! different system with a different filter: `Anchored` by the black-hole
//! interaction, `Warded` by the destruction paths, `Enraged` by lifetime accrual,
//! `Hastened` by auto-cast cadence. `Warded` is also stateful — it holds a
//! consumable charge — which a fieldless enum could not express.

use bevy::prelude::*;

use super::super::components::ArcaneCrystal;
use super::super::constants::*;

/// Haste: the crystal's auto-cast fires faster. Stacks with any infusion — this is
/// the one modifier that deliberately does not replace what the crystal projects.
#[derive(Component)]
pub(crate) struct CrystalHastened;

/// Berserker Rage: emissions hit harder, but the crystal burns its lifetime faster.
#[derive(Component)]
pub(crate) struct CrystalEnraged;

/// Wall of Stone: the crystal is rooted in place and ignores black-hole gravity.
#[derive(Component)]
pub(crate) struct CrystalAnchored;

/// Guardian Circle: absorbs destruction attempts (dispel, black-hole consumption)
/// until its charges run out.
#[derive(Component)]
pub(crate) struct CrystalWarded {
    pub charges: u32,
}

impl Default for CrystalWarded {
    fn default() -> Self {
        Self {
            charges: WARDED_CHARGES,
        }
    }
}

impl CrystalWarded {
    /// Spends a charge. Returns `true` if the ward absorbed the hit.
    pub(crate) fn absorb(&mut self) -> bool {
        if self.charges == 0 {
            return false;
        }
        self.charges -= 1;
        true
    }
}

/// Every modifier a crystal can carry. Used to clear the previous modifier when a
/// new one is applied — they are mutually exclusive with each other, but not with
/// an infusion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CrystalModifier {
    Hastened,
    Enraged,
    Anchored,
    Warded,
}

/// Replaces whatever modifier the crystal currently carries with `modifier`.
pub(crate) fn apply_modifier(
    commands: &mut Commands,
    crystal_entity: Entity,
    modifier: CrystalModifier,
) {
    let mut entity = commands.entity(crystal_entity);
    entity.remove::<(
        CrystalHastened,
        CrystalEnraged,
        CrystalAnchored,
        CrystalWarded,
    )>();
    match modifier {
        CrystalModifier::Hastened => {
            entity.insert(CrystalHastened);
        }
        CrystalModifier::Enraged => {
            entity.insert(CrystalEnraged);
        }
        CrystalModifier::Anchored => {
            entity.insert(CrystalAnchored);
        }
        CrystalModifier::Warded => {
            entity.insert(CrystalWarded::default());
        }
    }
}

/// Burns extra lifetime on enraged crystals — the cost side of the Rage trade.
///
/// The base tick already happens in `update_crystal_visuals`; this adds the
/// surplus so the crystal ages at `ENRAGED_LIFETIME_SCALE` overall.
pub(crate) fn tick_enraged_lifetime(
    time: Res<Time>,
    mut crystals: Query<
        &mut ArcaneCrystal,
        (
            With<CrystalEnraged>,
            Without<crate::game::multiplayer::components::GhostSpellEffect>,
        ),
    >,
) {
    let surplus = time.delta_secs() * (ENRAGED_LIFETIME_SCALE - 1.0);
    for mut crystal in &mut crystals {
        if crystal.permanent {
            continue;
        }
        crystal.time_alive += surplus;
    }
}
