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
