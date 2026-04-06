use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::game_mode::components::{RogueliteModifiers, ToggleModifier};

/// Marker component for entities that belong to the roguelite modifiers screen.
#[derive(Component)]
pub(super) struct OnRogueliteModifiersScreen;

/// Actions for buttons on the modifiers screen.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModifierButtonAction {
    StartRun,
    Back,
}

/// Identifies which modifier a slider controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub(super) enum ModifierSliderValue {
    GameSpeed,
    EnemyEffectiveness,
    EnemyCount,
    TerrainDensity,
}

impl ModifierSliderValue {
    pub fn get(&self, modifiers: &RogueliteModifiers) -> f32 {
        match self {
            Self::GameSpeed => modifiers.game_speed,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness,
            Self::EnemyCount => modifiers.enemy_count,
            Self::TerrainDensity => modifiers.terrain_density,
        }
    }

    pub fn set(&self, modifiers: &mut RogueliteModifiers, value: f32) {
        match self {
            Self::GameSpeed => modifiers.game_speed = value,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness = value,
            Self::EnemyCount => modifiers.enemy_count = value,
            Self::TerrainDensity => modifiers.terrain_density = value,
        }
    }

    pub fn min_value(&self) -> f32 {
        0.2
    }

    pub fn max_value(&self) -> f32 {
        3.0
    }

    pub fn step(&self) -> f32 {
        0.1
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::GameSpeed => "Wave Speed",
            Self::EnemyEffectiveness => "Enemy Strength",
            Self::EnemyCount => "Enemy Count",
            Self::TerrainDensity => "Terrain",
        }
    }
}

/// Component for modifier slider value display text.
#[derive(Component)]
pub(super) struct ModifierSliderText {
    pub value: ModifierSliderValue,
}

/// Button to decrease a modifier slider value.
#[derive(Component)]
pub(super) struct ModifierSliderDownButton {
    pub value: ModifierSliderValue,
}

/// Button to increase a modifier slider value.
#[derive(Component)]
pub(super) struct ModifierSliderUpButton {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider track.
#[derive(Component)]
pub(super) struct ModifierSliderTrack {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider fill.
#[derive(Component)]
pub(super) struct ModifierSliderFill {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider handle.
#[derive(Component)]
pub(super) struct ModifierSliderHandle {
    pub value: ModifierSliderValue,
    pub is_dragging: bool,
}

/// Marker for the seed input text display.
#[derive(Component)]
pub(super) struct SeedInputText;

/// Marker for the seed input box background (clickable to focus).
#[derive(Component)]
pub(super) struct SeedInputBox;

/// Resource tracking the seed input state.
#[derive(Resource, Default)]
pub(super) struct SeedInputState {
    pub text: String,
    pub focused: bool,
}

/// Marker for the "Random" toggle button.
#[derive(Component)]
pub(super) struct SeedRandomButton;

// ── Toggle Modifier Components ──────────────────────────────────────────────

/// Expand/collapse arrow button for a toggle row.
#[derive(Component)]
pub(super) struct ToggleExpandButton(pub ToggleModifier);

/// Insight cost text for a locked toggle (despawned on unlock).
#[derive(Component)]
pub(super) struct ToggleUnlockButton(pub ToggleModifier);

/// The expandable description text (hidden by default).
#[derive(Component)]
pub(super) struct ToggleDescriptionNode(pub ToggleModifier);

/// The row container for a toggle (used for visual updates).
#[derive(Component)]
pub(super) struct ToggleRowContainer(pub ToggleModifier);

/// Resource tracking which toggle descriptions are expanded.
#[derive(Resource, Default)]
pub(super) struct ExpandedToggles(pub HashSet<ToggleModifier>);

/// Resource tracking which toggles are enabled for the pending run.
#[derive(Resource, Default, Clone)]
pub(super) struct PendingToggles {
    pub enabled: Vec<ToggleModifier>,
}

impl PendingToggles {
    pub fn is_enabled(&self, toggle: ToggleModifier) -> bool {
        self.enabled.contains(&toggle)
    }

    pub fn toggle(&mut self, toggle: ToggleModifier) {
        if let Some(pos) = self.enabled.iter().position(|t| *t == toggle) {
            self.enabled.remove(pos);
        } else {
            self.enabled.push(toggle);
        }
    }
}

// ── Left Panel (Run Summary) ────────────────────────────────────────────────

/// Marker for the run summary text container in the left panel.
#[derive(Component)]
pub(super) struct RunSummaryContent;

/// Marker for the right scrollable panel.
#[derive(Component)]
pub(super) struct ScrollableModifierList;

/// Marker for the left scrollable summary panel.
#[derive(Component)]
pub(super) struct ScrollableRunSummary;

// ── Confirmation Popup ──────────────────────────────────────────────────────

/// Marker for the unlock confirmation popup overlay.
#[derive(Component)]
pub(super) struct ConfirmUnlockPopup;

/// Actions for confirmation popup buttons.
#[derive(Component, Clone, Copy)]
pub(super) enum ConfirmUnlockAction {
    Confirm(ToggleModifier),
    Cancel,
}
