//! Bridges Bevy's `Activate` (a completed click, or controller confirm) to
//! `MouseClicked`, which every button action reads.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::game::input::messages::MouseClicked;

pub(crate) fn forward_activate_as_mouse_clicked(
    activate: On<Activate>,
    mut clicked: MessageWriter<MouseClicked>,
) {
    clicked.write(MouseClicked {
        button: activate.entity,
    });
}
