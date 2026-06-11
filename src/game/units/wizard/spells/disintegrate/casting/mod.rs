//! Disintegrate casting and damage application.

mod beam_actions;
mod cleanup;
mod damage;
mod handle_casting;
mod talent_config;

pub use cleanup::cleanup_beams_on_cancel;
pub use damage::apply_disintegrate_damage;
pub use handle_casting::handle_disintegrate_casting;
pub(crate) use talent_config::{TalentConfig, compute_talent_config};
