//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::game::resources::CurrentLevel;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::components::{CastingState, LocalWizard, Mana, PrimedSpell};
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::systems::spawn_button;

/// Blocks spell input when any button is being interacted with.
///
/// This system runs before spell systems to prevent casting when clicking UI buttons.
pub(super) fn block_spell_input_on_button_interaction(
    button_query: Query<&Interaction, With<Button>>,
    mut block_spell_input: MessageWriter<BlockSpellInput>,
) {
    // Block spell input if any button is pressed or hovered
    for interaction in &button_query {
        if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
            block_spell_input.write(BlockSpellInput);
            return; // Only need to send once
        }
    }
}

/// Handles keyboard input during active gameplay (single-player only).
///
/// - Escape: Pause the game, transitioning to `InGameState::Paused`
///
/// In multiplayer, `mp_escape_key_handler` handles Escape instead.
pub(super) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    // Don't handle Escape in multiplayer — mp_escape_key_handler does it
    if mp_state.is_some() {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Paused);
    }
}

/// Spawns the king health bar as a vertical bar.
///
/// Used by both SP and MP HUDs.
fn spawn_king_health_bar(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|container| {
            // "King" label
            container.spawn((
                Text::new("King"),
                TextFont::from_font_size(KING_HEALTH_BAR_LABEL_FONT_SIZE),
                TextColor(Color::WHITE),
            ));

            // Health bar background
            container
                .spawn((
                    Node {
                        width: KING_HEALTH_BAR_WIDTH,
                        height: KING_HEALTH_BAR_HEIGHT,
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexEnd, // Fill grows upward from bottom
                        ..default()
                    },
                    BackgroundColor(KING_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(KING_HEALTH_BAR_BORDER_COLOR),
                    BorderRadius::all(Val::Px(3.0)),
                ))
                .with_children(|bar| {
                    // Health bar fill (anchored to bottom, height = percentage)
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(KING_HEALTH_BAR_FILL_COLOR),
                        BorderRadius::all(Val::Px(2.0)),
                        KingHealthBarFill,
                    ));
                });
        });
}

/// Spawns the gameplay HUD.
///
/// Creates a HUD with margins around screen edges containing:
/// - Spell book button in top left corner
/// - Level indicator and past victory in top right corner
/// - Mana bar in bottom right corner
/// - Cast bar below mana bar
pub(super) fn spawn_hud(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
) {
    // Root HUD container (fullscreen with margins)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(HUD_MARGIN),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            HudRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Top row (spell book button on left, level on right)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|row| {
                    // Button group (top-left)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        spawn_button(
                            buttons,
                            "Spells",
                            HudButtonAction::OpenSpellBook,
                            &BUTTON_STYLE,
                        );
                        spawn_button(
                            buttons,
                            "Cauldron",
                            HudButtonAction::OpenCauldronMenu,
                            &BUTTON_STYLE,
                        );
                    });

                    // Level and past victory display (top-right)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(5.0),
                        ..default()
                    })
                    .with_children(|level_container| {
                        // Level display
                        level_container.spawn((
                            Text::new(format!("Level: {}", current_level.0)),
                            TextFont::from_font_size(30.0),
                            TextColor(Color::WHITE),
                            LevelDisplay,
                        ));

                        // Past victory display (if exists)
                        if let Some(past_efficiency) =
                            config.efficiency_ratios.get(&current_level.0.to_string())
                        {
                            level_container.spawn((
                                Text::new(format!("Best: {:.1}%", past_efficiency * 100.0)),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                                PastVictoryDisplay,
                            ));
                        } else {
                            // Spawn empty placeholder so the component exists
                            level_container.spawn((
                                Text::new(""),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                                PastVictoryDisplay,
                            ));
                        }
                    });
                });

            // King health bar (middle, between top row and bottom bars)
            spawn_king_health_bar(parent);

            // Bottom-right bars container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: HUD_ELEMENT_GAP,
                    ..default()
                })
                .with_children(|bars| {
                    // Mana bar container (background)
                    bars.spawn((
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

                    // Cast bar container (background)
                    bars.spawn((
                        Node {
                            width: CAST_BAR_WIDTH,
                            height: CAST_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd, // Fill from right
                            ..default()
                        },
                        BackgroundColor(CAST_BAR_BG_COLOR),
                    ))
                    .with_children(|cast_bar| {
                        // Cast bar fill (starts at 0%)
                        cast_bar.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(CAST_BAR_FILL_COLOR),
                            CastBarFill,
                        ));

                        // Brewing overlay container (hidden by default, shown during brewing)
                        cast_bar
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Visibility::Hidden,
                                BrewingOverlay,
                            ))
                            .with_children(|overlay| {
                                overlay.spawn((
                                    Text::new("Brewing..."),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
                });
        });
}

/// Spawns a simplified HUD for multiplayer games.
///
/// Like `spawn_hud` but without the Cauldron button, level display, and past victory display
/// (those are single-player only concepts).
pub(super) fn spawn_mp_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(HUD_MARGIN),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            HudRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Top row (spell book button on left)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|row| {
                    // Button group (top-left) — Spells only, no Cauldron in MP
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        spawn_button(
                            buttons,
                            "Spells",
                            HudButtonAction::OpenSpellBook,
                            &BUTTON_STYLE,
                        );
                    });
                });

            // King health bar (middle, between top row and bottom bars)
            spawn_king_health_bar(parent);

            // Bottom-right bars container (mana bar + cast bar)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: HUD_ELEMENT_GAP,
                    ..default()
                })
                .with_children(|bars| {
                    // Mana bar
                    bars.spawn((
                        Node {
                            width: MANA_BAR_WIDTH,
                            height: MANA_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        },
                        BackgroundColor(MANA_BAR_BG_COLOR),
                    ))
                    .with_children(|mana| {
                        mana.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(MANA_BAR_FILL_COLOR),
                            ManaBarFill,
                        ));
                    });

                    // Cast bar
                    bars.spawn((
                        Node {
                            width: CAST_BAR_WIDTH,
                            height: CAST_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        },
                        BackgroundColor(CAST_BAR_BG_COLOR),
                    ))
                    .with_children(|cast_bar| {
                        cast_bar.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(CAST_BAR_FILL_COLOR),
                            CastBarFill,
                        ));
                    });
                });
        });
}

/// Handles HUD button click actions.
///
/// Sets the appropriate state for SP or MP depending on which is active.
pub(super) fn hud_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&HudButtonAction>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                HudButtonAction::OpenSpellBook => {
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::SpellBook);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::SpellBook);
                    }
                }
                HudButtonAction::OpenCauldronMenu => {
                    // Cauldron only exists in single-player
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::CauldronMenu);
                    }
                }
            }
        }
    }
}

/// Updates the mana bar width based on the local wizard's mana.
pub(super) fn update_mana_bar(
    wizard_query: Query<&Mana, With<LocalWizard>>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
) {
    if let Ok(mana) = wizard_query.single()
        && let Ok(mut node) = mana_bar_query.single_mut()
    {
        let mana_percent = mana.percentage() * 100.0;
        node.width = Val::Percent(mana_percent);
    }
}

/// Updates the cast bar width based on current wizard casting progress or brewing progress.
///
/// When the cauldron is brewing:
/// - Shows brewing progress with a grayed-out fill
/// - Displays "Brewing..." text overlay
///
/// Otherwise shows normal cast progress with gold fill.
pub(super) fn update_cast_bar(
    wizard_query: Query<(&CastingState, &PrimedSpell), With<LocalWizard>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut cast_bar_query: Query<(&mut Node, &mut BackgroundColor), With<CastBarFill>>,
    mut overlay_query: Query<&mut Visibility, With<BrewingOverlay>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    if let Ok((mut node, mut bg_color)) = cast_bar_query.single_mut() {
        if is_brewing {
            // Show brewing progress with gray fill
            if let Ok(state) = cauldron_query.single() {
                let progress_percent = state.progress() * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_BREWING_FILL_COLOR;
        } else {
            // Show normal cast progress with gold fill
            if let Ok((casting_state, primed_spell)) = wizard_query.single() {
                let progress_percent = casting_state.progress(primed_spell.cast_time) * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_FILL_COLOR;
        }
    }

    // Toggle brewing overlay visibility
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = if is_brewing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Updates the level display text when the current level changes.
pub(super) fn update_level_display(
    current_level: Res<CurrentLevel>,
    mut level_display_query: Query<&mut Text, With<LevelDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = level_display_query.single_mut()
    {
        **text = format!("Level: {}", current_level.0);
    }
}

/// Updates the past victory display text when the current level changes.
pub(super) fn update_past_victory_display(
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    mut past_victory_query: Query<&mut Text, With<PastVictoryDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = past_victory_query.single_mut()
    {
        if let Some(past_efficiency) = config.efficiency_ratios.get(&current_level.0.to_string()) {
            **text = format!("Best: {:.1}%", past_efficiency * 100.0);
        } else {
            **text = String::new();
        }
    }
}

/// Spawns the boss health bar when a boss appears and no bar exists yet.
pub(super) fn spawn_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<&Health, (With<Boss>, Without<Corpse>)>,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
) {
    let boss_exists = boss_query.iter().next().is_some();
    let bar_exists = bar_query.iter().next().is_some();

    if boss_exists && !bar_exists {
        // Top-center absolute container
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(BOSS_HEALTH_BAR_TOP_MARGIN),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BossHealthBarRoot,
                OnGameplayScreen,
            ))
            .with_children(|parent| {
                // Boss name
                parent.spawn((
                    Text::new("Boss"),
                    TextFont::from_font_size(BOSS_NAME_FONT_SIZE),
                    TextColor(Color::WHITE),
                ));

                // Health bar background
                parent
                    .spawn((
                        Node {
                            width: BOSS_HEALTH_BAR_WIDTH,
                            height: BOSS_HEALTH_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
                        BorderColor::all(BOSS_HEALTH_BAR_BORDER_COLOR),
                        BorderRadius::all(Val::Px(3.0)),
                    ))
                    .with_children(|bar| {
                        // Health bar fill
                        bar.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(BOSS_HEALTH_BAR_FILL_COLOR),
                            BorderRadius::all(Val::Px(2.0)),
                            BossHealthBarFill,
                        ));

                        // Health percentage text — absolute overlay centered on full bar
                        bar.spawn((Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                            .with_children(|overlay| {
                                overlay.spawn((
                                    Text::new("100%"),
                                    TextFont::from_font_size(BOSS_HEALTH_TEXT_FONT_SIZE),
                                    TextColor(Color::WHITE),
                                    BossHealthBarText,
                                ));
                            });
                    });
            });
    }
}

/// Updates the boss health bar fill and text. Despawns the bar when the boss dies.
pub(super) fn update_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<&Health, (With<Boss>, Without<Corpse>)>,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
    mut fill_query: Query<&mut Node, With<BossHealthBarFill>>,
    mut text_query: Query<&mut Text, With<BossHealthBarText>>,
) {
    if let Some(health) = boss_query.iter().next() {
        // Update fill width and text
        let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
        if let Ok(mut node) = fill_query.single_mut() {
            node.width = Val::Percent(hp_percent);
        }
        if let Ok(mut text) = text_query.single_mut() {
            **text = format!("{:.0}%", hp_percent);
        }
    } else {
        // Boss is dead or doesn't exist — remove the bar
        for entity in &bar_query {
            commands.entity(entity).despawn();
        }
    }
}

/// Updates the king health bar fill based on the local wizard's team's king health.
pub(super) fn update_king_health_bar(
    wizard_query: Query<&Team, With<LocalWizard>>,
    king_query: Query<(&Health, &Team), (With<King>, Without<Corpse>)>,
    mut fill_query: Query<&mut Node, With<KingHealthBarFill>>,
) {
    let Ok(wizard_team) = wizard_query.single() else {
        return;
    };
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };

    // Find the king matching the local wizard's team
    for (health, team) in &king_query {
        if team == wizard_team {
            let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
            fill_node.height = Val::Percent(hp_percent);
            return;
        }
    }

    // No matching king found (dead) — show empty bar
    fill_node.height = Val::Percent(0.0);
}
