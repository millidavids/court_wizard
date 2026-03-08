use bevy::prelude::*;

/// The three weather conditions the Meteorologist can invoke.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherType {
    /// Storm — applies Wet + Charged status. Rain, lightning strikes, shock spreads.
    Storm,
    /// Snow — applies Cold status. Frost slows stack harder, can freeze.
    Blizzard,
    /// Heat — applies Dry status. Fire spells create burning ground patches.
    Drought,
}

impl WeatherType {
    /// Display name for the UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            WeatherType::Storm => "Storm",
            WeatherType::Blizzard => "Blizzard",
            WeatherType::Drought => "Drought",
        }
    }

    /// Short description for the weather bar tooltip.
    pub const fn description(&self) -> &'static str {
        match self {
            WeatherType::Storm => "Wet + Charged. Shock spreads. Lightning strikes.",
            WeatherType::Blizzard => "Units are Cold. Frost slows harder. Can freeze.",
            WeatherType::Drought => "Units are Dry. Fire creates burning ground.",
        }
    }

    /// Hotkey label for the UI.
    pub const fn hotkey(&self) -> &'static str {
        match self {
            WeatherType::Storm => "Q",
            WeatherType::Blizzard => "W",
            WeatherType::Drought => "E",
        }
    }

    /// All weather types in order.
    pub const fn all() -> &'static [WeatherType] {
        &[
            WeatherType::Storm,
            WeatherType::Blizzard,
            WeatherType::Drought,
        ]
    }
}

/// Global weather state managed by the Meteorologist.
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherState {
    /// Currently active weather, or None for clear skies.
    pub active: Option<WeatherType>,
    /// Intensity multiplier (1.0 to max, grows over time).
    pub intensity: f32,
    /// Cooldown timer before weather can be changed again (seconds).
    pub cooldown: f32,
    /// How long the current weather has been active (seconds).
    pub time_active: f32,
    /// Timer for storm lightning strikes.
    pub lightning_timer: f32,
}
