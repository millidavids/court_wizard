mod migration;
mod save_cache;
mod save_key_compat;
mod save_structs;
mod wizard_crud;

// Flatten the entire public surface so every path that was crate::config::save_data::X
// still resolves identically.
pub(crate) use migration::migrate_legacy_saves;
pub(crate) use save_cache::{
    current_timestamp, flush_save_cache, from_base64, load_unified_save, new_unified_save,
    save_cache_is_dirty, save_unified, to_base64,
};
pub(crate) use save_structs::{
    AchievementId, EndlessLevelBest, RogueliteRun, SavedBoulder, SavedBush, SavedCrystal,
    SavedFlora, SavedPond, SavedTrampling, SavedTree, SavedWall, UnifiedSaveFile,
};
pub(crate) use wizard_crud::{
    accumulate_kill_stats, add_spell_research_progress, add_spell_talent_progress_batch,
    clear_current_roguelite_run, clear_progress, get_all_insight_bonuses, get_endless_best_stats,
    get_insight, get_spell_research_progress, get_spell_talent_progress,
    get_spell_talent_selections, get_total_levels_completed, get_unlocked_toggles,
    grant_achievement_insight, grant_insight, increment_games_played, increment_levels_completed,
    is_toggle_unlocked, load_current_roguelite_run, load_level_terrain_into_config,
    load_or_create_wizard, record_coop_guest_level_end, save_config_to_active_wizard,
    save_current_roguelite_run, save_level_terrain, save_roguelite_run, set_insight_bonus_levels,
    set_spell_talent_selection, spend_insight, toggle_roguelite_run_saved, unlock_achievement,
    unlock_ingredient, unlock_toggle, unlock_unit, unlock_wizard_type, update_endless_best_stats,
};

#[cfg(debug_assertions)]
pub(crate) use wizard_crud::unlock_everything_for_testing;
