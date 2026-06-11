//! Heat shimmer, plague/fog smoke, fire variants, and embers.

pub(crate) mod embers;
pub(crate) mod fire_smoke;
pub(crate) mod heat_shimmer;
pub(crate) mod plague_smoke;

// Public surface (was `pub` in the original file)
pub use embers::update_fire_embers;
pub use fire_smoke::{emit_fire_smoke_apex_puffs, spawn_fire_orange_smoke};
pub use heat_shimmer::{spawn_heat_shimmer, update_heat_shimmer};
pub use plague_smoke::{spawn_fog_smoke_puffs, spawn_plague_smoke_puffs, update_plague_smoke};

// pub(super) items from the original — visible to the vfx module (parent of area_effects)
pub(super) use fire_smoke::spawn_fire_smoke_puff;
