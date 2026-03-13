use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_SLEEP: PrimedSpell = PrimedSpell {
    spell: Spell::Sleep,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.5;
pub const MANA_COST: f32 = 35.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const SLEEP_DURATION: f32 = 8.0;
pub const BONUS_DAMAGE_MULTIPLIER: f32 = 2.0;

// --- Talent constants ---

// Tier 1
/// Deep Slumber: +40% sleep duration.
pub(super) const DEEP_SLUMBER_DURATION_MULT: f32 = 1.4;
/// Lullaby: +40% circle radius.
pub(super) const LULLABY_RADIUS_MULT: f32 = 1.4;
/// Nightmare Fuel: +50% wake-up bonus damage multiplier (applied to base 2.0x).
pub(super) const NIGHTMARE_FUEL_BONUS_MULT: f32 = 1.5;

// Tier 2
/// Narcoleptic Wave: sleep spreads after this many seconds.
pub(super) const NARCOLEPTIC_SPREAD_DELAY: f32 = 3.0;
/// Narcoleptic Wave: radius to spread sleep to nearby enemies.
pub(super) const NARCOLEPTIC_SPREAD_RADIUS: f32 = 60.0;
/// Night Terrors: damage per second while sleeping (not enough to wake with Comatose).
pub(super) const NIGHT_TERRORS_DPS: f32 = 2.0;
/// Drowsy: cast time multiplier (halved).
pub(super) const DROWSY_CAST_TIME_MULT: f32 = 0.5;
/// Drowsy: mana cost multiplier (-25%).
pub(super) const DROWSY_MANA_MULT: f32 = 0.75;

// Tier 3
/// Comatose: damage must exceed this fraction of max HP to wake.
pub(super) const COMATOSE_WAKE_THRESHOLD: f32 = 0.3;
/// Dreamwalker: duration override (sleepwalking lasts much longer).
pub(super) const DREAMWALKER_DURATION: f32 = 30.0;
/// Dreamwalker: speed multiplier for sleepwalking units (fraction of normal speed).
pub(super) const DREAMWALKER_SPEED_MULT: f32 = 0.5;
/// Eternal Slumber: enemies below this HP fraction when they fall asleep are killed instantly.
pub(super) const ETERNAL_SLUMBER_HP_THRESHOLD: f32 = 0.25;
