mod counters;
mod crud;
mod insight;
mod roguelite;
mod unlocks;

pub(crate) use counters::{
    accumulate_kill_stats, get_endless_best_stats, get_total_levels_completed,
    increment_games_played, increment_levels_completed, record_coop_guest_level_end,
    update_endless_best_stats,
};
pub(crate) use crud::{
    load_level_terrain_into_config, load_or_create_wizard, save_config_to_active_wizard,
    save_level_terrain,
};
pub(crate) use insight::{
    add_spell_research_progress, add_spell_talent_progress_batch, get_all_insight_bonuses,
    get_insight, get_spell_research_progress, get_spell_talent_progress,
    get_spell_talent_selections, get_unlocked_toggles, grant_achievement_insight, grant_insight,
    is_toggle_unlocked, set_insight_bonus_levels, set_spell_talent_selection, spend_insight,
    unlock_toggle,
};
pub(crate) use roguelite::{
    clear_current_roguelite_run, load_current_roguelite_run, save_current_roguelite_run,
    save_roguelite_run, toggle_roguelite_run_saved,
};
pub(crate) use unlocks::{
    clear_progress, unlock_achievement, unlock_ingredient, unlock_unit, unlock_wizard_type,
};

#[cfg(debug_assertions)]
pub(crate) use unlocks::unlock_everything_for_testing;
