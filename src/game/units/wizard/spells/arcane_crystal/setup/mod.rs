//! Arcane crystal helpers, casting, and spawn.

mod casting;
mod helpers;
mod spawn;
mod visuals;

pub(super) use casting::handle_arcane_crystal_casting;
pub(crate) use helpers::compute_talent_params;
pub(super) use helpers::crystal_target_teams;
pub(super) use helpers::{
    clear_absorption_flags, crystal_beam_geometry, find_random_enemies_in_range,
    find_random_targets_in_range, increment_resonance, scaled_count, spell_echo_multiplier,
};
pub(crate) use spawn::spawn_permanent_crystal;
pub(super) use visuals::{
    cleanup_expired_crystal_beams, cleanup_expired_crystal_visuals, cleanup_expired_crystals,
    crystal_black_hole_interaction, despawn_out_of_range_crystal_spawns, update_crystal_visuals,
};
