//! Clicker — win a roguelite run using only the mouse.
//!
//! Rather than inspecting the player's keybinding config, this observes how the
//! run is actually played. Two trackers set a run-scoped flag on
//! [`RogueliteRunState`], and the victory check reads it.
//!
//! The mouse (and the controller's virtual cursor + trigger, which is its
//! equivalent) stays clean. What breaks eligibility is any input that selects or
//! fires something the pointer could otherwise do: the action-bar hotkeys, the
//! controller's radial menu, and the archetype keys / D-pad abilities.

use bevy::prelude::*;

use super::super::helpers::do_unlock;
use crate::config::input_bindings::BindingContext;
use crate::config::save_data::grant_achievement_insight;
use crate::config::{GameConfig, InputBindings, WizardType};
use crate::game::game_mode::components::{
    GameMode, ROGUELITE_MAX_LEVEL, RogueliteRunState, is_roguelite_mode,
};
use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::messages::ActionBarKeyPressed;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::{CurrentLevel, GameOutcome};

use super::super::messages::BattleEndedMessage;
use super::super::resources::*;

/// Controller equivalents of the archetype keys — the D-pad abilities plus
/// Activate. `PrimaryCast`/`SecondaryCast` are absent on purpose: the triggers
/// are the controller's mouse buttons. The stick is absent for the same reason —
/// it drives the virtual cursor.
const PAD_ABILITIES: [GamepadAction; 5] = [
    GamepadAction::Activate,
    GamepadAction::AbilityUp,
    GamepadAction::AbilityDown,
    GamepadAction::AbilityLeft,
    GamepadAction::AbilityRight,
];

/// True while a roguelite run is in progress and still mouse-only.
///
/// The `GameMode` check is load-bearing, not redundant: a dormant roguelite run
/// keeps `RogueliteRunState` alive in the wizard tower, and starting an Endless
/// battle from there inserts `GameMode::Endless` without removing it. Gating on
/// the resource alone would let keyboard use in an unrelated Endless or versus
/// battle permanently disqualify the paused roguelite run.
pub(crate) fn run_still_mouse_only(
    game_mode: Option<Res<GameMode>>,
    run: Option<Res<RogueliteRunState>>,
) -> bool {
    is_roguelite_mode(game_mode.as_deref()) && run.is_some_and(|r| !r.used_non_mouse_input)
}

/// Breaks mouse-only status when an action bar slot is selected by keyboard
/// hotkey or by the controller's radial menu.
///
/// `ActionBarKeyPressed` has exactly two writers — `detect_keyboard_input` and
/// the radial commit in `translate_triggers_to_mouse_messages` — so it covers
/// both without catching mouse clicks, which prime spells via `MouseClicked`.
pub(crate) fn track_action_bar_shortcut(
    mut msg: MessageReader<ActionBarKeyPressed>,
    mut run: ResMut<RogueliteRunState>,
) {
    if msg.read().next().is_some() {
        run.used_non_mouse_input = true;
    }
}

/// Breaks mouse-only status when an archetype ability or Activate is triggered
/// by key or D-pad.
pub(crate) fn track_gameplay_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    pad: Res<GamepadActionState>,
    bindings: Res<InputBindings>,
    config: Res<GameConfig>,
    mut run: ResMut<RogueliteRunState>,
) {
    // Skip the binding lookups on frames with no key edge — that is nearly
    // every frame, and `context_keys` allocates a Vec.
    if keyboard.get_just_pressed().next().is_some() {
        // Universal "Activate" — slot keys are covered by the action bar tracker.
        if bindings
            .universal
            .activate
            .is_some_and(|k| keyboard.just_pressed(k))
        {
            run.used_non_mouse_input = true;
            return;
        }

        // Bound keys for the archetype currently being played. Read live so the
        // Wizard Cycle toggle's mid-run archetype swaps are picked up.
        if let Some(ctx) = wizard_type_to_context(config.wizard_type)
            && bindings
                .context_keys(ctx)
                .iter()
                .any(|(_, key)| key.is_some_and(|k| keyboard.just_pressed(k)))
        {
            run.used_non_mouse_input = true;
            return;
        }
    }

    if PAD_ABILITIES.iter().any(|a| pad.just_pressed(*a)) {
        run.used_non_mouse_input = true;
    }
}

/// Checks if the player won a roguelite run without ever leaving the mouse.
pub(crate) fn check_clicker(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ClickerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    game_mode: Option<Res<GameMode>>,
    current_level: Res<CurrentLevel>,
    run: Option<Res<RogueliteRunState>>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory {
            continue;
        }
        if !is_roguelite_mode(game_mode.as_deref()) {
            continue;
        }
        if current_level.0 != ROGUELITE_MAX_LEVEL {
            continue;
        }
        if run.as_ref().is_none_or(|r| r.used_non_mouse_input) {
            continue;
        }

        do_unlock(&mut res, &mut events);
        grant_achievement_insight(ClickerAchievement::achievement_id());
    }
}

/// Maps a WizardType to its BindingContext, if it has archetype-specific bindings.
fn wizard_type_to_context(wizard_type: WizardType) -> Option<BindingContext> {
    match wizard_type {
        WizardType::RuneCaster => Some(BindingContext::RuneCaster),
        WizardType::Swordcerer => Some(BindingContext::Swordcerer),
        WizardType::Arcanorouter => Some(BindingContext::ArcanoRouter),
        WizardType::Meteorologist => Some(BindingContext::Meteorologist),
        WizardType::Warglock => Some(BindingContext::Warglock),
        // BoringOleMage, Randomancer, Excremage, Shepherd, Psychopath, Alchemist
        // have no archetype-specific keybindings
        _ => None,
    }
}
