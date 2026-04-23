//! Gamepad rumble triggers for significant gameplay events.
//!
//! Minimal MVP: rumble on entering the score screen, with different
//! patterns for victory vs defeat. More granular triggers (spell cast,
//! king damage) can be added later.

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;
use std::time::Duration;

use super::resources::ActiveInputDevice;
use crate::config::GameConfig;
use crate::game::resources::GameOutcome;

const VICTORY_RUMBLE_DURATION: Duration = Duration::from_millis(400);
const VICTORY_RUMBLE_INTENSITY: GamepadRumbleIntensity = GamepadRumbleIntensity {
    strong_motor: 0.3,
    weak_motor: 0.6,
};
const DEFEAT_RUMBLE_DURATION: Duration = Duration::from_millis(700);
const DEFEAT_RUMBLE_INTENSITY: GamepadRumbleIntensity = GamepadRumbleIntensity {
    strong_motor: 0.9,
    weak_motor: 0.6,
};

/// Triggers a rumble pulse on entering the score screen. Victory = light
/// celebratory pulse; defeat = heavier strong-motor hit.
pub(super) fn rumble_on_score_screen(
    config: Res<GameConfig>,
    active: Res<ActiveInputDevice>,
    outcome: Option<Res<GameOutcome>>,
    mut rumble: MessageWriter<GamepadRumbleRequest>,
) {
    if !config.rumble_enabled {
        return;
    }
    let Some(gamepad) = active.gamepad_entity() else {
        return;
    };
    let is_victory = matches!(outcome.as_deref(), Some(GameOutcome::Victory));
    let (duration, intensity) = if is_victory {
        (VICTORY_RUMBLE_DURATION, VICTORY_RUMBLE_INTENSITY)
    } else {
        (DEFEAT_RUMBLE_DURATION, DEFEAT_RUMBLE_INTENSITY)
    };
    rumble.write(GamepadRumbleRequest::Add {
        duration,
        intensity,
        gamepad,
    });
}
