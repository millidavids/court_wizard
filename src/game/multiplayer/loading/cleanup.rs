//! Multiplayer loading cleanup and camera setup/restore.

use bevy::prelude::*;

use super::queue::MpSpawnQueue;
use super::resources::{MpConfigBackup, MpLoadingSync};
use crate::config::GameConfig;
use crate::networking::resources::PeerRole;
use crate::networking::session::MultiplayerSession;

/// Cleans up multiplayer loading resources and restores any `GameConfig`
/// fields multiplayer overwrote — runs on every exit of `MultiplayerLoading`
/// (success or abort), so the seed/level/saved-terrain mutations never leak
/// into a subsequent single-player run.
pub fn cleanup_mp_loading(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    backup: Option<Res<MpConfigBackup>>,
) {
    commands.remove_resource::<MpSpawnQueue>();
    commands.remove_resource::<MpLoadingSync>();

    if let Some(backup) = backup {
        config.seed = backup.previous_seed;
        config.current_level = backup.previous_current_level;
        commands.remove_resource::<MpConfigBackup>();
    }
    // The terrain was already enqueued (the queue holds owned clones of each
    // `Saved*` element), so clearing here doesn't affect the in-flight spawn.
    config.saved_flora.clear();
    config.saved_trees.clear();
    config.saved_ponds.clear();
    config.saved_bushes.clear();
    config.saved_boulders.clear();
}

/// Sets up the camera for the multiplayer game based on role.
///
/// Host uses the standard camera position; guest gets a mirrored view.
pub fn setup_mp_camera(
    session: Res<MultiplayerSession>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut transform) = camera_query.single_mut()
        && session.role == PeerRole::Guest
        && !session.is_coop()
    {
        // Versus guest: mirrored camera (opposite corner looking at origin). The
        // CO-OP guest stands beside the host and keeps the single-player camera
        // angle, so it is intentionally excluded here.
        *transform = Transform::from_xyz(1000.0, 2500.0, -2500.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}

/// Restores the default camera when leaving multiplayer.
pub fn restore_camera(mut camera_query: Query<&mut Transform, With<Camera3d>>) {
    if let Ok(mut transform) = camera_query.single_mut() {
        *transform = Transform::from_xyz(-1000.0, 2500.0, 2500.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}
