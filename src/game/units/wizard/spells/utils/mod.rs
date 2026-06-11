//! Shared utility functions for spell systems.
//!
//! These functions are used across many spell implementations to avoid duplication.

mod casting_helpers;
mod cooldowns;
mod reticle;
mod ring_vfx;
mod spell_heal;
mod spell_math;
mod spell_origin;

// --- spell_origin ---
pub use spell_origin::LocalSpellOrigin;
pub use spell_origin::local_player_team;
pub(crate) use spell_origin::local_spell_origin_snapshot;
pub(crate) use spell_origin::update_local_spell_origin_snapshot;

// --- spell_math ---
pub(crate) use spell_math::clamp_cursor_to_spell_range_with_origin;
pub(crate) use spell_math::clamp_to_spell_range;
pub(crate) use spell_math::clamp_to_spell_range_ground;
pub(crate) use spell_math::distance_to_line_segment_xz;
pub(crate) use spell_math::ground_projected_range;
pub(crate) use spell_math::sphere_intersects_cylinder;
pub(crate) use spell_math::xz_distance;

// --- spell_heal ---
pub(crate) use spell_heal::PendingDefenderHeal;
pub(crate) use spell_heal::apply_pending_defender_heal;
pub(crate) use spell_heal::apply_spell_heal;

// --- reticle ---
pub(crate) use reticle::RETICLE_Y;
pub(crate) use reticle::SpellCircleIndicator;
pub(crate) use reticle::indicator_pulse_scale;
pub(crate) use reticle::make_reticle_mesh;
pub(crate) use reticle::spawn_circle_indicator;
pub(crate) use reticle::update_spell_indicators;

// --- ring_vfx ---
pub(crate) use ring_vfx::AnimatedRingParticle;
pub(crate) use ring_vfx::RING_SPAWN_INTERVAL;
pub(crate) use ring_vfx::animate_ring_particles;
pub(crate) use ring_vfx::spawn_ring_particle;

// --- casting_helpers ---
pub(crate) use casting_helpers::TargetAssistWorldPos;
pub(crate) use casting_helpers::UniqueHitTracker;
pub(crate) use casting_helpers::apply_target_assist;
pub(crate) use casting_helpers::build_wizard_input;
pub(crate) use casting_helpers::cleanup_spell_caster;
pub(crate) use casting_helpers::compute_target_assist;
pub(crate) use casting_helpers::get_cursor_world_position;
pub(crate) use casting_helpers::handle_spell_release;
pub(crate) use casting_helpers::try_start_cast_with_indicator;
pub(crate) use casting_helpers::update_indicator_position;

// --- cooldowns ---
pub(crate) use cooldowns::HasCooldownRemaining;
pub(crate) use cooldowns::block_spells_during_global_cooldown;
pub(crate) use cooldowns::insert_global_cooldown_on_cast;
pub(crate) use cooldowns::tick_global_cast_cooldown;
pub(crate) use cooldowns::tick_spell_cooldown;
