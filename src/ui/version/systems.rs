//! Systems for version display.

use bevy::prelude::*;

use super::components::VersionText;
use crate::ui::button_systems::{PointerState, PointerVisual, PointerVisualChanges};
use crate::ui::components::UiButton;
use crate::ui::link_button::{LinkButton, link_glow_for};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = "https://github.com/millidavids/court_wizard";

const TEXT_REST: Color = Color::hsla(0.0, 0.0, 0.6, 1.0);
const TEXT_HOVER: Color = Color::hsla(45.0, 0.6, 0.85, 1.0);
const TEXT_PRESSED: Color = Color::hsla(50.0, 1.0, 0.95, 1.0);

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            UiButton,
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
    mut changes: PointerVisualChanges,
    mut buttons: Query<(PointerState, &Children, &mut BoxShadow), With<VersionText>>,
    mut text_query: Query<&mut TextColor>,
) {
    for entity in changes.entities() {
        let Ok((state, children, mut shadow)) = buttons.get_mut(entity) else {
            continue;
        };
        let visual = PointerVisual::of(state);
        let text_color = match visual {
            PointerVisual::Press => TEXT_PRESSED,
            PointerVisual::Hover => TEXT_HOVER,
            PointerVisual::Rest => TEXT_REST,
        };
        shadow.0 = link_glow_for(visual);
        for child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                *tc = TextColor(text_color);
            }
        }
    }
}
