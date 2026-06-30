//! In-game bar updates: mana, ammo, cast, boss health, king health.

mod boss_bar_spawn;
mod boss_bar_update;
mod reload_prompt;
mod resource_bars;

pub(super) use boss_bar_spawn::spawn_boss_health_bar;
pub(super) use boss_bar_update::{update_boss_health_bar, update_ray_eye_health_bar};
pub(crate) use reload_prompt::spawn_reload_prompt;
pub(super) use resource_bars::{
    update_ammo_display, update_cast_bar, update_king_health_bar, update_level_clock,
    update_level_display, update_mana_bar, update_overlay_text, update_past_victory_display,
};
