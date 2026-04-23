//! Configurable key bindings for all gameplay input.
//!
//! Each wizard archetype has its own binding context, and universal bindings
//! (action bar, activate) are shared across all archetypes.
//! All fields are `Option<KeyCode>` — `None` means the key is unbound.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Serde helpers
// ---------------------------------------------------------------------------

/// Serde module for `Option<KeyCode>` fields.
/// Serializes `None` as `"Unbound"`, `Some(key)` as the key's Debug name.
mod optional_keycode_serde {
    use bevy::prelude::KeyCode;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(key: &Option<KeyCode>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match key {
            None => serializer.serialize_str("Unbound"),
            Some(k) => serializer.serialize_str(&format!("{k:?}")),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<KeyCode>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "Unbound" {
            Ok(None)
        } else {
            Ok(Some(super::from_string(&s)))
        }
    }
}

/// Converts a debug-format string back to a `KeyCode`.
/// Falls back to `KeyCode::Escape` for unrecognized strings (should never happen
/// with well-formed config files).
fn from_string(s: &str) -> KeyCode {
    match s {
        "Digit1" => KeyCode::Digit1,
        "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3,
        "Digit4" => KeyCode::Digit4,
        "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6,
        "Digit7" => KeyCode::Digit7,
        "Digit8" => KeyCode::Digit8,
        "Digit9" => KeyCode::Digit9,
        "Digit0" => KeyCode::Digit0,
        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "ShiftLeft" => KeyCode::ShiftLeft,
        "ShiftRight" => KeyCode::ShiftRight,
        "ControlLeft" => KeyCode::ControlLeft,
        "ControlRight" => KeyCode::ControlRight,
        "AltLeft" => KeyCode::AltLeft,
        "AltRight" => KeyCode::AltRight,
        "Comma" => KeyCode::Comma,
        "Period" => KeyCode::Period,
        "Slash" => KeyCode::Slash,
        "Semicolon" => KeyCode::Semicolon,
        "Quote" => KeyCode::Quote,
        "BracketLeft" => KeyCode::BracketLeft,
        "BracketRight" => KeyCode::BracketRight,
        "Backslash" => KeyCode::Backslash,
        "Minus" => KeyCode::Minus,
        "Equal" => KeyCode::Equal,
        "Backquote" => KeyCode::Backquote,
        _ => {
            warn!("Unrecognized key binding string: {s}, falling back to Escape");
            KeyCode::Escape
        }
    }
}

// ---------------------------------------------------------------------------
// Binding contexts
// ---------------------------------------------------------------------------

/// Identifies which binding context (archetype) a key belongs to.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingContext {
    Universal,
    RuneCaster,
    Battlemage,
    Warglock,
    Meteorologist,
    ArcanoRouter,
}

/// Identifies a specific action within any binding context.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingAction {
    // Universal
    Activate,
    ActionSlot1,
    ActionSlot2,
    ActionSlot3,
    ActionSlot4,
    ActionSlot5,
    // RuneCaster
    Rune1,
    Rune2,
    Rune3,
    Rune4,
    // Battlemage
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    // Warglock
    Reload,
    // Meteorologist
    Weather1,
    Weather2,
    Weather3,
    // ArcanoRouter
    RangeUp,
    ManaUp,
    PowerUp,
    SpeedUp,
}

impl BindingAction {
    pub fn from_label(label: &str) -> Option<Self> {
        Some(match label {
            "Slot 1" => Self::ActionSlot1,
            "Slot 2" => Self::ActionSlot2,
            "Slot 3" => Self::ActionSlot3,
            "Slot 4" => Self::ActionSlot4,
            "Slot 5" => Self::ActionSlot5,
            "Activate" => Self::Activate,
            "Rune 1" => Self::Rune1,
            "Rune 2" => Self::Rune2,
            "Rune 3" => Self::Rune3,
            "Rune 4" => Self::Rune4,
            "Forward" => Self::MoveForward,
            "Backward" => Self::MoveBackward,
            "Left" => Self::MoveLeft,
            "Right" => Self::MoveRight,
            "Reload" => Self::Reload,
            "Storm" => Self::Weather1,
            "Blizzard" => Self::Weather2,
            "Drought" => Self::Weather3,
            "Range +" => Self::RangeUp,
            "Mana +" => Self::ManaUp,
            "Power +" => Self::PowerUp,
            "Speed +" => Self::SpeedUp,
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------------------
// Sub-structs
// ---------------------------------------------------------------------------

/// Key bindings shared across all wizard archetypes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct UniversalBindings {
    #[serde(with = "optional_keycode_serde")]
    pub action_slot_1: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub action_slot_2: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub action_slot_3: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub action_slot_4: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub action_slot_5: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub activate: Option<KeyCode>,
}

impl Default for UniversalBindings {
    fn default() -> Self {
        Self {
            action_slot_1: Some(KeyCode::Digit1),
            action_slot_2: Some(KeyCode::Digit2),
            action_slot_3: Some(KeyCode::Digit3),
            action_slot_4: Some(KeyCode::Digit4),
            action_slot_5: Some(KeyCode::Digit5),
            activate: Some(KeyCode::Space),
        }
    }
}

/// Key bindings for the Rune Caster archetype.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RuneCasterBindings {
    #[serde(with = "optional_keycode_serde")]
    pub rune_1: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub rune_2: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub rune_3: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub rune_4: Option<KeyCode>,
}

impl Default for RuneCasterBindings {
    fn default() -> Self {
        Self {
            rune_1: Some(KeyCode::KeyQ),
            rune_2: Some(KeyCode::KeyW),
            rune_3: Some(KeyCode::KeyE),
            rune_4: Some(KeyCode::KeyR),
        }
    }
}

/// Key bindings for the Battlemage (Swordcerer) archetype.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct BattlemageBindings {
    #[serde(with = "optional_keycode_serde")]
    pub move_forward: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_backward: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_left: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_right: Option<KeyCode>,
}

impl Default for BattlemageBindings {
    fn default() -> Self {
        Self {
            move_forward: Some(KeyCode::KeyW),
            move_backward: Some(KeyCode::KeyS),
            move_left: Some(KeyCode::KeyA),
            move_right: Some(KeyCode::KeyD),
        }
    }
}

/// Key bindings for the Warglock (Warglock) archetype.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct WarglockBindings {
    #[serde(with = "optional_keycode_serde")]
    pub reload: Option<KeyCode>,
}

impl Default for WarglockBindings {
    fn default() -> Self {
        Self {
            reload: Some(KeyCode::KeyR),
        }
    }
}

/// Key bindings for the Meteorologist archetype.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct MeteorologistBindings {
    #[serde(with = "optional_keycode_serde")]
    pub weather_1: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub weather_2: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub weather_3: Option<KeyCode>,
}

impl Default for MeteorologistBindings {
    fn default() -> Self {
        Self {
            weather_1: Some(KeyCode::KeyQ),
            weather_2: Some(KeyCode::KeyW),
            weather_3: Some(KeyCode::KeyE),
        }
    }
}

/// Key bindings for the ArcanoRouter archetype.
///
/// Only increment keys — sliders are reactive, so pushing one up pulls the
/// others down proportionally. Old save files that contain the removed
/// decrement fields are silently tolerated by serde (unknown fields ignored).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ArcanoRouterBindings {
    #[serde(with = "optional_keycode_serde")]
    pub range_up: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub mana_up: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub power_up: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub speed_up: Option<KeyCode>,
}

impl Default for ArcanoRouterBindings {
    fn default() -> Self {
        Self {
            range_up: Some(KeyCode::KeyQ),
            mana_up: Some(KeyCode::KeyW),
            power_up: Some(KeyCode::KeyE),
            speed_up: Some(KeyCode::KeyR),
        }
    }
}

// ---------------------------------------------------------------------------
// Main resource
// ---------------------------------------------------------------------------

/// Top-level input bindings resource, persisted as part of the config file.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct InputBindings {
    #[serde(default)]
    pub universal: UniversalBindings,
    #[serde(default)]
    pub rune_caster: RuneCasterBindings,
    #[serde(default)]
    pub battlemage: BattlemageBindings,
    #[serde(default)]
    pub warglock: WarglockBindings,
    #[serde(default)]
    pub meteorologist: MeteorologistBindings,
    #[serde(default)]
    pub arcanorouter: ArcanoRouterBindings,
}

#[allow(dead_code)]
impl InputBindings {
    /// Returns the current binding for a given context + action.
    pub fn get(&self, context: BindingContext, action: BindingAction) -> Option<KeyCode> {
        match (context, action) {
            (BindingContext::Universal, BindingAction::ActionSlot1) => self.universal.action_slot_1,
            (BindingContext::Universal, BindingAction::ActionSlot2) => self.universal.action_slot_2,
            (BindingContext::Universal, BindingAction::ActionSlot3) => self.universal.action_slot_3,
            (BindingContext::Universal, BindingAction::ActionSlot4) => self.universal.action_slot_4,
            (BindingContext::Universal, BindingAction::ActionSlot5) => self.universal.action_slot_5,
            (BindingContext::Universal, BindingAction::Activate) => self.universal.activate,
            (BindingContext::RuneCaster, BindingAction::Rune1) => self.rune_caster.rune_1,
            (BindingContext::RuneCaster, BindingAction::Rune2) => self.rune_caster.rune_2,
            (BindingContext::RuneCaster, BindingAction::Rune3) => self.rune_caster.rune_3,
            (BindingContext::RuneCaster, BindingAction::Rune4) => self.rune_caster.rune_4,
            (BindingContext::Battlemage, BindingAction::MoveForward) => {
                self.battlemage.move_forward
            }
            (BindingContext::Battlemage, BindingAction::MoveBackward) => {
                self.battlemage.move_backward
            }
            (BindingContext::Battlemage, BindingAction::MoveLeft) => self.battlemage.move_left,
            (BindingContext::Battlemage, BindingAction::MoveRight) => self.battlemage.move_right,
            (BindingContext::Warglock, BindingAction::Reload) => self.warglock.reload,
            (BindingContext::Meteorologist, BindingAction::Weather1) => {
                self.meteorologist.weather_1
            }
            (BindingContext::Meteorologist, BindingAction::Weather2) => {
                self.meteorologist.weather_2
            }
            (BindingContext::Meteorologist, BindingAction::Weather3) => {
                self.meteorologist.weather_3
            }
            (BindingContext::ArcanoRouter, BindingAction::RangeUp) => self.arcanorouter.range_up,
            (BindingContext::ArcanoRouter, BindingAction::ManaUp) => self.arcanorouter.mana_up,
            (BindingContext::ArcanoRouter, BindingAction::PowerUp) => self.arcanorouter.power_up,
            (BindingContext::ArcanoRouter, BindingAction::SpeedUp) => self.arcanorouter.speed_up,
            _ => None,
        }
    }

    /// Sets the binding for a given context + action (accepts `Option<KeyCode>`).
    pub fn set(&mut self, context: BindingContext, action: BindingAction, key: Option<KeyCode>) {
        match (context, action) {
            (BindingContext::Universal, BindingAction::ActionSlot1) => {
                self.universal.action_slot_1 = key;
            }
            (BindingContext::Universal, BindingAction::ActionSlot2) => {
                self.universal.action_slot_2 = key;
            }
            (BindingContext::Universal, BindingAction::ActionSlot3) => {
                self.universal.action_slot_3 = key;
            }
            (BindingContext::Universal, BindingAction::ActionSlot4) => {
                self.universal.action_slot_4 = key;
            }
            (BindingContext::Universal, BindingAction::ActionSlot5) => {
                self.universal.action_slot_5 = key;
            }
            (BindingContext::Universal, BindingAction::Activate) => {
                self.universal.activate = key;
            }
            (BindingContext::RuneCaster, BindingAction::Rune1) => {
                self.rune_caster.rune_1 = key;
            }
            (BindingContext::RuneCaster, BindingAction::Rune2) => {
                self.rune_caster.rune_2 = key;
            }
            (BindingContext::RuneCaster, BindingAction::Rune3) => {
                self.rune_caster.rune_3 = key;
            }
            (BindingContext::RuneCaster, BindingAction::Rune4) => {
                self.rune_caster.rune_4 = key;
            }
            (BindingContext::Battlemage, BindingAction::MoveForward) => {
                self.battlemage.move_forward = key;
            }
            (BindingContext::Battlemage, BindingAction::MoveBackward) => {
                self.battlemage.move_backward = key;
            }
            (BindingContext::Battlemage, BindingAction::MoveLeft) => {
                self.battlemage.move_left = key;
            }
            (BindingContext::Battlemage, BindingAction::MoveRight) => {
                self.battlemage.move_right = key;
            }
            (BindingContext::Warglock, BindingAction::Reload) => {
                self.warglock.reload = key;
            }
            (BindingContext::Meteorologist, BindingAction::Weather1) => {
                self.meteorologist.weather_1 = key;
            }
            (BindingContext::Meteorologist, BindingAction::Weather2) => {
                self.meteorologist.weather_2 = key;
            }
            (BindingContext::Meteorologist, BindingAction::Weather3) => {
                self.meteorologist.weather_3 = key;
            }
            (BindingContext::ArcanoRouter, BindingAction::RangeUp) => {
                self.arcanorouter.range_up = key;
            }
            (BindingContext::ArcanoRouter, BindingAction::ManaUp) => {
                self.arcanorouter.mana_up = key;
            }
            (BindingContext::ArcanoRouter, BindingAction::PowerUp) => {
                self.arcanorouter.power_up = key;
            }
            (BindingContext::ArcanoRouter, BindingAction::SpeedUp) => {
                self.arcanorouter.speed_up = key;
            }
            _ => {}
        }
    }

    /// Returns all key/action pairs for a given context as `(label, Option<KeyCode>)`.
    pub fn context_keys(&self, context: BindingContext) -> Vec<(&str, Option<KeyCode>)> {
        match context {
            BindingContext::Universal => vec![
                ("Slot 1", self.universal.action_slot_1),
                ("Slot 2", self.universal.action_slot_2),
                ("Slot 3", self.universal.action_slot_3),
                ("Slot 4", self.universal.action_slot_4),
                ("Slot 5", self.universal.action_slot_5),
                ("Activate", self.universal.activate),
            ],
            BindingContext::RuneCaster => vec![
                ("Rune 1", self.rune_caster.rune_1),
                ("Rune 2", self.rune_caster.rune_2),
                ("Rune 3", self.rune_caster.rune_3),
                ("Rune 4", self.rune_caster.rune_4),
            ],
            BindingContext::Battlemage => vec![
                ("Forward", self.battlemage.move_forward),
                ("Backward", self.battlemage.move_backward),
                ("Left", self.battlemage.move_left),
                ("Right", self.battlemage.move_right),
            ],
            BindingContext::Warglock => vec![("Reload", self.warglock.reload)],
            BindingContext::Meteorologist => vec![
                ("Storm", self.meteorologist.weather_1),
                ("Blizzard", self.meteorologist.weather_2),
                ("Drought", self.meteorologist.weather_3),
            ],
            BindingContext::ArcanoRouter => vec![
                ("Range +", self.arcanorouter.range_up),
                ("Mana +", self.arcanorouter.mana_up),
                ("Power +", self.arcanorouter.power_up),
                ("Speed +", self.arcanorouter.speed_up),
            ],
        }
    }

    /// Checks if binding `key` to `(context, action)` would conflict.
    /// Returns the conflict display label, or None.
    pub fn would_conflict(
        &self,
        key: KeyCode,
        context: BindingContext,
        action: BindingAction,
    ) -> Option<String> {
        self.find_conflict_inner(key, context, action)
            .map(|(ctx, label, _)| format!("{ctx:?}: {label}"))
    }

    /// Returns the (context, action) of the conflicting binding, if any.
    pub fn find_conflict(
        &self,
        key: KeyCode,
        context: BindingContext,
        action: BindingAction,
    ) -> Option<(BindingContext, BindingAction)> {
        self.find_conflict_inner(key, context, action)
            .and_then(|(ctx, label, _)| BindingAction::from_label(label).map(|a| (ctx, a)))
    }

    fn find_conflict_inner(
        &self,
        key: KeyCode,
        context: BindingContext,
        action: BindingAction,
    ) -> Option<(BindingContext, &str, Option<KeyCode>)> {
        // Check universal bindings (always checked)
        for (label, existing) in self.context_keys(BindingContext::Universal) {
            if let Some(existing_key) = existing
                && existing_key == key
                && !(context == BindingContext::Universal
                    && self.get(BindingContext::Universal, action) == Some(key))
            {
                return Some((BindingContext::Universal, label, existing));
            }
        }
        // Check the target context (if not universal)
        if context != BindingContext::Universal {
            for (label, existing) in self.context_keys(context) {
                if let Some(existing_key) = existing
                    && existing_key == key
                    && self.get(context, action) != Some(key)
                {
                    return Some((context, label, existing));
                }
            }
        }
        None
    }

    /// Returns `true` if all universal bindings are unbound.
    pub fn all_universal_unbound(&self) -> bool {
        self.universal.action_slot_1.is_none()
            && self.universal.action_slot_2.is_none()
            && self.universal.action_slot_3.is_none()
            && self.universal.action_slot_4.is_none()
            && self.universal.action_slot_5.is_none()
            && self.universal.activate.is_none()
    }

    /// Returns `true` if all bindings in the given context are unbound.
    pub fn all_context_unbound(&self, context: BindingContext) -> bool {
        self.context_keys(context)
            .iter()
            .all(|(_, key)| key.is_none())
    }
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

/// Returns a human-readable name for a `KeyCode`.
pub(crate) fn key_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Tab => "Tab",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        KeyCode::ShiftLeft => "L-Shift",
        KeyCode::ShiftRight => "R-Shift",
        KeyCode::ControlLeft => "L-Ctrl",
        KeyCode::ControlRight => "R-Ctrl",
        KeyCode::AltLeft => "L-Alt",
        KeyCode::AltRight => "R-Alt",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::Backquote => "`",
        KeyCode::Escape => "Esc",
        _ => "???",
    }
}

/// Returns a display name for an optional key binding.
/// Shows `"-"` for unbound keys.
pub(crate) fn key_display_name(key: Option<KeyCode>) -> &'static str {
    match key {
        Some(k) => key_name(k),
        None => "-",
    }
}

/// Returns `true` if a key is allowed to be bound.
/// Excludes Escape (used for menus) and Backspace (used as unbind key in UI).
#[allow(dead_code)]
pub(crate) fn is_bindable_key(key: KeyCode) -> bool {
    !matches!(key, KeyCode::Escape | KeyCode::Backspace)
}
