//! Per-archetype binding structs and the top-level `InputBindings` resource.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::context::{BindingAction, BindingContext};
use super::serde::optional_keycode_serde;

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

/// Key bindings for the Swordcerer archetype.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct SwordcererBindings {
    #[serde(with = "optional_keycode_serde")]
    pub move_forward: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_backward: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_left: Option<KeyCode>,
    #[serde(with = "optional_keycode_serde")]
    pub move_right: Option<KeyCode>,
}

impl Default for SwordcererBindings {
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
    #[serde(default, alias = "battlemage")]
    pub swordcerer: SwordcererBindings,
    #[serde(default)]
    pub warglock: WarglockBindings,
    #[serde(default)]
    pub meteorologist: MeteorologistBindings,
    #[serde(default)]
    pub arcanorouter: ArcanoRouterBindings,
}

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
            (BindingContext::Swordcerer, BindingAction::MoveForward) => {
                self.swordcerer.move_forward
            }
            (BindingContext::Swordcerer, BindingAction::MoveBackward) => {
                self.swordcerer.move_backward
            }
            (BindingContext::Swordcerer, BindingAction::MoveLeft) => self.swordcerer.move_left,
            (BindingContext::Swordcerer, BindingAction::MoveRight) => self.swordcerer.move_right,
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
            (BindingContext::Swordcerer, BindingAction::MoveForward) => {
                self.swordcerer.move_forward = key;
            }
            (BindingContext::Swordcerer, BindingAction::MoveBackward) => {
                self.swordcerer.move_backward = key;
            }
            (BindingContext::Swordcerer, BindingAction::MoveLeft) => {
                self.swordcerer.move_left = key;
            }
            (BindingContext::Swordcerer, BindingAction::MoveRight) => {
                self.swordcerer.move_right = key;
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
            BindingContext::Swordcerer => vec![
                ("Forward", self.swordcerer.move_forward),
                ("Backward", self.swordcerer.move_backward),
                ("Left", self.swordcerer.move_left),
                ("Right", self.swordcerer.move_right),
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
}
