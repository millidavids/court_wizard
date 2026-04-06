//! Permanent insight bonus system — endgame stat upgrades purchasable with Arcane Insight.

use bevy::prelude::*;

use crate::config::save_data::get_all_insight_bonuses;

/// The four wizard stats that can be permanently upgraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InsightBonusStat {
    SpellDamage,
    SpellRange,
    CastSpeed,
    ManaCost,
}

impl InsightBonusStat {
    /// Save-data key for this stat.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::SpellDamage => "spell_damage",
            Self::SpellRange => "spell_range",
            Self::CastSpeed => "cast_speed",
            Self::ManaCost => "mana_cost",
        }
    }

    /// Human-readable name shown in the UI.
    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::SpellDamage => "Spell Damage",
            Self::SpellRange => "Spell Range",
            Self::CastSpeed => "Cast Speed",
            Self::ManaCost => "Mana Cost",
        }
    }

    /// Description shown in the detail panel.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::SpellDamage => "Permanently increases all spell damage.",
            Self::SpellRange => "Permanently extends spell casting range.",
            Self::CastSpeed => "Permanently increases spell casting speed.",
            Self::ManaCost => "Permanently reduces mana cost of all spells.",
        }
    }

    /// Maximum upgrade level.
    pub(crate) const fn max_level() -> u8 {
        5
    }

    /// Bonus per level (1%).
    pub(crate) const fn bonus_per_level() -> f32 {
        0.01
    }

    /// Cost per level of insight bonus.
    pub(crate) const fn cost_per_level() -> u32 {
        500
    }

    /// Total insight needed to fully upgrade one stat from level 0 to max.
    pub(crate) fn total_cost() -> u32 {
        Self::cost_per_level() * Self::max_level() as u32
    }

    /// All four stats in display order.
    #[allow(dead_code)]
    pub(crate) const fn all() -> &'static [InsightBonusStat] {
        &[
            Self::SpellDamage,
            Self::SpellRange,
            Self::CastSpeed,
            Self::ManaCost,
        ]
    }

    /// Current level from save data.
    pub(crate) fn current_level(self) -> u8 {
        get_all_insight_bonuses()
            .get(self.id())
            .copied()
            .unwrap_or(0)
            .min(Self::max_level())
    }
}

/// Runtime resource holding computed multipliers from permanent insight bonuses.
///
/// Loaded from save data at game start and applied to the `Wizard` component.
#[derive(Resource)]
pub(crate) struct InsightBonuses {
    /// Spell damage multiplier (1.0 + 0.01 * level).
    pub spell_damage_mult: f32,
    /// Spell range multiplier (1.0 + 0.01 * level).
    pub spell_range_mult: f32,
    /// Cast speed multiplier (1.0 + 0.01 * level).
    pub cast_speed_mult: f32,
    /// Mana cost multiplier (1.0 - 0.01 * level).
    pub mana_cost_mult: f32,
}

impl Default for InsightBonuses {
    fn default() -> Self {
        Self {
            spell_damage_mult: 1.0,
            spell_range_mult: 1.0,
            cast_speed_mult: 1.0,
            mana_cost_mult: 1.0,
        }
    }
}

impl InsightBonuses {
    /// Load bonus levels from save data and compute multipliers.
    pub(crate) fn from_save() -> Self {
        let bonuses = get_all_insight_bonuses();
        let bonus = InsightBonusStat::bonus_per_level();
        let max = InsightBonusStat::max_level();

        let level = |id: &str| -> f32 {
            bonuses.get(id).copied().unwrap_or(0).min(max) as f32
        };

        Self {
            spell_damage_mult: 1.0 + bonus * level(InsightBonusStat::SpellDamage.id()),
            spell_range_mult: 1.0 + bonus * level(InsightBonusStat::SpellRange.id()),
            cast_speed_mult: 1.0 + bonus * level(InsightBonusStat::CastSpeed.id()),
            mana_cost_mult: 1.0 - bonus * level(InsightBonusStat::ManaCost.id()),
        }
    }
}
