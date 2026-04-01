use bevy::prelude::*;

use crate::game::game_mode::components::RogueliteModifiers;

/// Marker component for entities that belong to the roguelite modifiers screen.
#[derive(Component)]
pub(super) struct OnRogueliteModifiersScreen;

/// Actions for buttons on the modifiers screen.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModifierButtonAction {
    Continue,
    Back,
    Reset,
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

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::GameSpeed => "How quickly waves arrive",
            Self::EnemyEffectiveness => "How strong enemies are",
            Self::EnemyCount => "How many enemies spawn",
            Self::TerrainDensity => "How much terrain spawns",
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
    /// The current text in the seed input field.
    pub text: String,
    /// Whether the input field is focused (accepting keyboard input).
    pub focused: bool,
}

/// Marker for the "Random" toggle button.
#[derive(Component)]
pub(super) struct SeedRandomButton;
