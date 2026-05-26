use bevy::prelude::*;

pub struct LoadingUiPlugin;

impl Plugin for LoadingUiPlugin {
    fn build(&self, _app: &mut App) {
        // Neither SP nor MP shows a loading screen anymore: both complete in
        // a few imperceptible frames (SP via bulk `process_spawn_queue`, MP
        // via the matching bulk loop in `process_mp_spawn_queue`). The
        // screen's spawn/despawn helpers in `systems.rs` are kept so a
        // future long-running load (e.g. cinematics, big asset packs) can
        // re-attach them without re-implementing the UI.
    }
}
