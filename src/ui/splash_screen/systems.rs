use bevy::prelude::*;

use crate::state::AppState;

use super::components::{OnSplashScreen, SplashImage, SplashTimer};
use super::constants::*;

/// Spawns the splash screen UI: a black background with centered studio image and text.
pub(super) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(1000),
            OnSplashScreen,
            SplashTimer::new(DELAY_DURATION, FADE_IN_DURATION, HOLD_DURATION, FADE_OUT_DURATION),
        ))
        .with_children(|parent| {
            // Image wrapper — absolutely positioned, centered, full size
            // The wrapper centers the image without stretching it
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    OnSplashScreen,
                ))
                .with_children(|wrapper| {
                    wrapper.spawn((
                        ImageNode::new(asset_server.load(STUDIO_IMAGE_PATH))
                            .with_color(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                        Node {
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        OnSplashScreen,
                        SplashImage,
                    ));
                });

            // Text layer — centered on top of the image
            parent.spawn((
                Text::new("The Cult of"),
                TextFont::from_font_size(TEXT_FONT_SIZE),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                OnSplashScreen,
            ));

            parent.spawn((
                Text::new("David"),
                TextFont::from_font_size(DAVID_FONT_SIZE),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                OnSplashScreen,
            ));
        });
}

/// Ticks the splash timer, updates text/image alpha, and transitions to MainMenu when done.
pub(super) fn tick(
    time: Res<Time>,
    mut splash_query: Query<&mut SplashTimer>,
    mut text_query: Query<&mut TextColor, (With<OnSplashScreen>, Without<SplashImage>)>,
    mut image_query: Query<&mut ImageNode, With<SplashImage>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for mut timer in &mut splash_query {
        timer.elapsed += time.delta_secs();

        if timer.is_finished() {
            next_state.set(AppState::MainMenu);
            return;
        }

        let opacity = timer.opacity();

        // Apply opacity to the text
        let text_srgba = TEXT_COLOR.to_srgba();
        let text_faded = Color::srgba(text_srgba.red, text_srgba.green, text_srgba.blue, opacity);
        for mut text_color in &mut text_query {
            text_color.0 = text_faded;
        }

        // Apply opacity to the image (capped at IMAGE_MAX_OPACITY)
        let image_opacity = opacity * IMAGE_MAX_OPACITY;
        for mut image_node in &mut image_query {
            image_node.color = Color::srgba(1.0, 1.0, 1.0, image_opacity);
        }
    }
}

/// Despawns all splash screen entities.
pub(super) fn cleanup(mut commands: Commands, query: Query<Entity, With<SplashTimer>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
