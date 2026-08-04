mod input;
mod keyboard_highlight;
mod primed_highlight;
mod spawn;

pub(super) use input::{handle_keyboard_input, handle_slot_click, handle_spell_assignment};
pub(super) use keyboard_highlight::{
    highlight_keyboard_pressed_slots, reset_action_bar_on_device_change, update_action_bar_slots,
};
pub(super) use primed_highlight::highlight_active_slot;
pub(super) use spawn::{clear_blocked_action_bar_spells, reset_layout_progress, spawn_action_bar};

// The spell book's hotkey boxes display the same slot contents the action bar
// does, so they resolve icons through the same helper.
pub(crate) use spawn::slot_icon;

#[cfg(debug_assertions)]
pub(super) use input::handle_debug_mana_click;
