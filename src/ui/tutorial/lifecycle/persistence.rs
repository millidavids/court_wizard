use crate::config::save_data::{load_unified_save, new_unified_save, save_unified};

use super::super::resources::TutorialProgress;

pub(crate) fn load_tutorial_progress() -> TutorialProgress {
    let save_file = load_unified_save().unwrap_or_else(new_unified_save);
    TutorialProgress {
        completed: save_file.player.completed_tutorials.clone(),
    }
}

pub(crate) fn save_tutorial_progress(progress: &TutorialProgress) {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    save_file.player.completed_tutorials = progress.completed.clone();
    save_unified(&save_file);
}

pub(crate) fn reset_tutorial_progress() {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    save_file.player.completed_tutorials.clear();
    save_unified(&save_file);
}
