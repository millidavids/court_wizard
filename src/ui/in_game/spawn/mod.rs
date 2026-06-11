//! In-game HUD spawn and input handling.

mod hud_mp;
mod hud_sp;
mod input;

pub(crate) use hud_mp::spawn_mp_hud;
pub(crate) use hud_sp::spawn_hud;
pub(crate) use input::{
    block_spell_input_on_button_interaction, gamepad_hud_shortcuts, hud_button_action,
    keyboard_input,
};
