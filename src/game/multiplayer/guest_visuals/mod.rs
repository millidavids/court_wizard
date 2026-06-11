//! Guest visual: pick_material, spawn_spell_effect, handle_game_over.

mod game_over;
mod ghost_effect_spawn;
mod ghost_material;

pub use game_over::handle_game_over_message;
pub(super) use ghost_effect_spawn::spawn_spell_effect;
pub(super) use ghost_material::pick_material;
