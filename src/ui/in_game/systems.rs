//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::components::{Mana, Wizard};
use crate::state::InGameState;

/// Handles keyboard input during active gameplay.
///
/// - Escape: Pause the game, transitioning to `InGameState::Paused`
pub fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Paused);
    }
}

/// Spawns the gameplay HUD.
///
/// Creates a HUD with margins around screen edges containing:
/// - Mana bar in bottom right corner
pub fn spawn_hud(mut commands: Commands) {
    // Root HUD container (fullscreen with margins)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(HUD_MARGIN),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd, // Align to bottom
                align_items: AlignItems::FlexEnd,         // Align to right
                ..default()
            },
            HudRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Mana bar container (background)
            parent
                .spawn((
                    Node {
                        width: MANA_BAR_WIDTH,
                        height: MANA_BAR_HEIGHT,
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::FlexEnd, // Fill from right, empties from left
                        ..default()
                    },
                    BackgroundColor(MANA_BAR_BG_COLOR),
                ))
                .with_children(|parent| {
                    // Mana bar fill (starts at 100%, reduces from left)
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(MANA_BAR_FILL_COLOR),
                        ManaBarFill,
                    ));
                });
        });
}

/// Updates the mana bar width based on current wizard mana.
pub fn update_mana_bar(
    wizard_query: Query<&Mana, With<Wizard>>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
) {
    if let Ok(mana) = wizard_query.single()
        && let Ok(mut node) = mana_bar_query.single_mut()
    {
        let mana_percent = mana.percentage() * 100.0;
        node.width = Val::Percent(mana_percent);
    }
}
