//! Multiplayer-specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the multiplayer game screen.
///
/// Used for bulk cleanup when exiting the multiplayer game state.
#[derive(Component)]
pub struct OnMultiplayerGameScreen;

/// Marker for ghost entities rendered on the guest from host state snapshots.
///
/// Ghost entities are lightweight visual representations — they have a mesh,
/// material, and transform but no simulation components. Their positions are
/// updated each frame from the latest snapshot received from the host.
#[derive(Component)]
pub struct GhostEntity;

/// Marker for ghost arrow projectiles rendered on the guest.
///
/// Unlike `GhostEntity` (which uses `NetworkEntityMap` for stable identity),
/// ghost arrows are ephemeral — all are despawned and re-spawned each frame
/// from the snapshot since arrows have no stable network ID.
#[derive(Component)]
pub struct GhostArrow;

/// Marker for ghost magic missile projectiles rendered on the guest.
///
/// Ephemeral like `GhostArrow` — despawned and re-spawned each frame.
#[derive(Component)]
pub struct GhostMagicMissile;

/// Marker for ghost beam effects rendered on the guest.
///
/// Ephemeral like `GhostArrow` — despawned and re-spawned each frame.
#[derive(Component)]
pub struct GhostBeam;

/// Preloaded mesh and material handles for ghost spell effects on the guest.
#[derive(Resource)]
pub struct GhostSpellAssets {
    /// Small pink circle mesh for ghost magic missiles.
    pub missile_mesh: Handle<Mesh>,
    /// Unlit pink material for ghost magic missiles.
    pub missile_material: Handle<StandardMaterial>,
    /// Small rectangle mesh for ghost beams (unit size, scaled by transform).
    pub beam_mesh: Handle<Mesh>,
    /// Unlit orange material for ghost beams.
    pub beam_material: Handle<StandardMaterial>,
}

impl GhostSpellAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        Self {
            missile_mesh: meshes.add(Circle::new(8.0)),
            missile_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.4, 0.7),
                unlit: true,
                ..default()
            }),
            beam_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
            beam_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.6, 0.1),
                unlit: true,
                ..default()
            }),
        }
    }
}

/// Marker for entities on the multiplayer score screen.
///
/// Used for targeted cleanup when leaving the score screen sub-state.
#[derive(Component)]
pub struct OnMpScoreScreen;

/// Tracks rematch readiness for both players on the score screen.
#[derive(Resource, Default)]
pub struct MpRematchState {
    pub local_ready: bool,
    pub remote_ready: bool,
}

/// Button actions for the multiplayer score screen.
#[derive(Component, Clone)]
pub enum MpScoreButtonAction {
    Rematch,
    Disconnect,
}

/// Marker for the rematch status text on the score screen.
#[derive(Component)]
pub struct RematchStatusText;

/// Marker resource inserted when both players agree to rematch.
///
/// Signals that the transition back to `MainMenu` → `Multiplayer` should
/// skip the connection phase and go straight to wizard select, preserving
/// the existing WebRTC connection.
#[derive(Resource)]
pub struct PendingRematch;

/// Marker for entities on the MP escape menu overlay.
#[derive(Component)]
pub struct OnMpPauseScreen;

/// Button actions for the MP escape menu.
#[derive(Component, Clone)]
pub enum MpPauseButtonAction {
    Resume,
    Disconnect,
}

/// Marker for entities on the MP disconnected overlay.
#[derive(Component)]
pub struct OnMpDisconnectedScreen;

/// Button action for the disconnected overlay.
#[derive(Component, Clone)]
pub struct MpDisconnectedButtonAction;
