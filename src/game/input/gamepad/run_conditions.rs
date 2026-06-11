use bevy::prelude::*;

use super::resources::ActiveInputDevice;

pub(super) fn gamepad_active(active: Res<ActiveInputDevice>) -> bool {
    active.is_gamepad()
}
