//! Systems for version display.

use bevy::prelude::*;

use super::components::VersionText;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Spawns the version text in the bottom-left corner.
pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            VersionText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("v{}", VERSION)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
