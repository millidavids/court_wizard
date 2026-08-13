mod casting;
mod damage;
mod spawn;
mod zone_vfx;

pub use casting::handle_spike_growth_casting;
// Widened for the Arcane Crystal's Spike Growth infusion, which scatters
// scaled-down copies of the same zone.
pub use damage::{
    apply_spike_growth_damage, tick_lingering_poison, update_spike_storm_projectiles,
};
pub(crate) use spawn::spawn_spike_growth_zone;
pub use zone_vfx::{
    cleanup_spike_growth_zone, emit_spike_growth_rings, spike_storm_volley, update_death_garden,
};
