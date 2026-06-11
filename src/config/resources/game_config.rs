use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use crate::config::save_data::{
    SavedBoulder, SavedBush, SavedCrystal, SavedFlora, SavedPond, SavedTrampling, SavedTree,
    SavedWall,
};
use crate::game::units::wizard::components::Spell;

use super::config_file::{DisplayMode, VsyncMode};
use super::wizard_type::WizardType;

/// Colorblind correction mode.
///
/// Selects which color vision deficiency correction to apply.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ColorblindType {
    /// No correction applied
    #[default]
    None,
    /// Correction for red-blind (L-cone) deficiency
    Protanopia,
    /// Correction for green-blind (M-cone) deficiency
    Deuteranopia,
    /// Correction for blue-blind (S-cone) deficiency
    Tritanopia,
}

/// Which controller glyph set to render in on-screen prompts. `Auto` uses
/// the connected gamepad's vendor to pick one of the four supported styles;
/// the other variants force that style regardless of hardware.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ControllerGlyphStyle {
    #[default]
    Auto,
    Xbox,
    PlayStation,
    SteamDeck,
    Switch,
}

/// Tracks which wizard is currently active by their unique ID.
/// None means no wizard is loaded (main menu before selection).
#[derive(Resource, Default)]
pub struct ActiveSave(pub Option<String>);

/// Default current level for serde deserialization.
fn default_current_level() -> u32 {
    1
}

/// Default highest level achieved for serde deserialization.
fn default_highest_level() -> u32 {
    1
}

/// Default true for boolean settings.
fn default_true() -> bool {
    true
}

/// Default empty efficiency ratios map for serde deserialization.
fn default_efficiency_ratios() -> HashMap<String, f32> {
    HashMap::new()
}

/// Default colorblind correction strength (full correction).
fn default_colorblind_strength() -> f32 {
    1.0
}

/// Default 1.0 multiplier (game speed, gamepad sensitivity, etc.).
fn default_one() -> f32 {
    1.0
}

fn default_gamepad_deadzone() -> f32 {
    0.15
}

fn default_gamepad_curve() -> f32 {
    2.2
}

/// Default action bar slots: None for all 5 slots.
fn default_action_bar_slots() -> [Option<Spell>; 5] {
    [None; 5]
}

/// Serialize action bar slots as a map with string keys, filtering out None values.
pub(crate) fn serialize_action_bar<S>(
    slots: &[Option<Spell>; 5],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(slots.len()))?;
    for (idx, slot) in slots.iter().enumerate() {
        if let Some(spell) = slot {
            map.serialize_entry(&idx.to_string(), spell)?;
        }
    }
    map.end()
}

/// Deserialize action bar slots from a map of string indices to spells.
pub(crate) fn deserialize_action_bar<'de, D>(
    deserializer: D,
) -> Result<[Option<Spell>; 5], D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Spell> = HashMap::deserialize(deserializer).unwrap_or_default();
    let mut slots = [None; 5];
    for (idx_str, spell) in map {
        if let Ok(idx) = idx_str.parse::<usize>()
            && idx < 5
        {
            slots[idx] = Some(spell);
        }
    }
    Ok(slots)
}

/// Game configuration resource - runtime source of truth for all user settings.
///
/// This IS a runtime Bevy resource that holds all user-configurable settings:
/// - VSync mode
/// - Audio volumes (master, music, SFX)
/// - Global brightness
///
/// Changes to this resource are automatically persisted to disk.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Display mode (windowed or borderless fullscreen)
    #[serde(default)]
    pub display_mode: DisplayMode,
    /// Master volume level (0.0 = muted, 1.0 = full volume)
    pub master_volume: f32,
    /// Music track volume level (0.0 = muted, 1.0 = full volume)
    pub music_volume: f32,
    /// Sound effects volume level (0.0 = muted, 1.0 = full volume)
    pub sfx_volume: f32,
    /// Global brightness multiplier (0.1 = darkest to prevent soft-lock, 1.0 = normal, 2.0 = brightest)
    pub brightness: f32,
    /// Current level - restored on game start after page reload
    #[serde(default = "default_current_level")]
    pub current_level: u32,
    /// Highest level achieved across all playthroughs (high score marker)
    #[serde(default = "default_highest_level")]
    pub highest_level_achieved: u32,
    /// Efficiency ratios per level (defenders lost / total defenders at start)
    /// Key: level number as string, Value: efficiency ratio (0.0 = all defenders lost, 1.0 = no defenders lost)
    #[serde(default = "default_efficiency_ratios")]
    pub efficiency_ratios: HashMap<String, f32>,
    /// Action bar spell slots (0-4 correspond to keys 1-5)
    /// None = empty slot, Some(spell) = assigned spell
    #[serde(
        default = "default_action_bar_slots",
        serialize_with = "serialize_action_bar",
        deserialize_with = "deserialize_action_bar"
    )]
    pub action_bar_slots: [Option<Spell>; 5],
    /// Active wizard type for the current save
    #[serde(default)]
    pub wizard_type: WizardType,
    /// Whether to skip the splash screen sequence on startup
    #[serde(default)]
    pub skip_splash: bool,
    /// Whether tutorials are enabled for new players
    #[serde(default = "default_true")]
    pub tutorials_enabled: bool,
    /// Whether to show the level clock in the HUD
    #[serde(default = "default_true")]
    pub show_level_clock: bool,
    /// Whether gameplay continues while Spellbook/Cauldron menus are open
    #[serde(default = "default_true")]
    pub urgent_mode: bool,
    /// Colorblind correction mode (None = disabled)
    #[serde(default)]
    pub colorblind_type: ColorblindType,
    /// Colorblind correction strength (0.0 = no correction, 1.0 = full)
    #[serde(default = "default_colorblind_strength")]
    pub colorblind_strength: f32,
    /// Disables screen flashes, vignette pulses, channel-change flicker, and CRT flicker
    #[serde(default)]
    pub reduce_flashes: bool,
    /// Disables screen-warping distortion effects (lensing, heat shimmer, teleport ripple)
    #[serde(default)]
    pub reduce_motion: bool,
    /// Whether the CRT TV effect (scanlines, barrel distortion, vignette) is enabled
    #[serde(default = "default_true")]
    pub crt_enabled: bool,
    /// Game speed multiplier (0.5 = half speed, 1.0 = normal, 2.0 = double)
    #[serde(default = "default_one")]
    pub game_speed: f32,
    /// Whether to auto-pause when the game window loses focus
    #[serde(default)]
    pub auto_pause_on_focus_loss: bool,
    /// High contrast effect strength (0.0 = off, 1.0 = full)
    #[serde(default)]
    pub high_contrast_strength: f32,
    /// Aim assist — snaps spell targeting to nearest unit
    #[serde(default)]
    pub aim_assist: bool,
    /// Gamepad aim sensitivity multiplier (X axis, 0.3..=2.5, default 1.0)
    #[serde(default = "default_one")]
    pub gamepad_sensitivity_x: f32,
    /// Gamepad aim sensitivity multiplier (Y axis, 0.3..=2.5, default 1.0)
    #[serde(default = "default_one")]
    pub gamepad_sensitivity_y: f32,
    /// Gamepad stick deadzone (0.05..=0.30, default 0.15)
    #[serde(default = "default_gamepad_deadzone")]
    pub gamepad_deadzone: f32,
    /// Gamepad response curve exponent (1.0..=3.5, default 2.2)
    #[serde(default = "default_gamepad_curve")]
    pub gamepad_response_curve: f32,
    /// Whether controller rumble is enabled
    #[serde(default = "default_true")]
    pub rumble_enabled: bool,
    /// Which controller glyph set to use for on-screen prompts. `Auto` picks
    /// Xbox / PlayStation / Steam Deck / Nintendo from the connected
    /// gamepad's vendor; the other variants force that style.
    #[serde(default)]
    pub controller_glyph_style: ControllerGlyphStyle,
    /// Permanent walls saved from previous victories
    #[serde(skip)]
    pub saved_walls: Vec<SavedWall>,
    /// Permanent crystals saved from previous victories (Auto-Crystal talent)
    #[serde(skip)]
    pub saved_crystals: Vec<SavedCrystal>,
    /// Battlefield flora positions (persistent across battles, removed when trampled)
    #[serde(skip)]
    pub saved_flora: Vec<SavedFlora>,
    /// Trampling grid state (mud trails on battlefield)
    #[serde(skip)]
    pub saved_trampling: SavedTrampling,
    /// Persistent trees from previous victories
    #[serde(skip)]
    pub saved_trees: Vec<SavedTree>,
    /// Persistent ponds from previous victories
    #[serde(skip)]
    pub saved_ponds: Vec<SavedPond>,
    /// Persistent bushes from previous victories
    #[serde(skip)]
    pub saved_bushes: Vec<SavedBush>,
    /// Persistent terrain boulders from previous victories
    #[serde(skip)]
    pub saved_boulders: Vec<SavedBoulder>,
    /// Master seed for the current run (deterministic gameplay).
    /// None = auto-generate a random seed at run start.
    #[serde(skip)]
    pub seed: Option<u64>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            display_mode: DisplayMode::default(),
            master_volume: 1.0,
            music_volume: 0.3,
            sfx_volume: 0.8,
            brightness: 1.0,
            current_level: 1,
            highest_level_achieved: 1,
            efficiency_ratios: HashMap::new(),
            action_bar_slots: [None; 5],
            wizard_type: WizardType::default(),
            skip_splash: false,
            tutorials_enabled: true,
            show_level_clock: true,
            urgent_mode: true,
            colorblind_type: ColorblindType::default(),
            colorblind_strength: 1.0,
            reduce_flashes: false,
            reduce_motion: false,
            crt_enabled: true,
            game_speed: 1.0,
            auto_pause_on_focus_loss: false,
            high_contrast_strength: 0.0,
            aim_assist: false,
            gamepad_sensitivity_x: 1.0,
            gamepad_sensitivity_y: 1.0,
            gamepad_deadzone: 0.15,
            gamepad_response_curve: 2.2,
            rumble_enabled: true,
            controller_glyph_style: ControllerGlyphStyle::Auto,
            saved_walls: Vec::new(),
            saved_crystals: Vec::new(),
            saved_flora: Vec::new(),
            saved_trampling: SavedTrampling::default(),
            saved_trees: Vec::new(),
            saved_ponds: Vec::new(),
            saved_bushes: Vec::new(),
            saved_boulders: Vec::new(),
            seed: None,
        }
    }
}

impl GameConfig {
    /// Effective music volume (master × music slider).
    pub fn effective_music_volume(&self) -> f32 {
        self.master_volume * self.music_volume
    }

    /// Effective SFX volume (master × sfx slider × global SFX scaling).
    pub fn effective_sfx_volume(&self) -> f32 {
        self.master_volume * self.sfx_volume * 0.4
    }

    /// Snap radius for aim assist (0 when off, 150 world units when on).
    pub fn target_assist_snap_radius(&self) -> f32 {
        if self.aim_assist { 150.0 } else { 0.0 }
    }

    /// Returns true if any accessibility assists that affect gameplay are active.
    pub fn has_accessibility_assists(&self) -> bool {
        self.game_speed != 1.0 || self.aim_assist
    }
}
