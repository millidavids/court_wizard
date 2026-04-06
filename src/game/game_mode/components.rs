use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Which game mode the player is currently playing.
/// Inserted when the player selects a mode on the GameModeSelect screen.
/// Removed when returning to the main menu.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameMode {
    Endless,
    Roguelite,
}

impl GameMode {
    pub(crate) fn is_roguelite(&self) -> bool {
        matches!(self, GameMode::Roguelite)
    }

    pub(crate) fn is_endless(&self) -> bool {
        matches!(self, GameMode::Endless)
    }
}

/// Helper to check if the optional game mode resource is roguelite.
pub(crate) fn is_roguelite_mode(game_mode: Option<&GameMode>) -> bool {
    game_mode.is_some_and(|m| m.is_roguelite())
}

/// Helper to check if the optional game mode resource is endless.
pub(crate) fn is_endless_mode(game_mode: Option<&GameMode>) -> bool {
    game_mode.is_some_and(|m| m.is_endless())
}

/// Tracks the current roguelite run's accumulated stats.
/// Inserted when a new roguelite run starts, removed when the run ends.
#[derive(Resource)]
pub(crate) struct RogueliteRunState {
    pub started_at: u64,
    pub level_stats: Vec<LevelRunStats>,
}

/// Stats for a single level within a roguelite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LevelRunStats {
    pub level: u32,
    pub efficiency: f32,
    pub attackers_killed: u32,
    pub undead_killed: u32,
    pub defenders_lost: u32,
    pub elapsed_time: f32,
}

impl LevelRunStats {
    /// Calculates total kills (attackers + undead) for this level.
    pub fn total_kills(&self) -> u32 {
        self.attackers_killed + self.undead_killed
    }
}

/// Aggregate stats for a roguelite run.
pub(crate) struct RunAggregateStats {
    pub total_kills: u32,
    pub avg_efficiency: f32,
    pub total_time: f32,
}

impl RunAggregateStats {
    /// Computes aggregate stats from a slice of level stats.
    pub fn from_level_stats(stats: &[LevelRunStats]) -> Self {
        let total_kills: u32 = stats.iter().map(|s| s.total_kills()).sum();
        let total_time: f32 = stats.iter().map(|s| s.elapsed_time).sum();
        let avg_efficiency = if stats.is_empty() {
            0.0
        } else {
            stats.iter().map(|s| s.efficiency).sum::<f32>() / stats.len() as f32
        };
        Self {
            total_kills,
            avg_efficiency,
            total_time,
        }
    }
}

/// Formats seconds into a human-readable time string.
pub(crate) fn format_time(seconds: f32) -> String {
    let total = seconds as u32;
    let mins = total / 60;
    let secs = total % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Player-chosen modifiers for a roguelite run.
/// Inserted from the modifier selection screen; removed on return to main menu.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RogueliteModifiers {
    /// Wave spawn frequency multiplier (0.2–3.0, default 1.0).
    /// Higher values make waves arrive faster.
    pub game_speed: f32,
    /// Attacker base effectiveness multiplier (0.2–3.0, default 1.0).
    /// Higher values make enemies hit harder and move faster.
    pub enemy_effectiveness: f32,
    /// Enemy count multiplier (0.2–3.0, default 1.0).
    /// Higher values spawn more enemies per wave.
    pub enemy_count: f32,
    /// Terrain density multiplier (0.0–3.0, default 1.0).
    /// Controls how many trees, ponds, and bushes spawn.
    /// 0.0 = no terrain, 1.0 = normal, 3.0 = triple density.
    pub terrain_density: f32,
}

impl Default for RogueliteModifiers {
    fn default() -> Self {
        Self {
            game_speed: 1.0,
            enemy_effectiveness: 1.0,
            enemy_count: 1.0,
            terrain_density: 1.0,
        }
    }
}

impl RogueliteModifiers {
    /// Returns true if all modifiers are at their default (100%) values.
    #[allow(dead_code)]
    pub fn is_default(&self) -> bool {
        (self.game_speed - 1.0).abs() < 0.01
            && (self.enemy_effectiveness - 1.0).abs() < 0.01
            && (self.enemy_count - 1.0).abs() < 0.01
            && (self.terrain_density - 1.0).abs() < 0.01
    }

    /// Returns `(label, percentage)` pairs for sliders that deviate from 100%.
    pub fn non_default_entries(&self) -> Vec<(&'static str, u32)> {
        let mut entries = Vec::new();
        let sliders: [(&str, f32); 4] = [
            ("Wave Speed", self.game_speed),
            ("Enemy Strength", self.enemy_effectiveness),
            ("Enemy Count", self.enemy_count),
            ("Terrain", self.terrain_density),
        ];
        for (label, value) in sliders {
            if (value - 1.0).abs() > 0.01 {
                entries.push((label, (value * 100.0) as u32));
            }
        }
        entries
    }
}

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
            ToggleModifier::ManaDrought => {
                "No passive mana regen. Kills restore mana instead."
            }
            ToggleModifier::BossParade => {
                "Every 3rd wave includes a boss unit."
            }
            ToggleModifier::Urgent => {
                "Game keeps running while browsing menus."
            }
            ToggleModifier::FortifiedHorde => {
                "First wave arrives with damage shields."
            }
            ToggleModifier::GlassCannon => {
                "Defenders deal 2x damage but have half health."
            }
            ToggleModifier::SpellRotation => {
                "Can't cast the same spell twice in a row."
            }
            ToggleModifier::WizardCycle => {
                "Wizard type cycles through your unlocked archetypes."
            }
            ToggleModifier::ScorchedEarth => {
                "Ground-effect spells last 3x longer."
            }
            ToggleModifier::VeteranDefenders => {
                "Half as many defenders, but much tougher."
            }
            ToggleModifier::Attrition => {
                "Dead defenders don't come back next level."
            }
            ToggleModifier::RisingTide => {
                "Each wave has 25% more enemies than the last."
            }
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

/// Tracks surviving defenders across roguelite levels for the Attrition toggle.
/// Inserted at the start of a roguelite run with Attrition active.
#[derive(Resource)]
pub(crate) struct AttritionState {
    /// Surviving infantry count for the next level.
    pub infantry: u32,
    /// Surviving archers count for the next level.
    pub archers: u32,
    /// Surviving king's guard count for the next level.
    pub guards: u32,
}

impl Default for AttritionState {
    fn default() -> Self {
        Self {
            infantry: crate::game::constants::INITIAL_DEFENDER_COUNT,
            archers: crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT,
            guards: crate::game::constants::KINGS_GUARD_COUNT,
        }
    }
}

/// Flash banner for wizard type change announcements.
#[derive(Component)]
pub(crate) struct WizardCycleFlash {
    pub timer: f32,
}

/// Marker component for archetype-specific UI elements.
/// Added to all archetype UI roots so they can be despawned when cycling wizard types.
#[derive(Component)]
pub(crate) struct ArchetypeUI;

/// Timer for the Wizard Cycle toggle — cycles wizard archetype on expiry.
#[derive(Resource)]
pub(crate) struct WizardCycleTimer {
    pub timer: f32,
    pub unlocked_types: Vec<crate::config::WizardType>,
    pub current_index: usize,
}

impl WizardCycleTimer {
    /// Cycle interval in seconds.
    pub const INTERVAL: f32 = 30.0;
}

/// Tracks the last spell that was successfully cast (for Spell Rotation toggle).
/// When active, the player cannot cast the same spell consecutively.
#[derive(Resource)]
pub(crate) struct LastCastSpell {
    /// The last fully completed spell — blocks re-casting via spell_is_primed.
    pub spell: Option<crate::game::units::wizard::components::Spell>,
    /// Spell that consumed mana but hasn't finished yet (casting/channeling).
    /// Committed to `spell` when the wizard returns to Resting.
    pub pending_spell: Option<crate::game::units::wizard::components::Spell>,
    /// Previous frame's mana value — used to detect mana consumption.
    pub prev_mana: f32,
    /// When true, spell_is_primed skips the rotation check for one frame.
    /// Set by rune activation to bypass rotation blocking.
    pub bypass_until_cast: bool,
}

impl Default for LastCastSpell {
    fn default() -> Self {
        Self {
            spell: None,
            pending_spell: None,
            prev_mana: f32::MAX,
            bypass_until_cast: false,
        }
    }
}

/// Last level of a roguelite run (tier 4 boss, level 25).
/// Duration multiplier applied by the Scorched Earth toggle.
const SCORCHED_EARTH_DURATION_MULT: f32 = 3.0;

/// Last level of a roguelite run (tier 4 boss, level 25).
pub(crate) const ROGUELITE_MAX_LEVEL: u32 = 25;

/// Maximum stored roguelite runs per wizard.
pub(crate) const MAX_ROGUELITE_RUN_HISTORY: usize = 20;
