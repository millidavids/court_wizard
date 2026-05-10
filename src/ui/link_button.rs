//! Generic clickable URL link: marker component, click handler, shared hover/press glow.

use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use crate::game::input::messages::MouseClicked;

#[derive(Component)]
pub(crate) struct LinkButton(pub &'static str);

const LINK_GLOW_HOVER: Color = Color::srgba(1.0, 0.95, 0.7, 0.55);
const LINK_GLOW_PRESSED: Color = Color::srgba(1.0, 0.95, 0.6, 0.95);

pub(crate) fn link_glow_for(interaction: Interaction) -> Vec<ShadowStyle> {
    match interaction {
        Interaction::Pressed => vec![ShadowStyle {
            color: LINK_GLOW_PRESSED,
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(0.0),
            spread_radius: Val::Px(6.0),
            blur_radius: Val::Px(20.0),
        }],
        Interaction::Hovered => vec![ShadowStyle {
            color: LINK_GLOW_HOVER,
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(0.0),
            spread_radius: Val::Px(4.0),
            blur_radius: Val::Px(14.0),
        }],
        Interaction::None => vec![],
    }
}

pub(crate) fn handle_link_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&LinkButton>,
) {
    for event in button_clicked.read() {
        if let Ok(link) = button_query.get(event.button)
            && let Err(e) = webbrowser::open(link.0)
        {
            warn!("Failed to open URL {}: {}", link.0, e);
        }
    }
}
