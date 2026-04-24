use bevy::prelude::*;

use crate::game::input::gamepad::resources::RadialHoveredSlot;
use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;

use super::components::{ActionBarLayoutProgress, InfiniteMana};
use super::messages::AssignSpellToSlot;
use super::radial;
use super::systems;

/// Plugin that manages the action bar UI.
#[derive(Default)]
pub struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfiniteMana>()
            .init_resource::<ActionBarLayoutProgress>()
            .add_message::<AssignSpellToSlot>()
            // Reset the morph to linear at the start of every run so the
            // slots spawn in their linear positions.
            .add_systems(
                OnEnter(InGameState::Running),
                reset_layout_progress.before(systems::spawn_action_bar),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                reset_layout_progress.before(systems::spawn_action_bar),
            )
            // Clear blocked spells before spawning the action bar.
            .add_systems(
                OnEnter(InGameState::Running),
                systems::clear_blocked_action_bar_spells,
            )
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_action_bar.after(systems::clear_blocked_action_bar_spells),
            )
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                systems::spawn_action_bar,
            )
            .add_systems(
                Update,
                (
                    systems::handle_slot_click.in_set(ButtonActionSet),
                    systems::handle_debug_mana_click.in_set(ButtonActionSet),
                    systems::handle_keyboard_input,
                    systems::highlight_keyboard_pressed_slots,
                )
                    .run_if(is_local_wizard_active),
            )
            // Layout morph: runs every frame while the action bar exists so
            // the buttons smoothly reorganize whenever the input device
            // changes, and so the current progress is applied immediately
            // after spawn.
            .add_systems(
                Update,
                radial::animate_action_bar_layout.run_if(
                    in_state(InGameState::Running).or(in_state(MultiplayerGameState::Running)),
                ),
            )
            .add_systems(
                Update,
                radial::highlight_radial_hovered_slot
                    .run_if(is_local_wizard_active)
                    .run_if(resource_changed::<RadialHoveredSlot>),
            )
            .add_systems(
                Update,
                (radial::flash_committed_slot, radial::tick_commit_flash)
                    .run_if(is_local_wizard_active),
            )
            .add_systems(
                Update,
                (
                    systems::update_action_bar_slots,
                    systems::handle_spell_assignment,
                )
                    .run_if(
                        is_local_wizard_active
                            .or(in_state(InGameState::SpellBook))
                            .or(in_state(MultiplayerGameState::SpellBook)),
                    ),
            );
    }
}

/// Resets the radial-vs-linear morph progress on gameplay start. Snaps
/// directly to the radial endpoint when a gamepad is the active input device
/// so the action bar renders in its final radial layout from the very first
/// frame — otherwise the controller in-game tutorial (which references the
/// radial controls) would talk about a layout that hadn't morphed yet.
fn reset_layout_progress(
    mut progress: ResMut<ActionBarLayoutProgress>,
    active: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
) {
    progress.0 = if active.is_gamepad() { 1.0 } else { 0.0 };
}
