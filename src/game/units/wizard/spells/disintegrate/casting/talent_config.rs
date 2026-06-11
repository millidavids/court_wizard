use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::disintegrate::constants;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Talent configuration computed once from ActiveTalents.
pub(crate) struct TalentConfig {
    pub(crate) width_multiplier: f32,
    pub(crate) damage_multiplier: f32,
    pub(crate) mana_cost_multiplier: f32,
    pub(crate) forked: bool,
    pub(crate) escalating: bool,
    pub(crate) sweeping: bool,
    pub(crate) searing_finale: bool,
    pub(crate) resonance: bool,
    pub(crate) annihilation: bool,
}

impl Default for TalentConfig {
    fn default() -> Self {
        Self {
            width_multiplier: 1.0,
            damage_multiplier: 1.0,
            mana_cost_multiplier: 1.0,
            forked: false,
            escalating: false,
            sweeping: false,
            searing_finale: false,
            resonance: false,
            annihilation: false,
        }
    }
}

pub(crate) fn compute_talent_config(active_talents: Option<&ActiveTalents>) -> TalentConfig {
    let talents = active_talents;
    let t1 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 1));
    let t3 = talents.and_then(|t| t.get_selection(Spell::Disintegrate, 2));

    let mut cfg = TalentConfig::default();

    // Tier 1
    match t1 {
        Some(0) => {
            // Focused Lens
            cfg.width_multiplier *= constants::FOCUSED_LENS_WIDTH_MULT;
            cfg.damage_multiplier *= constants::FOCUSED_LENS_DAMAGE_MULT;
        }
        Some(1) => {
            // Unfocused Beam
            cfg.width_multiplier *= constants::UNFOCUSED_BEAM_WIDTH_MULT;
            cfg.damage_multiplier *= constants::UNFOCUSED_BEAM_DAMAGE_MULT;
        }
        Some(2) => {
            // Efficient Channeling
            cfg.mana_cost_multiplier *= constants::EFFICIENT_CHANNELING_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Forked Beam
            cfg.forked = true;
            cfg.damage_multiplier *= constants::FORKED_DAMAGE_MULT;
        }
        Some(1) => {
            // Escalating Intensity
            cfg.escalating = true;
        }
        Some(2) => {
            // Sweeping Destruction (+100% damage since player loses aim control)
            cfg.sweeping = true;
            cfg.damage_multiplier *= constants::SWEEPING_DAMAGE_MULT;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Annihilation Beam
            cfg.width_multiplier *= constants::ANNIHILATION_WIDTH_MULT;
            cfg.damage_multiplier *= constants::ANNIHILATION_DAMAGE_MULT;
            cfg.mana_cost_multiplier *= constants::ANNIHILATION_MANA_MULT;
            cfg.annihilation = true;
        }
        Some(1) => {
            // Searing Finale
            cfg.searing_finale = true;
        }
        Some(2) => {
            // Unstable Resonance
            cfg.resonance = true;
        }
        _ => {}
    }

    cfg
}
