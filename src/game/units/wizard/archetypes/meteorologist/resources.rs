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

    /// Stable wire ordinal for multiplayer replication.
    pub const fn to_u8(self) -> u8 {
        match self {
            WeatherType::Storm => 0,
            WeatherType::Blizzard => 1,
            WeatherType::Drought => 2,
        }
    }

    /// Inverse of [`WeatherType::to_u8`].
    pub const fn from_u8(value: u8) -> Option<WeatherType> {
        match value {
            0 => Some(WeatherType::Storm),
            1 => Some(WeatherType::Blizzard),
            2 => Some(WeatherType::Drought),
            _ => None,
        }
    }
}

/// One owner's weather: their chosen condition plus its own intensity ramp and
/// lightning timer. Each peer's `WeatherState` holds two of these so two opposing
/// Meteorologists' different weathers can be active at the same time.
#[derive(Debug, Clone, Default)]
pub struct WeatherSlot {
    /// Currently active weather for this slot, or None for clear skies.
    pub active: Option<WeatherType>,
    /// Intensity multiplier (1.0 to max, grows over time).
    pub intensity: f32,
    /// How long this slot's weather has been active (seconds).
    pub time_active: f32,
    /// Timer for this slot's storm lightning strikes.
    pub lightning_timer: f32,
}

/// Global weather state managed by the Meteorologist.
///
/// Split into two owner slots: `local` is this peer's own weather (driven by
/// local input) and `remote` is the opponent's (driven by received messages,
/// always None in single-player). The host applies the UNION of both slots'
/// effects to its units; both peers render both slots' visuals.
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherState {
    /// This peer's own weather.
    pub local: WeatherSlot,
    /// The opponent's weather (None outside multiplayer).
    pub remote: WeatherSlot,
    /// Cooldown gating the LOCAL player's next weather switch (seconds).
    pub cooldown: f32,
}

impl WeatherState {
    /// True when EITHER slot is currently the given weather.
    pub fn any_is(&self, weather: WeatherType) -> bool {
        self.local.active == Some(weather) || self.remote.active == Some(weather)
    }

    /// Highest intensity among slots currently set to the given weather
    /// (0.0 if neither slot is that weather). Used so two stacked weathers of
    /// the same kind take the stronger effect.
    pub fn max_intensity_for(&self, weather: WeatherType) -> f32 {
        let mut intensity = 0.0_f32;
        if self.local.active == Some(weather) {
            intensity = intensity.max(self.local.intensity);
        }
        if self.remote.active == Some(weather) {
            intensity = intensity.max(self.remote.intensity);
        }
        intensity
    }
}
