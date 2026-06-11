//! Dark mage spell effects: meteor explosions, lightning strikes, plague clouds, enrage.

mod spell_spawn;
mod spell_updates;
mod targeting;
mod vfx_updates;

// Public systems (registered in plugin.rs)
pub use spell_spawn::broadcast_plague_hazards;
pub use spell_updates::{
    dark_mage_enrage, update_lightning_strikes, update_meteor_explosions, update_plague_clouds,
};
pub use vfx_updates::{
    update_lightning_bolts, update_meteor_projectiles, update_plague_particles,
    update_visual_effects,
};

// pub(super) helpers — visible to the dark_mage module (ai.rs, plugin.rs, etc.)
pub(super) use spell_spawn::{
    spawn_lightning_strike, spawn_meteor_explosion, spawn_plague_cloud, spawn_telegraph_indicators,
};
pub(super) use targeting::{find_spell_target, spell_cooldown, telegraph_duration};
