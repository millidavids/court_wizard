//! Arcane crystal helpers, casting, and spawn.

mod absorb;
mod black_hole;
mod casting;
mod helpers;
mod spawn;
mod visuals;

pub(super) use absorb::{AbsorptionBookkeeping, absorb_into_crystal};
pub(super) use black_hole::crystal_black_hole_interaction;
pub(super) use casting::handle_arcane_crystal_casting;
pub(crate) use helpers::compute_talent_params;
pub(super) use helpers::crystal_target_teams;
pub(super) use helpers::{
    clear_absorption_flags, crystal_beam_geometry, destroy_crystal, find_random_enemies_in_range,
    find_random_targets_in_range, increment_resonance, register_infusion_spawn, scaled_count,
};
pub(crate) use spawn::spawn_permanent_crystal;
pub(super) use visuals::{
    cleanup_expired_crystal_beams, cleanup_expired_crystal_visuals, cleanup_expired_crystals,
    cleanup_orphaned_infusion_spawns, despawn_out_of_range_crystal_spawns, update_crystal_tint,
    update_crystal_visuals,
};
