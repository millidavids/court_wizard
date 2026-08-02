use bevy::prelude::*;

use crate::game::run_conditions::any_exist;
use crate::state::{AppState, InGameState, MultiplayerGameState};

use super::components::{Notification, NotificationQueue};
use super::systems;

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationQueue>()
            .add_systems(
                Update,
                (
                    systems::queue_notifications,
                    systems::spawn_next_notification
                        .run_if(|q: Res<NotificationQueue>| !q.is_empty()),
                    systems::update_notifications.run_if(any_exist::<Notification>()),
                )
                    .chain(),
            )
            // Drop any still-draining battle notifications when the match ends so
            // they can't render over the score screen or the wizard tower. The
            // score screen is a sub-state of `InGame`, so its own OnEnter hook is
            // required — OnExit(InGame) doesn't fire until gameplay is left
            // entirely, and notifications outrank the score screen's z-index.
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                systems::clear_notifications,
            )
            .add_systems(
                OnEnter(MultiplayerGameState::ScoreScreen),
                systems::clear_notifications,
            )
            .add_systems(OnExit(AppState::InGame), systems::clear_notifications)
            .add_systems(
                OnExit(AppState::MultiplayerGame),
                systems::clear_notifications,
            );
    }
}
