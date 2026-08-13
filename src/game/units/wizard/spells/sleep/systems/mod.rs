mod casting;
mod effects;
mod wake;

pub use casting::handle_sleep_casting;
pub use effects::{update_narcoleptic_wave, update_night_terrors, update_sleep_modifiers};
// Widened for the Arcane Crystal's Sleep infusion, which pulses the same effect.
pub(crate) use effects::apply_sleep;
pub use wake::update_sleepwalkers;
