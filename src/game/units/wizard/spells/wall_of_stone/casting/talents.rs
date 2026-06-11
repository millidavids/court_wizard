use super::super::super::super::components::Spell;
use super::super::components::WallOfStoneTalentParams;
use super::super::constants::*;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> WallOfStoneTalentParams {
    let mut params = WallOfStoneTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::WallOfStone, 0);
    let t2 = talents.get_selection(Spell::WallOfStone, 1);
    let t3 = talents.get_selection(Spell::WallOfStone, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Quarry Master
            params.mana_mult = QUARRY_MASTER_MANA_MULT;
            params.max_length_mult = QUARRY_MASTER_LENGTH_MULT;
        }
        Some(1) => {
            // Reinforced Stone
            params.health_mult = REINFORCED_STONE_HEALTH_MULT;
            params.width_mult = REINFORCED_STONE_WIDTH_MULT;
        }
        Some(2) => {
            // Quick Foundations
            params.quick_foundations = true;
            params.mana_mult = QUICK_FOUNDATIONS_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.jagged_stone = true,
        Some(1) => params.permafrost_aura = true,
        Some(2) => params.living_stone = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.collapsing_wall = true,
        Some(1) => {
            params.terraformer = true;
        }
        Some(2) => {
            params.maze_architect = true;
            params.mana_mult *= MAZE_ARCHITECT_MANA_MULT;
        }
        _ => {}
    }

    params
}
