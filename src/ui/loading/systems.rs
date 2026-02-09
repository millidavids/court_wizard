use bevy::prelude::*;

use crate::ui::resources::CustomFont;

/// Marker component for loading screen root.
#[derive(Component)]
pub struct LoadingScreen;

/// Spawns the loading screen UI.
pub fn spawn_loading_screen(mut commands: Commands, custom_font: Res<CustomFont>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            LoadingScreen,
        ))
        .with_children(|parent| {
            // "Loading..." text
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font: custom_font.handle.clone(),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Despawns the loading screen UI.
pub fn despawn_loading_screen(
    mut commands: Commands,
    loading_screen_query: Query<Entity, With<LoadingScreen>>,
) {
    for entity in &loading_screen_query {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}
