use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Duration multiplier applied by the Scorched Earth toggle.
const SCORCHED_EARTH_DURATION_MULT: f32 = 3.0;

/// Returns the Scorched Earth duration multiplier from optional toggles.
pub(crate) fn scorched_earth_mult(toggles: Option<&ActiveToggles>) -> f32 {
    toggles.map_or(1.0, |t| t.scorched_earth_mult())
}

/// Toggleable run modifiers unlocked with Insight.
/// Unlike slider modifiers which scale values, toggles change game rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum ToggleModifier {
    /// No passive mana regen; kills restore mana instead.
    ManaDrought,
    /// Every 3rd wave includes a boss (cycling Hag → Ogre → Lich).
    BossParade,
    /// Game keeps running while in spell book / cauldron menu.
    Urgent,
    /// First wave of each level has a temporary damage shield.
    FortifiedHorde,
    /// Defenders deal 2x damage but have 50% health.
    GlassCannon,
    /// No spell can be cast twice in a row.
    SpellRotation,
    /// Wizard archetype cycles through unlocked types on a timer.
    WizardCycle,
    /// Ground-effect spells last 3x longer.
    ScorchedEarth,
    /// Half defenders, but each has 2.5x health and 1.5x damage.
    VeteranDefenders,
    /// Dead defenders don't respawn between roguelite levels.
    Attrition,
    /// Each wave spawns 25% more enemies than the last.
    RisingTide,
}

impl ToggleModifier {
    /// All toggle modifiers in display order.
    pub const fn all() -> &'static [ToggleModifier] {
        &[
            ToggleModifier::ManaDrought,
            ToggleModifier::BossParade,
            ToggleModifier::Urgent,
            ToggleModifier::FortifiedHorde,
            ToggleModifier::GlassCannon,
            ToggleModifier::SpellRotation,
            ToggleModifier::WizardCycle,
            ToggleModifier::ScorchedEarth,
            ToggleModifier::VeteranDefenders,
            ToggleModifier::Attrition,
            ToggleModifier::RisingTide,
        ]
    }

    /// Display name shown in the UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            ToggleModifier::ManaDrought => "Blood Magic",
            ToggleModifier::BossParade => "Boss Parade",
            ToggleModifier::Urgent => "Urgent",
            ToggleModifier::FortifiedHorde => "Fortified Horde",
            ToggleModifier::GlassCannon => "Glass Cannon",
            ToggleModifier::SpellRotation => "Spell Rotation",
            ToggleModifier::WizardCycle => "Wizard Cycle",
            ToggleModifier::ScorchedEarth => "Scorched Earth",
            ToggleModifier::VeteranDefenders => "Veteran Defenders",
            ToggleModifier::Attrition => "Attrition",
            ToggleModifier::RisingTide => "Rising Tide",
        }
    }

    /// Description shown in tooltip/hover.
    #[allow(dead_code)]
    pub const fn description(&self) -> &'static str {
        match self {
            ToggleModifier::ManaDrought => "No passive mana regen. Kills restore mana instead.",
            ToggleModifier::BossParade => "Every 3rd wave includes a boss unit.",
            ToggleModifier::Urgent => "Game keeps running while browsing menus.",
            ToggleModifier::FortifiedHorde => "First wave arrives with damage shields.",
            ToggleModifier::GlassCannon => "Defenders deal 2x damage but have half health.",
            ToggleModifier::SpellRotation => "Can't cast the same spell twice in a row.",
            ToggleModifier::WizardCycle => "Wizard type cycles through your unlocked archetypes.",
            ToggleModifier::ScorchedEarth => "Ground-effect spells last 3x longer.",
            ToggleModifier::VeteranDefenders => "Half as many defenders, but much tougher.",
            ToggleModifier::Attrition => "Dead defenders don't come back next level.",
            ToggleModifier::RisingTide => "Each wave has 25% more enemies than the last.",
        }
    }

    /// Insight cost to permanently unlock this toggle.
    pub const fn insight_cost(&self) -> u32 {
        match self {
            ToggleModifier::ManaDrought => 60,
            ToggleModifier::BossParade => 70,
            ToggleModifier::Urgent => 30,
            ToggleModifier::FortifiedHorde => 50,
            ToggleModifier::GlassCannon => 40,
            ToggleModifier::SpellRotation => 45,
            ToggleModifier::WizardCycle => 55,
            ToggleModifier::ScorchedEarth => 45,
            ToggleModifier::VeteranDefenders => 55,
            ToggleModifier::Attrition => 60,
            ToggleModifier::RisingTide => 35,
        }
    }

    /// Bonus Insight percentage granted at battle end when this toggle is active.
    pub const fn insight_bonus_percent(&self) -> u32 {
        match self {
            ToggleModifier::ManaDrought => 20,
            ToggleModifier::BossParade => 25,
            ToggleModifier::Urgent => 10,
            ToggleModifier::FortifiedHorde => 15,
            ToggleModifier::GlassCannon => 15,
            ToggleModifier::SpellRotation => 15,
            ToggleModifier::WizardCycle => 20,
            ToggleModifier::ScorchedEarth => 10,
            ToggleModifier::VeteranDefenders => 20,
            ToggleModifier::Attrition => 25,
            ToggleModifier::RisingTide => 10,
        }
    }

    /// Stable string ID used for persistence (never changes even if enum is renamed).
    pub const fn id(&self) -> &'static str {
        match self {
            ToggleModifier::ManaDrought => "mana_drought",
            ToggleModifier::BossParade => "boss_parade",
            ToggleModifier::Urgent => "urgent",
            ToggleModifier::FortifiedHorde => "fortified_horde",
            ToggleModifier::GlassCannon => "glass_cannon",
            ToggleModifier::SpellRotation => "spell_rotation",
            ToggleModifier::WizardCycle => "wizard_cycle",
            ToggleModifier::ScorchedEarth => "scorched_earth",
            ToggleModifier::VeteranDefenders => "veteran_defenders",
            ToggleModifier::Attrition => "attrition",
            ToggleModifier::RisingTide => "rising_tide",
        }
    }

    /// Look up a toggle modifier by its stable string ID.
    #[allow(dead_code)]
    pub fn from_id(id: &str) -> Option<ToggleModifier> {
        ToggleModifier::all().iter().find(|t| t.id() == id).copied()
    }
}

/// Active toggle modifiers for the current run.
/// Inserted alongside `RogueliteModifiers` when a run starts.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ActiveToggles {
    toggles: Vec<ToggleModifier>,
}

impl ActiveToggles {
    pub fn new(toggles: Vec<ToggleModifier>) -> Self {
        Self { toggles }
    }

    /// Check if a specific toggle is active for this run.
    pub fn is_active(&self, toggle: ToggleModifier) -> bool {
        self.toggles.contains(&toggle)
    }

    /// Returns the Scorched Earth duration multiplier if active.
    pub fn scorched_earth_mult(&self) -> f32 {
        if self.is_active(ToggleModifier::ScorchedEarth) {
            SCORCHED_EARTH_DURATION_MULT
        } else {
            1.0
        }
    }

    /// All active toggles.
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &ToggleModifier> {
        self.toggles.iter()
    }

    /// Number of active toggles.
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.toggles.len()
    }

    /// Total Insight bonus percentage from all active toggles.
    pub fn total_insight_bonus_percent(&self) -> u32 {
        self.toggles.iter().map(|t| t.insight_bonus_percent()).sum()
    }

    /// Serializes active toggles as a list of string IDs for save data.
    pub fn to_ids(&self) -> Vec<String> {
        self.toggles.iter().map(|t| t.id().to_string()).collect()
    }

    /// Deserializes from a list of string IDs. Unknown IDs are silently skipped.
    #[allow(dead_code)]
    pub fn from_ids(ids: &[String]) -> Self {
        Self {
            toggles: ids
                .iter()
                .filter_map(|id| ToggleModifier::from_id(id))
                .collect(),
        }
    }
}

/// Marker for enemies with the Fortified Horde temporary shield.
/// Used to drive the pulsing yellow glow visual effect.
#[derive(Component)]
pub(crate) struct FortifiedHordeShield;
