mod ambience;
mod combat_state;
mod lifecycle;
mod shadows;
mod timers;

pub use ambience::{load_battle_ambience_assets, update_battle_ambience, update_crowd_ambience};
pub use combat_state::{
    activate_defenders_on_proximity, calculate_effectiveness, track_wizard_enemy_damage,
    wizard_has_not_damaged_enemies,
};
pub use lifecycle::{
    cleanup_for_replay, cleanup_game, init_level_from_config, reset_resources_for_replay,
    stop_all_sfx,
};
pub use shadows::{
    ShadowAssets, ShadowMaterial, preload_shadow_assets, spawn_terrain_shadow, spawn_unit_shadows,
    update_unit_shadows,
};
pub(super) use timers::{apply_game_speed, auto_pause_on_focus_loss};
pub use timers::{tick_attack_cycle, tick_elapsed_time};
