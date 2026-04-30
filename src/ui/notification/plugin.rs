use bevy::prelude::*;

use crate::game::run_conditions::any_exist;

use super::components::{Notification, NotificationQueue};
use super::systems;

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationQueue>().add_systems(
            Update,
            (
                systems::queue_notifications,
                systems::spawn_next_notification.run_if(|q: Res<NotificationQueue>| !q.is_empty()),
                systems::update_notifications.run_if(any_exist::<Notification>()),
            )
                .chain(),
        );
    }
}
