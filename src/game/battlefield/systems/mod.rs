mod ambient;
mod lava_effects;
mod setup;
mod terrain_hazards;
mod water_effects;

pub use ambient::emit_ambient_motes;
pub use lava_effects::{emit_lava_fire_smoke, emit_lava_sparks};
pub(crate) use setup::load_battlefield_assets;
pub use setup::{setup_battlefield, spawn_castle_wall};
pub use terrain_hazards::{apply_lava_damage, apply_water_slow};
pub use water_effects::{emit_water_ripples, update_water_ripples};
