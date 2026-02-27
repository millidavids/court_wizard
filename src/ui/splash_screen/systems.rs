use bevy::prelude::*;

use crate::state::{AppState, SplashState};

use super::components::{
    SplashEntity, SplashFadeBackground, SplashFadeImage, SplashFadeImageCapped, SplashFadeText,
    SplashTimer, SplashTransition,
};
use super::constants::*;

// ---------------------------------------------------------------------------
// Setup systems — one per substate
// ---------------------------------------------------------------------------

/// Language substate: Rust logo with gray circle background.
pub(super) fn setup_language(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(1000),
            SplashEntity,
            SplashTimer::new(DELAY_DURATION, FADE_IN_DURATION, HOLD_DURATION, FADE_OUT_DURATION),
            SplashTransition::NextSplash(SplashState::Engine),
        ))
        .with_children(|parent| {
            // Gray circle background
            parent
                .spawn((
                    Node {
                        width: Val::Px(RUST_CIRCLE_SIZE),
                        height: Val::Px(RUST_CIRCLE_SIZE),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::ZERO),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BorderRadius::all(Val::Percent(50.0)),
                    SplashEntity,
                    SplashFadeBackground {
                        color: RUST_CIRCLE_COLOR,
                    },
                ))
                .with_children(|circle| {
                    circle.spawn((
                        ImageNode::new(asset_server.load(RUST_LOGO_PATH))
                            .with_color(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                        Node {
                            width: Val::Px(RUST_LOGO_SIZE),
                            height: Val::Px(RUST_LOGO_SIZE),
                            ..default()
                        },
                        SplashEntity,
                        SplashFadeImage,
                    ));
                });
        });
}

/// Engine substate: Bevy logo centered, no background.
pub(super) fn setup_engine(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(1000),
            SplashEntity,
            SplashTimer::new(DELAY_DURATION, FADE_IN_DURATION, HOLD_DURATION, FADE_OUT_DURATION),
            SplashTransition::NextSplash(SplashState::Studio),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(asset_server.load(BEVY_LOGO_PATH))
                    .with_color(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                Node {
                    height: Val::Px(BEVY_LOGO_HEIGHT),
                    ..default()
                },
                SplashEntity,
                SplashFadeImage,
            ));
        });
}

/// Studio substate: studio branding text with faint background image.
pub(super) fn setup_studio(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            SplashEntity,
            SplashTimer::new(DELAY_DURATION, FADE_IN_DURATION, HOLD_DURATION, FADE_OUT_DURATION),
            SplashTransition::MainMenu,
        ))
        .with_children(|parent| {
            // Image wrapper — absolutely positioned, centered, full size
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
                    SplashEntity,
                ))
                .with_children(|wrapper| {
                    wrapper.spawn((
                        ImageNode::new(asset_server.load(STUDIO_IMAGE_PATH))
                            .with_color(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                        Node {
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        SplashEntity,
                        SplashFadeImageCapped {
                            max_opacity: STUDIO_IMAGE_MAX_OPACITY,
                        },
                    ));
                });

            // Text layer — centered on top of the image
            parent.spawn((
                Text::new("The Cult of"),
                TextFont::from_font_size(TEXT_FONT_SIZE),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                SplashEntity,
                SplashFadeText {
                    color: STUDIO_TEXT_COLOR,
                },
            ));

            parent.spawn((
                Text::new("David"),
                TextFont::from_font_size(DAVID_FONT_SIZE),
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                SplashEntity,
                SplashFadeText {
                    color: STUDIO_TEXT_COLOR,
                },
            ));
        });
}

// ---------------------------------------------------------------------------
// Shared tick — drives all splash substates
// ---------------------------------------------------------------------------

pub(super) fn tick(
    time: Res<Time>,
    mut splash_query: Query<(&mut SplashTimer, &SplashTransition)>,
    mut image_query: Query<&mut ImageNode, With<SplashFadeImage>>,
    mut bg_query: Query<(&mut BackgroundColor, &SplashFadeBackground)>,
    mut text_query: Query<(&mut TextColor, &SplashFadeText)>,
    mut capped_query: Query<(&mut ImageNode, &SplashFadeImageCapped), Without<SplashFadeImage>>,
    mut next_splash: ResMut<NextState<SplashState>>,
    mut next_app: ResMut<NextState<AppState>>,
) {
    for (mut timer, transition) in &mut splash_query {
        timer.elapsed += time.delta_secs();

        if timer.is_finished() {
            match transition {
                SplashTransition::NextSplash(state) => next_splash.set(*state),
                SplashTransition::MainMenu => next_app.set(AppState::MainMenu),
            }
            return;
        }

        let opacity = timer.opacity();

        // Fade images at full opacity
        for mut image_node in &mut image_query {
            image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity);
        }

        // Fade background colors
        for (mut bg, fade_bg) in &mut bg_query {
            let c = fade_bg.color.to_srgba();
            bg.0 = Color::srgba(c.red, c.green, c.blue, opacity);
        }

        // Fade text colors
        for (mut text_color, fade_text) in &mut text_query {
            let c = fade_text.color.to_srgba();
            text_color.0 = Color::srgba(c.red, c.green, c.blue, opacity);
        }

        // Fade images with capped max opacity
        for (mut image_node, capped) in &mut capped_query {
            image_node.color = Color::srgba(1.0, 1.0, 1.0, opacity * capped.max_opacity);
        }
    }
}

// ---------------------------------------------------------------------------
// Shared cleanup — despawns all entities with SplashEntity
// ---------------------------------------------------------------------------

pub(super) fn cleanup_substate(mut commands: Commands, query: Query<Entity, With<SplashEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
