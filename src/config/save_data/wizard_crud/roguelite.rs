use super::super::save_cache::{load_unified_save, new_unified_save, save_unified};
use super::super::save_structs::{RogueliteRun, SavedRogueliteRun};
use crate::config::resources::ActiveSave;

/// Save a completed roguelite run to the wizard's run history.
/// Caps at MAX_ROGUELITE_RUN_HISTORY entries (FIFO).
pub(crate) fn save_roguelite_run(active_save: &ActiveSave, run: RogueliteRun) {
    use crate::game::game_mode::components::MAX_ROGUELITE_RUN_HISTORY;

    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.run_history.push(run);
        // Trim oldest unsaved runs when over limit (single pass)
        let unsaved_count = wizard
            .roguelite
            .run_history
            .iter()
            .filter(|r| !r.saved)
            .count();
        if unsaved_count > MAX_ROGUELITE_RUN_HISTORY {
            let excess = unsaved_count - MAX_ROGUELITE_RUN_HISTORY;
            let mut removed = 0;
            wizard.roguelite.run_history.retain(|r| {
                if !r.saved && removed < excess {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    save_unified(&save_file);
}

/// Saves the current in-progress roguelite run to disk so it can be resumed later.
/// Called when returning to the wizard tower between levels.
pub(crate) fn save_current_roguelite_run(
    active_save: &ActiveSave,
    run: &crate::game::game_mode::components::RogueliteRunState,
    config: &crate::config::GameConfig,
    modifiers: Option<&crate::game::game_mode::components::RogueliteModifiers>,
    toggles: Option<&crate::game::game_mode::components::ActiveToggles>,
    seed: Option<u64>,
) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.current_run = Some(SavedRogueliteRun {
            started_at: run.started_at,
            current_level: config.current_level,
            wizard_type: config.wizard_type,
            level_stats: run.level_stats.clone(),
            modifiers: modifiers.cloned(),
            seed,
            active_toggles: toggles.map(|t| t.to_ids()).unwrap_or_default(),
            accessibility_assists: config.has_accessibility_assists(),
        });
    }

    save_unified(&save_file);
}

/// Clears the current in-progress roguelite run from disk.
/// Called when the run ends (victory, explicit abandon, or exit-to-menu mid-level).
pub(crate) fn clear_current_roguelite_run(active_save: &ActiveSave) {
    let Some(wizard_id) = &active_save.0 else {
        return;
    };

    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    if let Some(wizard) = save_file.wizards.iter_mut().find(|w| &w.id == wizard_id) {
        wizard.roguelite.current_run = None;
    }

    save_unified(&save_file);
}

/// Loads the current in-progress roguelite run from disk, if one exists.
pub(crate) fn load_current_roguelite_run(active_save: &ActiveSave) -> Option<SavedRogueliteRun> {
    let wizard_id = active_save.0.as_ref()?;
    let save_file = load_unified_save()?;
    let wizard = save_file.wizards.iter().find(|w| &w.id == wizard_id)?;
    wizard.roguelite.current_run.clone()
}

/// Toggle the saved status of a roguelite run identified by its `started_at` timestamp.
pub(crate) fn toggle_roguelite_run_saved(started_at: u64) {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);

    for wizard in &mut save_file.wizards {
        if let Some(run) = wizard
            .roguelite
            .run_history
            .iter_mut()
            .find(|r| r.started_at == started_at)
        {
            run.saved = !run.saved;
            break;
        }
    }

    save_unified(&save_file);
}
