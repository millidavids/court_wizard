mod animation_drivers;
mod electric_poison;
mod facing_direction;
mod fire_frost;
mod movement_helpers;
mod pending_effects;
mod sprite_utils;
mod temp_hp_ring;
mod vfx_tinting;

pub use animation_drivers::{
    apply_knockback_effects, finalize_dying_to_corpse, lay_corpse_flat, update_combat_animation,
    update_dying_animation, update_pulsing_animation, update_rising_animation,
    update_walking_animation,
};
pub use electric_poison::{
    update_airborne_units, update_electric_arc_visuals, update_poisoned, update_shocked,
    update_sickened,
};
pub use facing_direction::update_facing_direction;
pub use fire_frost::{emit_burning_unit_vfx, update_fire_dot, update_frost_accumulation};
pub use movement_helpers::{
    calculate_weighted_movement, find_closest_enemy_in_range, is_cc_immobilized,
    is_staging_attacker, update_melee_unit_targeting, update_timed_modifier,
};
pub use pending_effects::process_pending_damage_effects;
pub use sprite_utils::{
    archer_sprite_tint_for_team, corpse_material_for_team, create_corpse_sprite_materials,
    create_default_sprite_material, create_sprite_material, resurrect_corpse_as_infantry,
    sprite_tint_for_team,
};
pub use temp_hp_ring::{TempHpRingIndicator, spawn_temp_hp_rings, update_temp_hp_rings};
pub use vfx_tinting::update_persistent_effect_visuals;
