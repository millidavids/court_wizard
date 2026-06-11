use super::super::super::super::components::Spell;
use super::super::components::GreaseTalentParams;
use super::super::constants;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> GreaseTalentParams {
    let mut params = GreaseTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Grease, 0);
    let t2 = talents.get_selection(Spell::Grease, 1);
    let t3 = talents.get_selection(Spell::Grease, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Extra Slippery
            params.slow_mult = constants::EXTRA_SLIPPERY_SLOW_MULT;
        }
        Some(1) => {
            // Wider Slick
            params.radius_mult = constants::WIDER_SLICK_RADIUS_MULT;
        }
        Some(2) => {
            // Volatile Mixture
            params.burn_damage_mult = constants::VOLATILE_MIXTURE_BURN_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.slip_and_fall = true,
        Some(1) => params.oil_slick = true,
        Some(2) => params.lingering_flames = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.chain_combustion = true,
        Some(1) => params.grease_geyser = true,
        Some(2) => params.endless_oil = true,
        _ => {}
    }

    params
}
