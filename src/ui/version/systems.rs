//! Systems for version display.

use bevy::prelude::*;

use super::components::VersionText;
use crate::ui::link_button::{LinkButton, link_glow_for};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = "https://github.com/millidavids/court_wizard";

const TEXT_REST: Color = Color::hsla(0.0, 0.0, 0.6, 1.0);
const TEXT_HOVER: Color = Color::hsla(45.0, 0.6, 0.85, 1.0);
const TEXT_PRESSED: Color = Color::hsla(50.0, 1.0, 0.95, 1.0);

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BoxShadow(vec![]),
            VersionText,
            LinkButton(REPO_URL),
        ))
        .with_child((
            Text::new(format!("v{}", VERSION)),
            TextFont::from_font_size(14.0),
            TextColor(TEXT_REST),
        ));
}

pub(super) fn update_version_link_visuals(
    mut buttons: Query<
        (&Interaction, &Children, &mut BoxShadow),
        (Changed<Interaction>, With<VersionText>),
    >,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children, mut shadow) in &mut buttons {
        let text_color = match *interaction {
            Interaction::Pressed => TEXT_PRESSED,
            Interaction::Hovered => TEXT_HOVER,
            Interaction::None => TEXT_REST,
        };
        shadow.0 = link_glow_for(*interaction);
        for child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(text_color);
            }
        }
    }
}
