use bevy::prelude::*;

use crate::game::units::wizard::archetypes::meteorologist::resources::WeatherType;

/// Root container for the weather bar UI.
#[derive(Component)]
pub(super) struct WeatherBarRoot;

/// A weather button in the bar, associated with a specific weather type.
#[derive(Component)]
pub(super) struct WeatherButton {
    pub weather: WeatherType,
}

/// The intensity fill bar inside a weather button.
#[derive(Component)]
pub(super) struct WeatherIntensityFill {
    pub weather: WeatherType,
}

/// Text showing the active weather name and description.
#[derive(Component)]
pub(super) struct WeatherStatusText;
