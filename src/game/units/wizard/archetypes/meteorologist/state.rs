//! Weather state management, input handling, and status application.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::WeatherChangedMessage;
use super::resources::{WeatherState, WeatherType};
use crate::config::input_bindings::InputBindings;
use crate::game::units::components::{Corpse, Health, Shocked};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{LocalWizard, Mana, Wizard};

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

    // Pressing the active weather clears it. Only the LOCAL slot is toggled —
    // this also naturally blocks a wizard from stacking its own weather.
    if weather.local.active == Some(requested) {
        weather.local.active = None;
        weather.local.intensity = INTENSITY_MIN;
        weather.local.time_active = 0.0;
        weather.cooldown = WEATHER_SWITCH_COOLDOWN;
        writer.write(WeatherChangedMessage);
        return true;
    }

    // Check mana (`can_afford` applies the wizard's mana_cost_multiplier).
    if !mana.can_afford(WEATHER_MANA_COST) {
        return false;
    }
    mana.consume(WEATHER_MANA_COST);

    weather.local.active = Some(requested);
    weather.local.intensity = INTENSITY_MIN;
    weather.local.time_active = 0.0;
    weather.local.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL;
    weather.cooldown = WEATHER_SWITCH_COOLDOWN;
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
    // Local wizard only: MP has two `Wizard` entities, so a bare `With<Wizard>`
    // single query errors and no weather fires.
    mut mana_query: Query<&mut Mana, (With<Wizard>, With<LocalWizard>)>,
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

/// Ramps one slot's intensity from its `time_active`. Runs on both peers from
/// the same formula, so the remote slot stays in sync without intensity packets.
fn ramp_slot(slot: &mut super::resources::WeatherSlot, delta: f32) {
    if slot.active.is_some() {
        slot.time_active += delta;
        let t = (slot.time_active / INTENSITY_RAMP_TIME).min(1.0);
        slot.intensity = INTENSITY_MIN + (INTENSITY_MAX - INTENSITY_MIN) * t;
    }
}

/// Ticks the switch cooldown and ramps BOTH slots' intensity.
pub fn tick_weather_timers(time: Res<Time>, mut weather: ResMut<WeatherState>) {
    let delta = time.delta_secs();

    // Tick cooldown (gates only the local player's next switch)
    if weather.cooldown > 0.0 {
        weather.cooldown = (weather.cooldown - delta).max(0.0);
    }

    // Tick intensity ramp for both this peer's weather and the opponent's.
    ramp_slot(&mut weather.local, delta);
    ramp_slot(&mut weather.remote, delta);
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
    // Union across BOTH slots: an effect is present if EITHER weather produces
    // it. Each weather maps to a DISJOINT set of components (Storm → Wet+Charged,
    // Blizzard → Cold, Drought → Dry), so a unit can carry several at once
    // (e.g. Wet from one player's Storm and Cold from the other's Blizzard).
    // Computing the union booleans first means a single if/else per effect — no
    // per-slot `else` that would wipe a modifier the other slot just added. When
    // both slots are the same weather, the stronger (max) intensity wins.
    let storm = weather.any_is(WeatherType::Storm);
    let blizzard = weather.any_is(WeatherType::Blizzard);
    let drought = weather.any_is(WeatherType::Drought);

    // Wet + Charged (storm)
    if storm {
        let intensity = weather.max_intensity_for(WeatherType::Storm);
        for entity in units_without_wet.iter() {
            commands.entity(entity).insert(WetModifier {
                intensity,
                time_remaining: super::components::WET_DURATION,
            });
        }
        for entity in units_without_charged.iter() {
            commands
                .entity(entity)
                .insert(ChargedModifier { intensity });
        }
    } else {
        // Don't remove wet — let the timer expire naturally (10s duration).
        // Only remove Charged immediately (storm-exclusive effect).
        for entity in units_with_charged.iter() {
            commands.entity(entity).remove::<ChargedModifier>();
        }
    }

    // Cold (blizzard)
    if blizzard {
        let intensity = weather.max_intensity_for(WeatherType::Blizzard);
        for entity in units_without_cold.iter() {
            commands.entity(entity).insert(ColdModifier { intensity });
        }
    } else {
        for entity in units_with_cold.iter() {
            commands.entity(entity).remove::<ColdModifier>();
        }
    }

    // Dry (drought)
    if drought {
        let intensity = weather.max_intensity_for(WeatherType::Drought);
        for entity in units_without_dry.iter() {
            commands.entity(entity).insert(DryModifier { intensity });
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
    // Each effect syncs to the MAX intensity across the slots producing it, so
    // stacked same-kind weather drives the stronger value. Independent `if`s (not
    // a single match) because multiple effects can be live at once.
    if weather.any_is(WeatherType::Storm) {
        let intensity = weather.max_intensity_for(WeatherType::Storm);
        for mut m in wet_query.iter_mut() {
            m.intensity = intensity;
            // Refresh timer while a storm is active
            m.time_remaining = super::components::WET_DURATION;
        }
        for mut m in charged_query.iter_mut() {
            m.intensity = intensity;
        }
    }
    if weather.any_is(WeatherType::Blizzard) {
        let intensity = weather.max_intensity_for(WeatherType::Blizzard);
        for mut m in cold_query.iter_mut() {
            m.intensity = intensity;
        }
    }
    if weather.any_is(WeatherType::Drought) {
        let intensity = weather.max_intensity_for(WeatherType::Drought);
        for mut m in dry_query.iter_mut() {
            m.intensity = intensity;
        }
    }
}

/// Spreads Shocked from shocked units to nearby wet units.
/// Works for any source of Wet (ponds or storm weather).
pub fn spread_shock_to_wet(
    mut commands: Commands,
    weather: Option<Res<WeatherState>>,
    shocked_wet: Query<(&Transform, &Shocked, &WetModifier), Without<Corpse>>,
    wet_targets: Query<
        (Entity, &Transform, Has<Shocked>, Has<SpellShield>),
        (With<WetModifier>, Without<Corpse>),
    >,
) {
    // Use storm intensity for spread radius if any storm is active, else base.
    let intensity = weather
        .as_ref()
        .filter(|w| w.any_is(WeatherType::Storm))
        .map(|w| w.max_intensity_for(WeatherType::Storm))
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
                let mut new_charge = Shocked::new(0.0);
                new_charge.arc_chance = charge.arc_chance * 0.5;
                commands.entity(target_entity).insert(new_charge);
            }
        }
    }
}
