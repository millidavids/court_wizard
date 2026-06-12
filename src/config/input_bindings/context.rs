//! Binding context and action enumerations.

// ---------------------------------------------------------------------------
// Binding contexts
// ---------------------------------------------------------------------------

/// Identifies which binding context (archetype) a key belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingContext {
    Universal,
    RuneCaster,
    Swordcerer,
    Warglock,
    Meteorologist,
    ArcanoRouter,
}

/// Identifies a specific action within any binding context.
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
    // Swordcerer
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
