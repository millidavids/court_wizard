//! Black hole talent parameter computation.

use super::super::components::BlackHoleTalentParams;
use super::super::constants::*;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Compute talent parameters from active talent selections.
pub(super) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> BlackHoleTalentParams {
    let mut params = BlackHoleTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::BlackHole, 0);
    let t2 = talents.get_selection(Spell::BlackHole, 1);
    let t3 = talents.get_selection(Spell::BlackHole, 2);

    // Tier 1
    match t1 {
        Some(0) => params.gravity_mult = DENSER_CORE_GRAVITY_MULT,
        Some(1) => {
            params.radius_mult = EXPANSIVE_VOID_RADIUS_MULT;
            params.damage_mult = EXPANSIVE_VOID_DAMAGE_MULT;
        }
        // Some(2) Quick Collapse: handled at cast time, not stored in params
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => params.event_horizon = true,
        Some(1) => params.crushing_pressure = true,
        Some(2) => params.void_siphon = true,
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => params.singularity = true,
        // Some(1) Twin Stars: handled at spawn time
        Some(2) => params.dimensional_rift = true,
        _ => {}
    }

    params
}
