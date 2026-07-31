use bevy::prelude::*;

use super::modifiers::LevelRunStats;

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
    /// True once the run has used the action-bar hotkeys, the controller's
    /// radial menu, or an archetype key / D-pad ability. Blocks "Clicker".
    pub used_non_mouse_input: bool,
}

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
