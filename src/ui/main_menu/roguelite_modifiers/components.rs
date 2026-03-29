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
}

impl ModifierSliderValue {
    pub fn get(&self, modifiers: &RogueliteModifiers) -> f32 {
        match self {
            Self::GameSpeed => modifiers.game_speed,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness,
            Self::EnemyCount => modifiers.enemy_count,
        }
    }

    pub fn set(&self, modifiers: &mut RogueliteModifiers, value: f32) {
        match self {
            Self::GameSpeed => modifiers.game_speed = value,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness = value,
            Self::EnemyCount => modifiers.enemy_count = value,
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
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Self::GameSpeed => "How quickly waves arrive",
            Self::EnemyEffectiveness => "How strong enemies are",
            Self::EnemyCount => "How many enemies spawn",
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
