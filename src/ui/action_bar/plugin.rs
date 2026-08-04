use bevy::prelude::*;

use crate::game::input::gamepad::resources::RadialHoveredSlot;
use crate::game::run_conditions::is_local_wizard_active;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::button_systems::sync_front_face_colors;
use crate::ui::plugin::ButtonActionSet;

#[cfg(debug_assertions)]
use super::components::InfiniteMana;
use super::components::{ActionBarLayoutProgress, ActionBarRoot};
use super::messages::AssignSpellToSlot;
use super::radial;
use super::run_conditions::action_bar_enabled;
use super::systems;

/// Plugin that manages the action bar UI.
#[derive(Default)]
pub struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionBarLayoutProgress>()
            .add_message::<AssignSpellToSlot>();

        #[cfg(debug_assertions)]
        app.init_resource::<InfiniteMana>();

        // Reset the morph to linear at the start of every run so the slots
        // spawn in their linear positions.
        app.add_systems(
            OnEnter(InGameState::Running),
            systems::reset_layout_progress.before(systems::spawn_action_bar),
        )
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::reset_layout_progress.before(systems::spawn_action_bar),
        )
        // Clear blocked spells before spawning the action bar.
        .add_systems(
            OnEnter(InGameState::Running),
            systems::clear_blocked_action_bar_spells,
        )
        .add_systems(
            OnEnter(InGameState::Running),
            systems::spawn_action_bar
                .after(systems::clear_blocked_action_bar_spells)
                .run_if(action_bar_enabled)
                // `Running` is re-entered every time the player closes the
                // spell book, pause menu, or cauldron, and nothing despawns the
                // bar in between — without this guard each round trip stacks
                // another full set of slots on top of the last.
                .run_if(not(any_with_component::<ActionBarRoot>)),
        )
        // Clear Shepherd-blocked (offensive) spells before spawning the action
        // bar in multiplayer too — this was SP-only, so a guest Shepherd's
        // offensive spells still appeared in the bar.
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::clear_blocked_action_bar_spells,
        )
        .add_systems(
            OnEnter(MultiplayerGameState::Running),
            systems::spawn_action_bar
                .after(systems::clear_blocked_action_bar_spells)
                .run_if(action_bar_enabled)
                // `Running` is re-entered every time the player closes the
                // spell book, pause menu, or cauldron, and nothing despawns the
                // bar in between — without this guard each round trip stacks
                // another full set of slots on top of the last.
                .run_if(not(any_with_component::<ActionBarRoot>)),
        )
        .add_systems(
            Update,
            (
                systems::handle_slot_click.in_set(ButtonActionSet),
                systems::handle_keyboard_input,
                // After the front-face sync, so the persistent primed-slot
                // write (which dirties `ButtonColors` and thus re-runs that
                // sync) can't reset the press highlight to rest the same frame.
                systems::highlight_keyboard_pressed_slots.after(sync_front_face_colors),
            )
                .run_if(is_local_wizard_active),
        )
        // On every mouse/keyboard ↔ controller switch, clear any stale
        // press/hover left on the slots so buttons don't read "stuck
        // depressed" after the transition. Runs before the highlight
        // systems so a held key or hovered radial slot re-lights at once.
        .add_systems(
            Update,
            systems::reset_action_bar_on_device_change
                .before(systems::highlight_keyboard_pressed_slots)
                .before(radial::highlight_radial_hovered_slot)
                .run_if(is_local_wizard_active)
                .run_if(
                    resource_changed::<crate::game::input::gamepad::resources::ActiveInputDevice>,
                ),
        )
        // Layout morph: runs every frame while the action bar exists so
        // the buttons smoothly reorganize whenever the input device
        // changes, and so the current progress is applied immediately
        // after spawn.
        .add_systems(
            Update,
            radial::animate_action_bar_layout
                .run_if(in_state(InGameState::Running).or(in_state(MultiplayerGameState::Running))),
        )
        .add_systems(
            Update,
            radial::highlight_radial_hovered_slot
                .after(sync_front_face_colors)
                .run_if(is_local_wizard_active)
                .run_if(resource_changed::<RadialHoveredSlot>),
        )
        .add_systems(
            Update,
            (radial::flash_committed_slot, radial::tick_commit_flash)
                .after(sync_front_face_colors)
                .run_if(is_local_wizard_active),
        )
        .add_systems(
            Update,
            (
                systems::update_action_bar_slots,
                systems::handle_spell_assignment,
                // After the slot refresh so a config change can't wipe the
                // highlight in the same frame it reassigns a slot, and before
                // the front-face sync so the new border reaches the visible
                // layer in that same frame.
                systems::highlight_active_slot
                    .after(systems::update_action_bar_slots)
                    .before(sync_front_face_colors),
            )
                .run_if(
                    is_local_wizard_active
                        .or(in_state(InGameState::SpellBook))
                        .or(in_state(MultiplayerGameState::SpellBook)),
                ),
        );

        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            systems::handle_debug_mana_click
                .in_set(ButtonActionSet)
                .run_if(is_local_wizard_active),
        );

        // Hide the debug INF button whenever the global F2 toggle is off.
        // Runs after the radial layout system (which also writes Visibility)
        // so this override always wins.
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            crate::game::debug_ui::sync_marker_visibility::<super::components::DebugManaButton>
                .after(radial::animate_action_bar_layout)
                .run_if(in_state(InGameState::Running).or(in_state(MultiplayerGameState::Running))),
        );
    }
}
