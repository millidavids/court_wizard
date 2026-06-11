use bevy::prelude::*;

use super::resources::ActiveInputDevice;

pub(crate) fn gamepad_active(active: Res<ActiveInputDevice>) -> bool {
    active.is_gamepad()
}
