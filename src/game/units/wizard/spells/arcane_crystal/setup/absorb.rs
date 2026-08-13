//! Shared absorption bookkeeping for every spell a crystal can take in.
//!
//! Each detector in `hits/` used to hand-roll the same preamble (pulse, set the
//! remembered spell, reset the auto-cast timer, roll Spell Echo, scale the
//! emission count, credit talent progress, tick resonance). Centralising it here
//! keeps the five — soon thirty — absorption paths from drifting apart.

use bevy::prelude::*;
use rand::Rng;

use super::super::components::{ArcaneCrystal, CrystalInfusion, ResonanceCascade};
use super::helpers::{increment_resonance, scaled_count, spell_echo_multiplier};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;

/// What an absorption worked out to, for the caller to emit with.
pub(crate) struct AbsorptionResult {
    /// Number of sub-spells to emit, already scaled by count multiplier and echo.
    pub count: usize,
}

/// Which bookkeeping a given absorption performs.
///
/// The distinction matters because Disintegrate re-asserts itself **every frame**
/// while the beam is channeled. Counting that path would inflate talent progress
/// and drive Resonance Cascade off a frame timer, and — worse — rolling Spell
/// Echo would draw from [`GameRng`] once per frame, shifting the seeded stream
/// and breaking run-for-run determinism.
///
/// [`GameRng`]: crate::game::seeded_rng::resources::GameRng
#[derive(Clone, Copy)]
pub(crate) struct AbsorptionBookkeeping {
    pub apply_echo: bool,
    pub count_resonance: bool,
    pub count_progress: bool,
}

impl AbsorptionBookkeeping {
    /// A single discrete absorption: rolls Echo, credits progress, ticks resonance.
    pub const DISCRETE: Self = Self {
        apply_echo: true,
        count_resonance: true,
        count_progress: true,
    };

    /// A channeled beam re-asserting itself this frame. Draws no randomness and
    /// counts nothing.
    pub const CHANNELED: Self = Self {
        apply_echo: false,
        count_resonance: false,
        count_progress: false,
    };
}

/// Records an absorption on `crystal` and returns the emission count.
///
/// Returns `None` when the crystal refuses the absorption — currently only
/// Auto-Crystal turrets, which fire their own missiles and never take spells in.
/// Callers that need to skip more than the emission (Disintegrate also skips its
/// beam maintenance) should keep their own guard as well.
#[allow(clippy::too_many_arguments)]
pub(crate) fn absorb_into_crystal(
    commands: &mut Commands,
    crystal: &mut ArcaneCrystal,
    resonance: &mut Option<Mut<ResonanceCascade>>,
    progress: &mut BattleTalentProgress,
    rng: &mut impl Rng,
    infusion: CrystalInfusion,
    base_count: usize,
    bookkeeping: AbsorptionBookkeeping,
) -> Option<AbsorptionResult> {
    if crystal.permanent {
        return None;
    }

    // Re-infusing replaces what the crystal projects, so anything the previous
    // infusion left standing has to go with it.
    if crystal.infusion != Some(infusion) {
        crystal.clear_infusion_spawns(commands);
    }

    crystal.mark_absorption();
    crystal.infusion = Some(infusion);
    crystal.auto_cast_timer = 0.0;

    // Roll Echo before the caller draws for target selection — that ordering is
    // what the pre-refactor detectors used, and the seeded RNG stream depends on it.
    let echo_mult = if bookkeeping.apply_echo {
        spell_echo_multiplier(rng, crystal.spell_echo)
    } else {
        1
    };
    let count = scaled_count(base_count, crystal.count_mult) * echo_mult;
    debug_assert!(echo_mult == 1 || bookkeeping.apply_echo);

    if bookkeeping.count_progress {
        progress.increment(Spell::ArcaneCrystal, count as u32);
    }
    if bookkeeping.count_resonance {
        increment_resonance(resonance);
    }

    Some(AbsorptionResult { count })
}
