//! Weather state management, input handling, and status application.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::WeatherChangedMessage;
use super::resources::{WeatherState, WeatherType};
use crate::config::input_bindings::InputBindings;
use crate::game::units::components::{Corpse, ElectricCharge, Health};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{Mana, Wizard};

/// Applies the Drought healing reduction to a heal amount.
/// Returns the (possibly reduced) heal amount.
pub(crate) fn apply_dry_healing_reduction(amount: f32, is_dry: bool) -> f32 {
    if is_dry {
        amount * (1.0 - DRY_HEALING_REDUCTION)
    } else {
        amount
    }
}

/// Shared weather-switching logic used by both keyboard and button handlers.
/// Returns true if the switch was performed, false if blocked (cooldown, mana, etc.).
pub(crate) fn try_switch_weather(
    weather: &mut WeatherState,
    mana: &mut Mana,
    requested: WeatherType,
    writer: &mut MessageWriter<WeatherChangedMessage>,
) -> bool {
    if weather.cooldown > 0.0 {
        return false;
    }

    // Pressing the active weather clears it
    if weather.active == Some(requested) {
        weather.active = None;
        weather.intensity = INTENSITY_MIN;
        weather.time_active = 0.0;
        weather.cooldown = WEATHER_SWITCH_COOLDOWN;
        writer.write(WeatherChangedMessage);
        return true;
    }

    // Check mana
    if mana.current < WEATHER_MANA_COST {
        return false;
    }
    mana.consume(WEATHER_MANA_COST);

    weather.active = Some(requested);
    weather.intensity = INTENSITY_MIN;
    weather.time_active = 0.0;
    weather.cooldown = WEATHER_SWITCH_COOLDOWN;
    weather.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL;
    writer.write(WeatherChangedMessage);
    true
}

/// Resets weather state when entering gameplay.
pub fn reset_weather_state(mut weather: ResMut<WeatherState>) {
    *weather = WeatherState::default();
}

/// Handles weather key input to change weather.
pub fn handle_weather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut weather: ResMut<WeatherState>,
    mut mana_query: Query<&mut Mana, With<Wizard>>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    let weather_keys: [(Option<KeyCode>, WeatherType); 3] = [
        (bindings.meteorologist.weather_1, WeatherType::Storm),
        (bindings.meteorologist.weather_2, WeatherType::Blizzard),
        (bindings.meteorologist.weather_3, WeatherType::Drought),
    ];

    let mut requested = None;
    for (key_opt, weather_type) in weather_keys {
        if let Some(key) = key_opt
            && keyboard.just_pressed(key)
        {
            requested = Some(weather_type);
            break;
        }
    }

    let Some(requested) = requested else { return };

    let Ok(mut mana) = mana_query.single_mut() else {
        return;
    };
    try_switch_weather(&mut weather, &mut mana, requested, &mut writer);
}

/// Ticks cooldown and intensity timers.
pub fn tick_weather_timers(time: Res<Time>, mut weather: ResMut<WeatherState>) {
    let delta = time.delta_secs();

    // Tick cooldown
    if weather.cooldown > 0.0 {
        weather.cooldown = (weather.cooldown - delta).max(0.0);
    }

    // Tick intensity ramp
    if weather.active.is_some() {
        weather.time_active += delta;
        let t = (weather.time_active / INTENSITY_RAMP_TIME).min(1.0);
        weather.intensity = INTENSITY_MIN + (INTENSITY_MAX - INTENSITY_MIN) * t;
    }
}

/// Applies or removes weather status components on all living units.
#[allow(clippy::too_many_arguments)]
pub fn apply_weather_status(
    mut commands: Commands,
    weather: Res<WeatherState>,
    units_without_wet: Query<Entity, (Without<Corpse>, Without<WetModifier>, With<Health>)>,
    units_without_cold: Query<Entity, (Without<Corpse>, Without<ColdModifier>, With<Health>)>,
    units_without_dry: Query<Entity, (Without<Corpse>, Without<DryModifier>, With<Health>)>,
    units_without_charged: Query<Entity, (Without<Corpse>, Without<ChargedModifier>, With<Health>)>,
    _units_with_wet: Query<Entity, With<WetModifier>>,
    units_with_cold: Query<Entity, With<ColdModifier>>,
    units_with_dry: Query<Entity, With<DryModifier>>,
    units_with_charged: Query<Entity, With<ChargedModifier>>,
) {
    let active = weather.active;

    // Apply Wet + Charged if storm, remove if not
    if active == Some(WeatherType::Storm) {
        for entity in units_without_wet.iter() {
            commands.entity(entity).insert(WetModifier {
                intensity: weather.intensity,
                time_remaining: super::components::WET_DURATION,
            });
        }
        for entity in units_without_charged.iter() {
            commands.entity(entity).insert(ChargedModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        // Don't remove wet — let the timer expire naturally (10s duration).
        // Only remove Charged immediately (storm-exclusive effect).
        for entity in units_with_charged.iter() {
            commands.entity(entity).remove::<ChargedModifier>();
        }
    }

    // Apply Cold if blizzard, remove if not
    if active == Some(WeatherType::Blizzard) {
        for entity in units_without_cold.iter() {
            commands.entity(entity).insert(ColdModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        for entity in units_with_cold.iter() {
            commands.entity(entity).remove::<ColdModifier>();
        }
    }

    // Apply Dry if drought, remove if not
    if active == Some(WeatherType::Drought) {
        for entity in units_without_dry.iter() {
            commands.entity(entity).insert(DryModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        for entity in units_with_dry.iter() {
            commands.entity(entity).remove::<DryModifier>();
        }
    }
}

/// Syncs intensity value to existing weather status components.
pub fn update_weather_intensity(
    weather: Res<WeatherState>,
    mut wet_query: Query<&mut WetModifier>,
    mut cold_query: Query<&mut ColdModifier>,
    mut dry_query: Query<&mut DryModifier>,
    mut charged_query: Query<&mut ChargedModifier>,
) {
    let intensity = weather.intensity;
    match weather.active {
        Some(WeatherType::Storm) => {
            for mut m in wet_query.iter_mut() {
                m.intensity = intensity;
                // Refresh timer while storm is active
                m.time_remaining = super::components::WET_DURATION;
            }
            for mut m in charged_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        Some(WeatherType::Blizzard) => {
            for mut m in cold_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        Some(WeatherType::Drought) => {
            for mut m in dry_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        None => {}
    }
}

/// Spreads ElectricCharge from shocked units to nearby wet units.
/// Works for any source of Wet (ponds or storm weather).
pub fn spread_shock_to_wet(
    mut commands: Commands,
    weather: Option<Res<WeatherState>>,
    shocked_wet: Query<(&Transform, &ElectricCharge, &WetModifier), Without<Corpse>>,
    wet_targets: Query<
        (Entity, &Transform, Has<ElectricCharge>, Has<SpellShield>),
        (With<WetModifier>, Without<Corpse>),
    >,
) {
    // Use weather intensity for spread radius if storm is active, otherwise base radius
    let intensity = weather
        .as_ref()
        .filter(|w| w.active == Some(WeatherType::Storm))
        .map(|w| w.intensity)
        .unwrap_or(1.0);
    let spread_radius = WET_SHOCK_SPREAD_RADIUS * intensity;

    for (source_tf, charge, _wet) in shocked_wet.iter() {
        let source_pos = source_tf.translation;

        for (target_entity, target_tf, already_shocked, has_shield) in wet_targets.iter() {
            if already_shocked || has_shield {
                continue;
            }
            let dx = source_pos.x - target_tf.translation.x;
            let dz = source_pos.z - target_tf.translation.z;
            let dist_sq = dx * dx + dz * dz;

            if dist_sq <= spread_radius * spread_radius && dist_sq > 0.1 {
                // Spread a weaker charge (half the source's arc chance)
                let mut new_charge = ElectricCharge::new(0.0);
                new_charge.arc_chance = charge.arc_chance * 0.5;
                commands.entity(target_entity).insert(new_charge);
            }
        }
    }
}
