//! Systems for version display.

use bevy::prelude::*;

use super::components::{GitHubButton, VersionText};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_URL: &str = "https://github.com/millidavids/court_wizard";

/// Spawns the version text as a clickable button in the bottom-left corner.
pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::hsla(0.0, 0.0, 0.3, 1.0)),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::hsla(0.0, 0.0, 0.15, 1.0)),
            VersionText,
            GitHubButton,
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

/// Handles GitHub button clicks to open the repository in a browser.
pub fn handle_github_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<GitHubButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Open URL in browser (WASM only)
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.open_with_url_and_target(GITHUB_URL, "_blank");
                }
            }
        }
    }
}

/// Updates GitHub button color on hover.
pub fn update_github_button_style(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<GitHubButton>),
    >,
) {
    const NORMAL_BG: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
    const NORMAL_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);
    const HOVER_BG: Color = Color::hsla(0.0, 0.0, 0.25, 1.0);
    const HOVER_BORDER: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);
    const PRESSED_BG: Color = Color::hsla(0.0, 0.0, 0.35, 1.0);
    const PRESSED_BORDER: Color = Color::hsla(0.0, 0.0, 0.5, 1.0);

    for (interaction, mut bg_color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BG.into();
                *border_color = BorderColor::all(PRESSED_BORDER);
            }
            Interaction::Hovered => {
                *bg_color = HOVER_BG.into();
                *border_color = BorderColor::all(HOVER_BORDER);
            }
            Interaction::None => {
                *bg_color = NORMAL_BG.into();
                *border_color = BorderColor::all(NORMAL_BORDER);
            }
        }
    }
}

/// Despawns the version button.
pub fn cleanup(mut commands: Commands, query: Query<Entity, With<VersionText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
