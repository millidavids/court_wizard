//! Talent configuration derivation for chain lightning.

use super::super::super::super::components::Spell;
use super::super::constants;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Computed talent configuration for chain lightning, derived from ActiveTalents.
pub(super) struct ChainLightningTalentConfig {
    pub(super) bounce_range_mult: f32,
    pub(super) initial_damage_mult: f32,
    pub(super) damage_falloff: f32,
    pub(super) static_charge: bool,
    pub(super) split_count: usize,
    pub(super) max_bounces: u32,
    pub(super) magnetic_pull: bool,
    pub(super) thunderstorm_count: u32,
    pub(super) mana_cost_mult: f32,
    pub(super) chain_reaction: bool,
}

pub(super) fn compute_chain_lightning_talent_config(
    active_talents: Option<&ActiveTalents>,
) -> ChainLightningTalentConfig {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::ChainLightning, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::ChainLightning, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::ChainLightning, 2));

    // Tier 1 defaults
    let mut bounce_range_mult = 1.0;
    let mut initial_damage_mult = 1.0;
    let mut damage_falloff = constants::DAMAGE_FALLOFF;
    let mut static_charge = false;

    match t1 {
        Some(0) => {
            bounce_range_mult = constants::CONDUCTING_BOLTS_RANGE_MULT;
            initial_damage_mult = constants::CONDUCTING_BOLTS_DAMAGE_MULT;
        }
        Some(1) => {
            initial_damage_mult = constants::HIGH_VOLTAGE_DAMAGE_MULT;
            damage_falloff = constants::HIGH_VOLTAGE_FALLOFF;
        }
        Some(2) => static_charge = true,
        _ => {}
    }

    // Tier 2 defaults
    let mut split_count = constants::SPLIT_COUNT;
    let mut max_bounces = constants::MAX_BOUNCES;
    let mut magnetic_pull = false;

    match t2 {
        Some(0) => split_count = constants::FORKED_SPLIT_COUNT,
        Some(1) => {
            // Overcharge: no damage falloff, fewer splits, fewer bounces
            damage_falloff = constants::OVERCHARGE_FALLOFF;
            split_count = constants::OVERCHARGE_SPLIT_COUNT;
            max_bounces = constants::OVERCHARGE_MAX_BOUNCES;
        }
        Some(2) => magnetic_pull = true,
        _ => {}
    }

    // Tier 3 defaults
    let mut thunderstorm_count = 1;
    let mut mana_cost_mult = 1.0;
    let mut chain_reaction = false;

    match t3 {
        Some(0) => {
            thunderstorm_count = constants::THUNDERSTORM_CAST_COUNT;
            mana_cost_mult = constants::THUNDERSTORM_MANA_MULT;
        }
        Some(1) => chain_reaction = true,
        Some(2) => {
            max_bounces = constants::LIVING_LIGHTNING_MAX_BOUNCES;
            mana_cost_mult = constants::LIVING_LIGHTNING_MANA_MULT;
        }
        _ => {}
    }

    ChainLightningTalentConfig {
        bounce_range_mult,
        initial_damage_mult,
        damage_falloff,
        static_charge,
        split_count,
        max_bounces,
        magnetic_pull,
        thunderstorm_count,
        mana_cost_mult,
        chain_reaction,
    }
}
