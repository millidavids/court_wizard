//! Sleep spell components.

/// Talent parameters computed from active talent selections.
/// Passed through casting to configure the SleepModifier applied to targets.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SleepTalentParams {
    // Tier 1: numeric modifiers
    pub duration_mult: f32,
    pub radius_mult: f32,
    pub bonus_damage_mult: f32,
    // Tier 2: behavioral flags
    pub narcoleptic_wave: bool,
    pub night_terrors: bool,
    pub cast_time_mult: f32,
    pub mana_mult: f32,
    // Tier 3: transformative flags
    pub comatose: bool,
    pub dreamwalker: bool,
    pub eternal_slumber: bool,
}

impl Default for SleepTalentParams {
    fn default() -> Self {
        Self {
            duration_mult: 1.0,
            radius_mult: 1.0,
            bonus_damage_mult: 1.0,
            narcoleptic_wave: false,
            night_terrors: false,
            cast_time_mult: 1.0,
            mana_mult: 1.0,
            comatose: false,
            dreamwalker: false,
            eternal_slumber: false,
        }
    }
}
