//! Serde helpers for `Option<KeyCode>` input binding fields.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Serde helpers
// ---------------------------------------------------------------------------

/// Serde module for `Option<KeyCode>` fields.
/// Serializes `None` as `"Unbound"`, `Some(key)` as the key's Debug name.
pub(super) mod optional_keycode_serde {
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
pub(super) fn from_string(s: &str) -> KeyCode {
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
