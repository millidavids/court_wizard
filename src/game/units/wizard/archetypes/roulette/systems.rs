use bevy::prelude::*;
use rand::Rng;

use super::constants::*;
use super::messages::*;
use super::resources::*;
use crate::game::units::wizard::components::{CastingState, Wizard};
use crate::game::units::wizard::messages::PrimeSpellMessage;

/// Handles spacebar press to initiate a roulette spin.
pub fn handle_spin_trigger(
    mut messages: MessageReader<RouletteSpinMessage>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut state: ResMut<RouletteState>,
) {
    for _ in messages.read() {
        if matches!(state.phase, RoulettePhase::Idle | RoulettePhase::Selected { .. }) {
            let rng = &mut game_rng.0;
            let result_index = rng.gen_range(0..state.wheel_spells.len());
            state.phase = RoulettePhase::Spinning {
                elapsed: 0.0,
                current_highlight: 0,
                tick_timer: 0.0,
                result_index,
            };
        }
    }
}

/// Updates the spinning wheel animation and determines when it lands.
pub fn update_spin(
    time: Res<Time>,
    mut state: ResMut<RouletteState>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
) {
    // Read immutable data before mutable destructuring
    let total = state.wheel_spells.len();
    let dt = time.delta_secs();

    let RoulettePhase::Spinning {
        ref mut elapsed,
        ref mut current_highlight,
        ref mut tick_timer,
        result_index,
    } = state.phase
    else {
        return;
    };

    *elapsed += dt;
    *tick_timer += dt;

    // Easing: tick interval increases as we approach SPIN_DURATION (slows down)
    let progress = (*elapsed / SPIN_DURATION).min(1.0);
    let current_tick_interval = INITIAL_TICK_INTERVAL + progress * progress * 0.4;

    if *tick_timer >= current_tick_interval {
        *tick_timer = 0.0;
        *current_highlight = (*current_highlight + 1) % total;
    }

    // When spin duration expires, snap to result
    if *elapsed >= SPIN_DURATION {
        let result_idx = result_index;
        let spell = state.wheel_spells[result_idx];

        prime_spell.write(PrimeSpellMessage {
            spell: spell.primed_config().with_empowerment(ROULETTE_EMPOWERMENT),
        });
        state.phase = RoulettePhase::Selected { spell };
    }
}

/// After the wizard finishes casting (returns to Resting), reset the wheel to Idle.
pub fn reset_after_cast(
    mut state: ResMut<RouletteState>,
    wizard_query: Query<&CastingState, (With<Wizard>, Changed<CastingState>)>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !matches!(state.phase, RoulettePhase::Selected { .. }) {
        return;
    }

    // Reset when CastingState changes to Casting/Channeling (spells with cast times)
    for casting_state in &wizard_query {
        if !matches!(casting_state, CastingState::Resting) {
            state.phase = RoulettePhase::Idle;
            return;
        }
    }

    // Reset on left mouse click (covers instant-cast spells like Magic Missile)
    if mouse.just_pressed(MouseButton::Left) {
        state.phase = RoulettePhase::Idle;
    }
}

/// Resets the roulette state when exiting the Running state (e.g., going to pause, game over, etc.).
pub fn reset_roulette_state(mut state: ResMut<RouletteState>) {
    state.phase = RoulettePhase::Idle;
}
