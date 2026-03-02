use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::state::{AppState, SplashState};

use super::components::{SplashAssets, SplashEntity, SplashTimer, SplashTransition};
use super::constants::*;

// ---------------------------------------------------------------------------
// Black substate — waits for the CRT shader to load before proceeding
// ---------------------------------------------------------------------------

/// Black substate setup: preloads all splash assets including the CRT shader.
pub(super) fn setup_black(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SplashAssets {
        rust_logo: asset_server.load(RUST_LOGO_PATH),
        bevy_logo: asset_server.load(BEVY_LOGO_PATH),
        studio_image: asset_server.load(STUDIO_IMAGE_PATH),
        shader: asset_server.load(SHADER_ASSET_PATH),
    });

    // Just a black screen — no timer, transitions are handled by tick_black
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(1000),
        SplashEntity,
    ));
}

/// Ticks the Black substate — waits for the CRT shader asset to be loaded,
/// then transitions to the next state. If skip_splash is enabled, goes
/// straight to MainMenu. Otherwise proceeds to the Language splash.
pub(super) fn tick_black(
    splash_assets: Option<Res<SplashAssets>>,
    asset_server: Res<AssetServer>,
    game_config: Option<Res<GameConfig>>,
    mut next_splash: ResMut<NextState<SplashState>>,
    mut next_app: ResMut<NextState<AppState>>,
) {
    let Some(assets) = splash_assets else {
        return;
    };

    // Wait until the shader is fully loaded
    if !asset_server.is_loaded_with_dependencies(&assets.shader) {
        return;
    }

    let skip = game_config
        .as_ref()
        .is_some_and(|config| config.skip_splash);

    if skip {
        next_app.set(AppState::MainMenu);
    } else {
        next_splash.set(SplashState::Language);
    }
}

// ---------------------------------------------------------------------------
// Setup systems — one per visual substate
// ---------------------------------------------------------------------------

/// Language substate: Rust logo with gray circle background.
pub(super) fn setup_language(mut commands: Commands, splash_assets: Res<SplashAssets>) {
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
            SplashTimer::new(HOLD_DURATION),
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
                    BackgroundColor(RUST_CIRCLE_COLOR),
                    BorderRadius::all(Val::Percent(50.0)),
                ))
                .with_children(|circle| {
                    circle.spawn((
                        ImageNode::new(splash_assets.rust_logo.clone()),
                        Node {
                            width: Val::Px(RUST_LOGO_SIZE),
                            height: Val::Px(RUST_LOGO_SIZE),
                            ..default()
                        },
                    ));
                });
        });
}

/// Engine substate: Bevy logo centered, no background.
pub(super) fn setup_engine(mut commands: Commands, splash_assets: Res<SplashAssets>) {
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
            SplashTimer::new(HOLD_DURATION),
            SplashTransition::NextSplash(SplashState::Studio),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(splash_assets.bevy_logo.clone()),
                Node {
                    height: Val::Px(BEVY_LOGO_HEIGHT),
                    ..default()
                },
            ));
        });
}

/// Studio substate: studio branding text with faint background image.
pub(super) fn setup_studio(mut commands: Commands, splash_assets: Res<SplashAssets>) {
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
            SplashTimer::new(HOLD_DURATION),
            SplashTransition::MainMenu,
        ))
        .with_children(|parent| {
            // Image wrapper — absolutely positioned, centered, full size
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|wrapper| {
                    wrapper.spawn((
                        ImageNode::new(splash_assets.studio_image.clone())
                            .with_color(Color::srgba(1.0, 1.0, 1.0, STUDIO_IMAGE_MAX_OPACITY)),
                        Node {
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });

            // Text layer — centered on top of the image
            parent.spawn((
                Text::new("The Cult of"),
                TextFont::from_font_size(TEXT_FONT_SIZE),
                TextColor(STUDIO_TEXT_COLOR),
            ));

            parent.spawn((
                Text::new("David"),
                TextFont::from_font_size(DAVID_FONT_SIZE),
                TextColor(STUDIO_TEXT_COLOR),
            ));
        });
}

// ---------------------------------------------------------------------------
// Shared tick — drives Language/Engine/Studio substates
// ---------------------------------------------------------------------------

pub(super) fn tick(
    time: Res<Time>,
    mut splash_query: Query<(&mut SplashTimer, &SplashTransition)>,
    mut next_splash: ResMut<NextState<SplashState>>,
    mut next_app: ResMut<NextState<AppState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for (mut timer, transition) in &mut splash_query {
        timer.elapsed += time.delta_secs();

        if timer.is_finished() {
            channel_change.write(ChannelChangeMessage);
            match transition {
                SplashTransition::NextSplash(state) => next_splash.set(*state),
                SplashTransition::MainMenu => next_app.set(AppState::MainMenu),
            }
            return;
        }
    }
}

/// Removes the preloaded splash assets when leaving the splash state entirely.
pub(super) fn cleanup_assets(mut commands: Commands) {
    commands.remove_resource::<SplashAssets>();
}
