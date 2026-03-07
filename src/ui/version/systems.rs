//! Systems for version display.

use bevy::prelude::*;

use super::components::VersionText;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Spawns the version text in the bottom-left corner.
pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new(format!("v{}", VERSION)),
        TextFont::from_font_size(14.0),
        TextColor(Color::hsla(0.0, 0.0, 0.6, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        VersionText,
    ));
}
