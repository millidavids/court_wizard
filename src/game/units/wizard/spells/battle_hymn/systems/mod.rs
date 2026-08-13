mod aura;

// Widened for the Arcane Crystal's Battle Hymn infusion, which projects the same
// buff from the crystal instead of from a one-off cast.
pub(crate) use aura::apply_battle_hymn_buff;
mod casting;
mod effects;
mod song_motes;

pub use casting::handle_battle_hymn_casting;
pub use effects::update_battle_hymn_modifier;
pub use song_motes::emit_battle_hymn_song_motes;
