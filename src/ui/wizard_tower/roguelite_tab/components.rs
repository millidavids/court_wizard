use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::game_mode::components::ToggleModifier;

/// Actions for roguelite tab buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RogueliteAction {
    StartRun,
    EndRun,
    ContinueRun,
    ChangeWizardType,
}

/// Identifies which modifier a slider controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub(crate) enum ModifierSliderValue {
    GameSpeed,
    EnemyEffectiveness,
    EnemyCount,
    TerrainDensity,
}

impl ModifierSliderValue {
    pub(crate) fn get(
        &self,
        modifiers: &crate::game::game_mode::components::RogueliteModifiers,
    ) -> f32 {
        match self {
            Self::GameSpeed => modifiers.game_speed,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness,
            Self::EnemyCount => modifiers.enemy_count,
            Self::TerrainDensity => modifiers.terrain_density,
        }
    }

    pub(crate) fn set(
        &self,
        modifiers: &mut crate::game::game_mode::components::RogueliteModifiers,
        value: f32,
    ) {
        match self {
            Self::GameSpeed => modifiers.game_speed = value,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness = value,
            Self::EnemyCount => modifiers.enemy_count = value,
            Self::TerrainDensity => modifiers.terrain_density = value,
        }
    }

    pub(crate) fn min_value(&self) -> f32 {
        0.2
    }

    pub(crate) fn max_value(&self) -> f32 {
        3.0
    }

    pub(crate) fn step(&self) -> f32 {
        0.1
    }

    pub(crate) fn label(&self) -> &'static str {
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
pub(crate) struct ModifierSliderText {
    pub value: ModifierSliderValue,
}

/// Button to decrease a modifier slider value.
#[derive(Component)]
pub(crate) struct ModifierSliderDownButton {
    pub value: ModifierSliderValue,
}

/// Button to increase a modifier slider value.
#[derive(Component)]
pub(crate) struct ModifierSliderUpButton {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider track.
#[derive(Component)]
pub(crate) struct ModifierSliderTrack {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider fill.
#[derive(Component)]
pub(crate) struct ModifierSliderFill {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider handle.
#[derive(Component)]
pub(crate) struct ModifierSliderHandle {
    pub value: ModifierSliderValue,
    pub is_dragging: bool,
}

/// Marker for the seed input text display.
#[derive(Component)]
pub(crate) struct SeedInputText;

/// Marker for the seed input box background (clickable to focus).
#[derive(Component)]
pub(crate) struct SeedInputBox;

/// Resource tracking the seed input state.
#[derive(Resource, Default)]
pub(crate) struct SeedInputState {
    pub text: String,
    pub focused: bool,
}

/// Marker for the "Random" toggle button.
#[derive(Component)]
pub(crate) struct SeedRandomButton;

// ── Toggle Modifier Components ──────────────────────────────────────────────

/// Expand/collapse arrow button for a toggle row.
#[derive(Component)]
pub(crate) struct ToggleExpandButton(pub ToggleModifier);

/// Insight cost text for a locked toggle (despawned on unlock).
#[derive(Component)]
pub(crate) struct ToggleUnlockButton(pub ToggleModifier);

/// The expandable description text (hidden by default).
#[derive(Component)]
pub(crate) struct ToggleDescriptionNode(pub ToggleModifier);

/// The row container for a toggle (used for visual updates).
#[derive(Component)]
pub(crate) struct ToggleRowContainer(pub ToggleModifier);

/// Resource tracking which toggle descriptions are expanded.
#[derive(Resource, Default)]
pub(crate) struct ExpandedToggles(pub HashSet<ToggleModifier>);

/// Resource tracking which toggles are enabled for the pending run.
#[derive(Resource, Default, Clone)]
pub(crate) struct PendingToggles {
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

/// Marker for the run summary text container in the left panel.
#[derive(Component)]
pub(crate) struct RunSummaryContent;

/// Marker for the confirmation popup overlay.
#[derive(Component)]
pub(crate) struct ConfirmUnlockPopup;

/// Actions for confirmation popup buttons.
#[derive(Component, Clone, Copy)]
pub(crate) enum ConfirmUnlockAction {
    Confirm(ToggleModifier),
    Cancel,
}

/// Marker for the scrollable content area on the right panel.
#[derive(Component)]
pub(crate) struct RogueliteScrollableContent;

/// Marker for the scrollable left panel.
#[derive(Component)]
pub(crate) struct RogueliteScrollableLeft;
