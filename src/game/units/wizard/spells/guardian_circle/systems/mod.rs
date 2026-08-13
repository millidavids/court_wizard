mod buff;

// Widened for the Arcane Crystal's Guardian Circle infusion, which refreshes the
// same ward from the crystal on a timer.
pub(crate) use buff::apply_guardian_circle_buff;
mod casting;
mod talent_reactions;

pub use buff::cleanup_guardian_circle_shielded;
pub use casting::handle_guardian_circle_casting;
pub use talent_reactions::{chain_ward_on_death, martyrdom_on_death, retaliating_wards_check};
