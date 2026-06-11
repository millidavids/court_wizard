mod fish;
mod lily;
mod ripple;
mod spawn;

pub(in crate::game) use spawn::spawn_single_pond;

pub use fish::tick_pond_shocked;
pub use lily::{apply_frozen_pond_slow, apply_pond_wet, tick_wet_timer};
pub use ripple::{
    emit_pond_fog_particles, emit_pond_ripples, restore_pond_material_on_thaw,
    tick_pond_evaporation, tick_pond_frozen, update_frozen_pond_tint,
};
