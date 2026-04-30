use bevy::prelude::*;

/// A single effect that an ingredient/brew applies when active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrewEffect {
    /// Multiplies wizard mana regeneration rate.
    ManaRegenMultiplier(f32),
    /// Multiplies spell power/empowerment.
    SpellPowerMultiplier(f32),
    /// Heals all defender units by this amount per second.
    DefenderHealPerSecond(f32),
    /// Multiplies wizard cast speed (higher = faster).
    CastSpeedMultiplier(f32),
    /// Multiplies spell area of effect.
    SpellRangeMultiplier(f32),
    /// Bonus damage percentage for defender units (0.5 = +50%).
    DefenderDamageBonus(f32),
    /// Damage resistance percentage for defender units (0.25 = 25% reduction).
    DamageResistancePercent(f32),
    /// Speed bonus percentage for defender units (0.3 = +30%).
    DefenderSpeedBonus(f32),
    /// Speed reduction percentage for attacker/undead units (0.25 = 25% slow).
    AttackerSlowPercent(f32),
    /// Grants temporary hit points to defenders per second.
    DefenderShieldPerSecond(f32),
    /// Multiplies wizard maximum mana capacity.
    MaxManaMultiplier(f32),
    /// Multiplies defender attack speed (higher = faster).
    AttackSpeedMultiplier(f32),
    /// Multiplies buff duration from brews.
    BuffDurationMultiplier(f32),
    /// Flat bonus to defender effectiveness.
    EffectivenessBonus(f32),
}

impl BrewEffect {
    /// Returns true if this effect does nothing (multiplier is 1.0 or flat value is 0.0).
    pub fn is_noop(&self) -> bool {
        match self {
            BrewEffect::ManaRegenMultiplier(v)
            | BrewEffect::SpellPowerMultiplier(v)
            | BrewEffect::CastSpeedMultiplier(v)
            | BrewEffect::SpellRangeMultiplier(v)
            | BrewEffect::MaxManaMultiplier(v)
            | BrewEffect::AttackSpeedMultiplier(v)
            | BrewEffect::BuffDurationMultiplier(v) => (*v - 1.0).abs() < f32::EPSILON,
            BrewEffect::DefenderHealPerSecond(v)
            | BrewEffect::DefenderDamageBonus(v)
            | BrewEffect::DamageResistancePercent(v)
            | BrewEffect::DefenderSpeedBonus(v)
            | BrewEffect::AttackerSlowPercent(v)
            | BrewEffect::DefenderShieldPerSecond(v)
            | BrewEffect::EffectivenessBonus(v) => v.abs() < f32::EPSILON,
        }
    }

    /// Returns a human-readable description of this effect, e.g. "Mana Regen +100%".
    pub fn display_text(&self) -> String {
        match self {
            BrewEffect::ManaRegenMultiplier(v) => format!("Mana regen: +{:.0}%", (v - 1.0) * 100.0),
            BrewEffect::SpellPowerMultiplier(v) => {
                format!("Spell power: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::DefenderHealPerSecond(v) => format!("Defender healing: {:.1} HP/s", v),
            BrewEffect::CastSpeedMultiplier(v) => {
                format!("Cast speed: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::SpellRangeMultiplier(v) => {
                format!("Spell area: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::DefenderDamageBonus(v) => format!("Defender damage: +{:.0}%", v * 100.0),
            BrewEffect::DamageResistancePercent(v) => {
                format!("Damage resistance: {:.0}%", v * 100.0)
            }
            BrewEffect::DefenderSpeedBonus(v) => format!("Defender speed: +{:.0}%", v * 100.0),
            BrewEffect::AttackerSlowPercent(v) => format!("Enemy slow: {:.0}%", v * 100.0),
            BrewEffect::DefenderShieldPerSecond(v) => format!("Defender shield: {:.1} HP/s", v),
            BrewEffect::MaxManaMultiplier(v) => format!("Max mana: +{:.0}%", (v - 1.0) * 100.0),
            BrewEffect::AttackSpeedMultiplier(v) => {
                format!("Attack speed: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::BuffDurationMultiplier(v) => {
                format!("Buff duration: +{:.0}%", (v - 1.0) * 100.0)
            }
            BrewEffect::EffectivenessBonus(v) => {
                format!("Defender effectiveness: +{:.0}%", v * 100.0)
            }
        }
    }

    /// Returns a short 1-2 character abbreviation for use in buff tracker boxes.
    pub const fn abbreviation(&self) -> &'static str {
        match self {
            BrewEffect::ManaRegenMultiplier(_) => "MR",
            BrewEffect::SpellPowerMultiplier(_) => "SP",
            BrewEffect::DefenderHealPerSecond(_) => "HL",
            BrewEffect::CastSpeedMultiplier(_) => "CS",
            BrewEffect::SpellRangeMultiplier(_) => "AE",
            BrewEffect::DefenderDamageBonus(_) => "DD",
            BrewEffect::DamageResistancePercent(_) => "DR",
            BrewEffect::DefenderSpeedBonus(_) => "DS",
            BrewEffect::AttackerSlowPercent(_) => "SL",
            BrewEffect::DefenderShieldPerSecond(_) => "SH",
            BrewEffect::MaxManaMultiplier(_) => "MM",
            BrewEffect::AttackSpeedMultiplier(_) => "AS",
            BrewEffect::BuffDurationMultiplier(_) => "BD",
            BrewEffect::EffectivenessBonus(_) => "EF",
        }
    }

    /// Scales an effect's magnitude by a factor.
    /// Multiplier effects (base 1.0): 1.0 + (value - 1.0) * factor
    /// Flat effects (base 0.0): value * factor
    pub fn scale(self, factor: f32) -> BrewEffect {
        match self {
            BrewEffect::ManaRegenMultiplier(v) => {
                BrewEffect::ManaRegenMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::SpellPowerMultiplier(v) => {
                BrewEffect::SpellPowerMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::CastSpeedMultiplier(v) => {
                BrewEffect::CastSpeedMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::SpellRangeMultiplier(v) => {
                BrewEffect::SpellRangeMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::MaxManaMultiplier(v) => {
                BrewEffect::MaxManaMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::AttackSpeedMultiplier(v) => {
                BrewEffect::AttackSpeedMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::BuffDurationMultiplier(v) => {
                BrewEffect::BuffDurationMultiplier(1.0 + (v - 1.0) * factor)
            }
            BrewEffect::DefenderHealPerSecond(v) => BrewEffect::DefenderHealPerSecond(v * factor),
            BrewEffect::DefenderDamageBonus(v) => BrewEffect::DefenderDamageBonus(v * factor),
            BrewEffect::DamageResistancePercent(v) => {
                BrewEffect::DamageResistancePercent(v * factor)
            }
            BrewEffect::DefenderSpeedBonus(v) => BrewEffect::DefenderSpeedBonus(v * factor),
            BrewEffect::AttackerSlowPercent(v) => BrewEffect::AttackerSlowPercent(v * factor),
            BrewEffect::DefenderShieldPerSecond(v) => {
                BrewEffect::DefenderShieldPerSecond(v * factor)
            }
            BrewEffect::EffectivenessBonus(v) => BrewEffect::EffectivenessBonus(v * factor),
        }
    }
}
