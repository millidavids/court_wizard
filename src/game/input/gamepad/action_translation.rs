//! Per-archetype gamepad → existing message translation.
//!
//! Each archetype-specific input system has a gamepad counterpart here that
//! reads the D-pad / face buttons and emits the same domain messages the
//! keyboard systems emit (`RunePressed`, `WeatherChangedMessage`, etc.). The
//! downstream gameplay systems remain source-agnostic.
//!
//! Bindings are hardcoded here for now; rebinding UI lives in a later phase.

use bevy::prelude::*;

use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::messages::{SpacebarHeld, SpacebarPressed, SpacebarReleased};
use crate::game::units::wizard::archetypes::arcanorouter::{
    constants::SLIDER_KEY_STEP, messages::SliderAdjustMessage, resources::SliderType,
};
use crate::game::units::wizard::archetypes::gunslinger::messages::ReloadMessage;
use crate::game::units::wizard::archetypes::meteorologist::{
    messages::WeatherChangedMessage,
    resources::{WeatherState, WeatherType},
};
use crate::game::units::wizard::archetypes::roulette::messages::RouletteSpinMessage;
use crate::game::units::wizard::archetypes::runes::messages::{ActivateRuneSequence, RunePressed};
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::game::units::wizard::components::{LocalWizard, Mana, Wizard};

// ---------------------------------------------------------------------------
// Universal Activate (Spacebar analog) → A button
// ---------------------------------------------------------------------------

/// Mirrors the keyboard "Activate" key onto the `Activate` action (default South).
/// Emits `SpacebarPressed/Held/Released` just like the keyboard path.
pub(super) fn translate_activate_button(
    state: Res<GamepadActionState>,
    mut pressed: MessageWriter<SpacebarPressed>,
    mut held: MessageWriter<SpacebarHeld>,
    mut released: MessageWriter<SpacebarReleased>,
) {
    if state.just_pressed(GamepadAction::Activate) {
        pressed.write(SpacebarPressed);
    }
    if state.pressed(GamepadAction::Activate) {
        held.write(SpacebarHeld);
    }
    if state.just_released(GamepadAction::Activate) {
        released.write(SpacebarReleased);
    }
}

// ---------------------------------------------------------------------------
// RuneCaster: D-pad → runes, Activate (A/South) → invoke the sequence
// ---------------------------------------------------------------------------

pub(super) fn translate_runes(
    state: Res<GamepadActionState>,
    mut rune_pressed: MessageWriter<RunePressed>,
    mut rune_activate: MessageWriter<ActivateRuneSequence>,
) {
    for rune in Rune::ALL {
        if state.just_pressed(rune.dpad_action()) {
            rune_pressed.write(RunePressed { rune });
        }
    }
    // A / South invokes the built sequence — mirrors the keyboard activate key,
    // which writes `ActivateRuneSequence` in `detect_rune_input`. (The matching
    // `SpacebarPressed` that drives the cast comes from `translate_activate_button`.)
    if state.just_pressed(GamepadAction::Activate) {
        rune_activate.write(ActivateRuneSequence);
    }
}

// ---------------------------------------------------------------------------
// Randomancer: D-pad Up → spin
// ---------------------------------------------------------------------------

pub(super) fn translate_roulette(
    state: Res<GamepadActionState>,
    mut spin: MessageWriter<RouletteSpinMessage>,
) {
    if state.just_pressed(GamepadAction::AbilityUp) {
        spin.write(RouletteSpinMessage);
    }
}

// ---------------------------------------------------------------------------
// Meteorologist: D-pad Up/Down/Right → weather types
// ---------------------------------------------------------------------------

pub(super) fn translate_weather(
    state: Res<GamepadActionState>,
    mut weather: ResMut<WeatherState>,
    mut mana_query: Query<&mut Mana, (With<Wizard>, With<LocalWizard>)>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    let weather_bindings: [(GamepadAction, WeatherType); 3] = [
        (GamepadAction::AbilityUp, WeatherType::Storm),
        (GamepadAction::AbilityDown, WeatherType::Blizzard),
        (GamepadAction::AbilityRight, WeatherType::Drought),
    ];
    let mut requested = None;
    for (action, weather_type) in weather_bindings {
        if state.just_pressed(action) {
            requested = Some(weather_type);
            break;
        }
    }
    let Some(requested) = requested else { return };
    let Ok(mut mana) = mana_query.single_mut() else {
        return;
    };
    crate::game::units::wizard::archetypes::meteorologist::systems::try_switch_weather(
        &mut weather,
        &mut mana,
        requested,
        &mut writer,
    );
}

// ---------------------------------------------------------------------------
// Warglock (Gunslinger): D-pad Up → reload
// ---------------------------------------------------------------------------

pub(super) fn translate_warglock(
    state: Res<GamepadActionState>,
    mut reload: MessageWriter<ReloadMessage>,
) {
    if state.just_pressed(GamepadAction::AbilityUp) {
        reload.write(ReloadMessage);
    }
}

// ---------------------------------------------------------------------------
// ArcanoRouter: D-pad 4-way → increment 4 sliders
// ---------------------------------------------------------------------------

pub(super) fn translate_arcanorouter(
    state: Res<GamepadActionState>,
    mut adjust: MessageWriter<SliderAdjustMessage>,
) {
    let adjustments: [(GamepadAction, SliderType); 4] = [
        (GamepadAction::AbilityUp, SliderType::Range),
        (GamepadAction::AbilityDown, SliderType::Mana),
        (GamepadAction::AbilityRight, SliderType::Power),
        (GamepadAction::AbilityLeft, SliderType::Speed),
    ];
    for (action, slider) in adjustments {
        if state.just_pressed(action) {
            adjust.write(SliderAdjustMessage {
                slider,
                delta: SLIDER_KEY_STEP,
            });
        }
    }
}
