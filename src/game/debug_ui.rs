//! Global F2 toggle that hides/shows developer-only UI affordances
//! (infinite-mana button, +10000 Insight button, hitbox cylinders).
//!
//! Default state is hidden so the trailer/release build look clean even
//! when filming on a debug binary. Pressing F2 in any state flips the
//! flag; consumers react to `DebugUiVisible` via change detection.

use bevy::prelude::*;

/// When `true`, debug-only UI elements (infinite mana button,
/// +10000 Insight button, hitbox cylinders) are visible.
#[cfg(debug_assertions)]
#[derive(Resource, Default)]
pub(crate) struct DebugUiVisible(pub bool);

#[cfg(debug_assertions)]
fn toggle_debug_ui_visible(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<DebugUiVisible>,
) {
    if keyboard.just_pressed(KeyCode::F2) {
        visible.0 = !visible.0;
    }
}

/// Generic system that drives the `Visibility` of any entity tagged with
/// marker `M` from the global F2 flag. Register one copy per marker.
#[cfg(debug_assertions)]
pub(crate) fn sync_marker_visibility<M: Component>(
    visible: Res<DebugUiVisible>,
    mut q: Query<&mut Visibility, With<M>>,
) {
    let target = if visible.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut q {
        if *vis != target {
            *vis = target;
        }
    }
}

pub(crate) struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(debug_assertions)]
        {
            _app.init_resource::<DebugUiVisible>()
                .add_systems(Update, toggle_debug_ui_visible);
        }
    }
}
