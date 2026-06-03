//! Per-archetype gamepad → existing message translation.
//!
//! Each archetype-specific input system has a gamepad counterpart here that
//! reads the D-pad / face buttons and emits the same domain messages the
//! keyboard systems emit (`RunePressed`, `WeatherChangedMessage`, etc.). The
//! downstream gameplay systems remain source-agnostic.
//!
//! Bindings are hardcoded here for now; rebinding UI lives in a later phase.

use bevy::prelude::*;

use super::resources::ActiveInputDevice;
use crate::game::input::messages::{SpacebarHeld, SpacebarPressed, SpacebarReleased};
use crate::game::units::wizard::archetypes::arcanorouter::{
    messages::SliderAdjustMessage, resources::SliderType,
};
use crate::game::units::wizard::archetypes::gunslinger::messages::ReloadMessage;
use crate::game::units::wizard::archetypes::meteorologist::{
    messages::WeatherChangedMessage,
    resources::{WeatherState, WeatherType},
};
use crate::game::units::wizard::archetypes::roulette::messages::RouletteSpinMessage;
use crate::game::units::wizard::archetypes::runes::messages::RunePressed;
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::game::units::wizard::components::{LocalWizard, Mana, Wizard};

/// ArcanoRouter slider increment per D-pad press.
const ARCANOROUTER_STEP: f32 = 0.05;

/// Returns the currently active gamepad component, or `None` when mouse/keyboard is active.
fn active_gamepad<'a>(
    active: &ActiveInputDevice,
    gamepads: &'a Query<&Gamepad>,
) -> Option<&'a Gamepad> {
    active.gamepad_entity().and_then(|e| gamepads.get(e).ok())
}

// ---------------------------------------------------------------------------
// Universal Activate (Spacebar analog) → A button
// ---------------------------------------------------------------------------

/// Mirrors the keyboard "Activate" key onto the South face button (A/Cross).
/// Emits `SpacebarPressed/Held/Released` just like the keyboard path.
pub(super) fn translate_activate_button(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut pressed: MessageWriter<SpacebarPressed>,
    mut held: MessageWriter<SpacebarHeld>,
    mut released: MessageWriter<SpacebarReleased>,
) {
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    let button = GamepadButton::South;
    if gamepad.just_pressed(button) {
        pressed.write(SpacebarPressed);
    }
    if gamepad.pressed(button) {
        held.write(SpacebarHeld);
    }
    if gamepad.just_released(button) {
        released.write(SpacebarReleased);
    }
}

// ---------------------------------------------------------------------------
// RuneCaster: D-pad → runes (A handled by translate_activate_button)
// ---------------------------------------------------------------------------

pub(super) fn translate_runes(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut rune_pressed: MessageWriter<RunePressed>,
) {
    // Activate (South → ActivateRuneSequence) is handled by the keyboard-side
    // `detect_rune_input` which consumes `SpacebarPressed`; `translate_activate_button`
    // already emits that when South is pressed.
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    let rune_bindings: [(GamepadButton, Rune); 4] = [
        (GamepadButton::DPadUp, Rune::Q),
        (GamepadButton::DPadDown, Rune::W),
        (GamepadButton::DPadRight, Rune::E),
        (GamepadButton::DPadLeft, Rune::R),
    ];
    for (button, rune) in rune_bindings {
        if gamepad.just_pressed(button) {
            rune_pressed.write(RunePressed { rune });
        }
    }
}

// ---------------------------------------------------------------------------
// Randomancer: D-pad Up → spin
// ---------------------------------------------------------------------------

pub(super) fn translate_roulette(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut spin: MessageWriter<RouletteSpinMessage>,
) {
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::DPadUp) {
        spin.write(RouletteSpinMessage);
    }
}

// ---------------------------------------------------------------------------
// Meteorologist: D-pad Up/Down/Right → weather types
// ---------------------------------------------------------------------------

pub(super) fn translate_weather(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut weather: ResMut<WeatherState>,
    mut mana_query: Query<&mut Mana, (With<Wizard>, With<LocalWizard>)>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    let weather_bindings: [(GamepadButton, WeatherType); 3] = [
        (GamepadButton::DPadUp, WeatherType::Storm),
        (GamepadButton::DPadDown, WeatherType::Blizzard),
        (GamepadButton::DPadRight, WeatherType::Drought),
    ];
    let mut requested = None;
    for (button, weather_type) in weather_bindings {
        if gamepad.just_pressed(button) {
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
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut reload: MessageWriter<ReloadMessage>,
) {
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::DPadUp) {
        reload.write(ReloadMessage);
    }
}

// ---------------------------------------------------------------------------
// ArcanoRouter: D-pad 4-way → increment 4 sliders
// ---------------------------------------------------------------------------

pub(super) fn translate_arcanorouter(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut adjust: MessageWriter<SliderAdjustMessage>,
) {
    let Some(gamepad) = active_gamepad(&active, &gamepads) else {
        return;
    };
    let adjustments: [(GamepadButton, SliderType); 4] = [
        (GamepadButton::DPadUp, SliderType::Range),
        (GamepadButton::DPadDown, SliderType::Mana),
        (GamepadButton::DPadRight, SliderType::Power),
        (GamepadButton::DPadLeft, SliderType::Speed),
    ];
    for (button, slider) in adjustments {
        if gamepad.just_pressed(button) {
            adjust.write(SliderAdjustMessage {
                slider,
                delta: ARCANOROUTER_STEP,
            });
        }
    }
}
