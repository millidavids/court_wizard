use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use crate::config::save_data::{
    SavedBoulder, SavedBush, SavedCrystal, SavedFlora, SavedPond, SavedTrampling, SavedTree,
    SavedWall,
};
use crate::game::units::wizard::components::Spell;

/// Temporary structure for TOML serialization only.
///
/// This is NOT a runtime resource. It only exists during:
/// 1. Startup: Load from disk → apply to Bevy components
/// 2. Save: Read from Bevy components → serialize to disk
///
/// During runtime, Bevy components are the single source of truth.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ConfigFile {
    /// Window configuration settings
    pub window: WindowConfig,
    /// Audio configuration settings
    pub audio: AudioConfig,
    /// Game configuration settings (includes all user preferences)
    pub game: GameConfig,
    /// Input key bindings
    #[serde(default)]
    pub controls: super::input_bindings::InputBindings,
}

/// VSync (vertical synchronization) mode options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum VsyncMode {
    /// VSync enabled
    #[default]
    On,
    /// VSync disabled
    Off,
    /// Adaptive VSync (falls back to off if frame rate drops)
    Adaptive,
}

/// Display mode options for the game window.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Standard windowed mode with title bar and borders
    #[default]
    Windowed,
    /// Borderless fullscreen (covers the screen without exclusive mode)
    BorderlessFullscreen,
}

/// Window settings for serialization to/from TOML.
///
/// During runtime, Bevy's `Window` component is the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WindowConfig {
    /// VSync mode (on, off, or adaptive)
    pub vsync: VsyncMode,
    /// Display mode (windowed or borderless fullscreen)
    #[serde(default)]
    pub display_mode: DisplayMode,
    /// Scale factor override (None uses OS default)
    pub scale_factor: Option<f64>,
    /// Saved window X position in physical pixels (None = let OS decide)
    #[serde(default)]
    pub position_x: Option<i32>,
    /// Saved window Y position in physical pixels (None = let OS decide)
    #[serde(default)]
    pub position_y: Option<i32>,
    /// Saved windowed mode width in logical pixels
    #[serde(default = "default_windowed_width")]
    pub windowed_width: f32,
    /// Saved windowed mode height in logical pixels
    #[serde(default = "default_windowed_height")]
    pub windowed_height: f32,
}

const DEFAULT_WINDOWED_WIDTH: f32 = 1920.0;
const DEFAULT_WINDOWED_HEIGHT: f32 = 1080.0;

fn default_windowed_width() -> f32 {
    DEFAULT_WINDOWED_WIDTH
}
fn default_windowed_height() -> f32 {
    DEFAULT_WINDOWED_HEIGHT
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            vsync: VsyncMode::default(),
            display_mode: DisplayMode::default(),
            scale_factor: Some(1.0),
            position_x: None,
            position_y: None,
            windowed_width: DEFAULT_WINDOWED_WIDTH,
            windowed_height: DEFAULT_WINDOWED_HEIGHT,
        }
    }
}

/// Runtime resource tracking the last windowed-mode size and position.
/// Saved before entering fullscreen/borderless so we can restore when switching back.
#[derive(Resource, Debug, Clone)]
pub(crate) struct SavedWindowedGeometry {
    pub width: f32,
    pub height: f32,
    pub position: Option<IVec2>,
    /// When set, apply this display mode change on the next frame.
    /// Deferred by one frame to avoid scissor rect / render target size mismatch
    /// when the window mode and resolution change in the same frame.
    pub pending_mode_change: Option<DisplayMode>,
}

/// Audio settings for serialization to/from TOML.
///
/// During runtime, Bevy's audio resources are the source of truth.
/// This struct is only used for persistence to/from the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AudioConfig {
    /// Master volume level (0.0 = muted, 1.0 = full volume)
    pub master_volume: f32,
    /// Music track volume level (0.0 = muted, 1.0 = full volume)
    pub music_volume: f32,
    /// Sound effects volume level (0.0 = muted, 1.0 = full volume)
    pub sfx_volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.3,
            sfx_volume: 0.8,
        }
    }
}

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

/// Wizard class types available for selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum WizardType {
    /// Boring Ole Mage - no special mechanics, flat 5% bonus to all stats.
    #[default]
    BoringOleMage,
    /// Rune-based caster - the classic wizard type.
    RuneCaster,
    /// Randomancer - spins a roulette wheel for powerful random spells.
    Randomancer,
    /// Arcanorouter - allocates resources between range, mana, power, and speed.
    Arcanorouter,
    /// Excremage - converts all spells to Poop damage with brown visuals.
    Excremage,
    /// Alchemist - brewing specialist with faster brews, longer buffs, and Philosopher's Stone.
    Alchemist,
    /// Warglock - replaces spells with 5 guns.
    Warglock,
    /// Battlemage - enters the battlefield as a melee/ranged fighter.
    Battlemage,
    /// Meteorologist - manipulates weather to apply global status effects.
    Meteorologist,
    /// Shepherd - support-only wizard with no damage-dealing spells, bonus to support effects.
    Shepherd,
    /// Psychopath - wants maximum carnage; must kill 80% of defenders to win.
    Psychopath,
}

impl WizardType {
    /// Returns the display name for this wizard type.
    pub const fn display_name(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => "Boring Ole Mage",
            WizardType::RuneCaster => "Rune Caster",
            WizardType::Randomancer => "Randomancer",
            WizardType::Arcanorouter => "Arcanorouter",
            WizardType::Excremage => "Excremage",
            WizardType::Alchemist => "The Alchemist",
            WizardType::Warglock => "Warglock",
            WizardType::Battlemage => "Swordcerer",
            WizardType::Meteorologist => "Meteorologist",
            WizardType::Shepherd => "Shepherd",
            WizardType::Psychopath => "Psychopath",
        }
    }

    /// Returns true if this wizard type uses its own exclusive casting mechanic
    /// (rune sequences or roulette spins) and should not access the spell book
    /// or action bar spell priming.
    pub const fn uses_exclusive_casting(&self) -> bool {
        matches!(self, WizardType::RuneCaster | WizardType::Randomancer)
    }

    /// Returns a short description of this wizard type's playstyle.
    pub const fn description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => {
                "A straightforward wizard with a small bonus to everything."
            }
            WizardType::RuneCaster => "Master rune sequences to empower your spells.",
            WizardType::Randomancer => "Spin the wheel of fate for powerful random spells.",
            WizardType::Arcanorouter => "Route arcane power between range, mana, power, and speed.",
            WizardType::Excremage => "Turn all spells into poop.",
            WizardType::Alchemist => "Master the cauldron with faster brews and stronger potions.",
            WizardType::Warglock => "Who needs spells when you have guns?",
            WizardType::Battlemage => "Leave the tower. Enter the fray.",
            WizardType::Meteorologist => "Control the weather. Control the battlefield.",
            WizardType::Shepherd => "Heal. Shield. Inspire. Never harm.",
            WizardType::Psychopath => "Burn it all. Both sides.",
        }
    }

    /// Returns a longer description explaining the archetype's mechanics in detail.
    pub const fn long_description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => {
                "No special mechanics to learn. Your spells are slightly stronger, cheaper, faster, and longer-ranged than other wizards. A solid choice for beginners or anyone who just wants to cast spells without thinking about it."
            }
            WizardType::RuneCaster => {
                "Press Q, W, E, and R to build rune sequences. Single runes cast basic spells, while two-rune combos unlock powerful advanced spells. Successful sequences empower your spells with a 1.25x bonus. Sequences time out if you wait too long between keys."
            }
            WizardType::Randomancer => {
                "Press SPACE to spin a magical roulette wheel. Whatever spell it lands on, you cast — no choosing. In exchange for giving up control, your spells are empowered with a massive 1.75x bonus. Adapting to whatever the wheel gives you is the whole game."
            }
            WizardType::Arcanorouter => {
                "Dynamically allocate a shared pool of arcane energy between four stats: Range, Mana Efficiency, Power, and Speed. Use Q/A, W/S, E/D, and R/F to increase or decrease each slider. Pump everything into Power for devastating spells, or balance your build for versatility. Adjust mid-battle to adapt to the situation."
            }
            WizardType::Excremage => {
                "All your spells deal Poop damage and turn units into smelly messes. Your spells may lack elemental finesse, but nothing clears a battlefield like the smell of fear... and other things."
            }
            WizardType::Alchemist => {
                "Your brews take 20% less time and your buffs last 25% longer. Once per battle, you can add the Philosopher's Stone to a brew — it removes all dilution, so every ingredient brews at full strength. The Stone doesn't count toward the 3-ingredient limit."
            }
            WizardType::Warglock => {
                "Spells? Where you're going, you don't need spells. Your 5 action bar slots become 5 different guns, each with its own ammo pool. Machine gun sprays bullets, magnum hits hard, rocket launcher explodes, shotgun blasts in a cone, and flamethrower burns everything. Reload with R or let it auto-reload when empty."
            }
            WizardType::Battlemage => {
                "Click 'Enter the Fray' to leave your tower and join the battle as a warrior. Move with WASD, shoot magic missiles with left-click, and swing your sword with right-click. You're fast but vulnerable — if your health hits zero, you teleport back to the tower. While on the field, spells, spell book, and cauldron are disabled."
            }
            WizardType::Meteorologist => {
                "Press Q, W, or E to change the weather. Storm makes units Wet and Charged — shocked units spread electricity to nearby wet targets, electric arcs hit more targets, and random lightning strikes the field. Blizzard makes units Cold — frost spells slow even harder, and can freeze units solid. Drought makes units Dry — fire spells create burning patches on the ground. Weather effects grow stronger the longer they persist."
            }
            WizardType::Shepherd => {
                "You cannot cast any spell that deals damage. No fireballs, no black holes, no spike growth — nothing that hurts. In exchange, all your support spells are 30% more powerful: bigger heals, stronger shields, longer buffs, and tougher walls. Guide your flock to victory through faith alone."
            }
            WizardType::Psychopath => {
                "Victory isn't enough — you need carnage. Your spells deal 30% extra damage to your own defenders. To win a level, at least 70% of your defenders must be dead by the time the last attacker falls. If too many survive, you lose. Efficiency is for cowards."
            }
        }
    }

    /// Returns flavor text shown on the progress screen when the wizard type is locked.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => "Sometimes boring is best.",
            WizardType::RuneCaster => "Mysterious symbols. Mysterious results.",
            WizardType::Randomancer => "You don't choose the spell. The spell chooses you.",
            WizardType::Arcanorouter => "Geordi would be proud of your power routing.",
            WizardType::Excremage => "Something smells off...",
            WizardType::Alchemist => "The cauldron whispers to those who listen.",
            WizardType::Warglock => "War and sorcery, forged into one.",
            WizardType::Battlemage => "Some wizards prefer a more... hands-on approach.",
            WizardType::Meteorologist => "The sky darkens. Something is brewing up there.",
            WizardType::Shepherd => "Violence is never the answer... right?",
            WizardType::Psychopath => "Some people just want to watch the world burn.",
        }
    }

    /// Returns all available wizard types.
    pub const fn all() -> &'static [WizardType] {
        &[
            WizardType::BoringOleMage,
            WizardType::RuneCaster,
            WizardType::Randomancer,
            WizardType::Arcanorouter,
            WizardType::Excremage,
            WizardType::Alchemist,
            WizardType::Warglock,
            WizardType::Battlemage,
            WizardType::Meteorologist,
            WizardType::Shepherd,
            WizardType::Psychopath,
        ]
    }
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

/// Default game speed (normal).
fn default_game_speed() -> f32 {
    1.0
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
    #[serde(default = "default_game_speed")]
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

/// Resource that tracks debounce timer for automatic config saving.
///
/// This prevents excessive file writes during window resizing by waiting
/// for a period of inactivity before saving to disk.
#[derive(Resource)]
pub(crate) struct SaveDebounceTimer {
    /// Timer that counts down after a window resize event
    pub timer: Timer,
    /// Whether a save is pending after the timer expires
    pub pending: bool,
}

impl Default for SaveDebounceTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            pending: false,
        }
    }
}
