//! Finger of Death casting and beam spawn.

mod casting_logic;
mod damage;

pub use casting_logic::handle_finger_of_death_casting;
pub use damage::apply_finger_of_death_damage;
pub(crate) use damage::spawn_beam;
