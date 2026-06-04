//! In-game HUD spawn and input handling.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::{GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::game::resources::{CurrentLevel, WaveState};
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::systems::spawn_button;

/// Blocks spell input when the mouse is interacting with a UI button so
/// clicking a HUD button doesn't simultaneously fire a spell. With a
/// gamepad, spells are triggered by RT (not by a UI cursor), and the hidden
/// OS cursor may sit over a HUD button at any time — gating on mouse mode
/// prevents those incidental hovers from silently killing RT casts.
pub(super) fn block_spell_input_on_button_interaction(
    active: Res<ActiveInputDevice>,
    button_query: Query<&Interaction, With<Button>>,
    mut block_spell_input: MessageWriter<BlockSpellInput>,
) {
    if active.is_gamepad() {
        return;
    }
    for interaction in &button_query {
        if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
            block_spell_input.write(BlockSpellInput);
            return;
        }
    }
}

/// Opens the spell book on gamepad X (West) and the cauldron menu on
/// gamepad Y (North) during gameplay. Matches the existing HUD-button
/// routing so state handling stays in one place.
#[allow(clippy::too_many_arguments)]
pub(super) fn gamepad_hud_shortcuts(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    current_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    config: Res<crate::config::GameConfig>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    // Gameplay must be actively running — `InGameState` exists only in
    // single-player, `MultiplayerGameState` only in multiplayer.
    let sp_running = current_state
        .as_ref()
        .is_some_and(|s| *s.get() == InGameState::Running);
    let mp_running = mp_state
        .as_ref()
        .is_some_and(|s| *s.get() == MultiplayerGameState::Running);
    if !sp_running && !mp_running {
        return;
    }
    let Some(gp_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gp_entity) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::West) && !config.wizard_type.uses_exclusive_casting() {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::SpellBook);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::SpellBook);
        }
    }
    if gamepad.just_pressed(GamepadButton::North) {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::CauldronMenu);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::CauldronMenu);
        }
    }
}

/// Handles pause input during active gameplay (single-player only):
/// Escape on keyboard or Start on the gamepad. **Not** B/East — in gameplay
/// that button is reserved for other actions and shouldn't stop the game.
///
/// In multiplayer, `mp_escape_key_handler` handles pause instead.
pub(super) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    current_state: Option<Res<State<InGameState>>>,
    next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    // Multiplayer pause is handled by `mp_escape_key_handler`; the
    // `InGameState` resources only exist in single-player anyway.
    if mp_state.is_some() {
        return;
    }
    let (Some(current_state), Some(mut next_in_game_state)) = (current_state, next_in_game_state)
    else {
        return;
    };
    if *current_state.get() != InGameState::Running {
        return;
    }
    let gamepad_start = active
        .gamepad_entity()
        .and_then(|e| gamepads.get(e).ok())
        .is_some_and(|g| g.just_pressed(GamepadButton::Start));
    if keyboard.just_pressed(KeyCode::Escape) || gamepad_start {
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
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(KING_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(KING_HEALTH_BAR_BORDER_COLOR),
                ))
                .with_children(|bar| {
                    // Health bar fill (anchored to bottom, height = percentage)
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(KING_HEALTH_BAR_FILL_COLOR),
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
    wave_state: Res<WaveState>,
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
                    // Button group + buff tracker (top-left)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|buttons| {
                        // Hide Spells button for archetypes that don't manage spells.
                        if !matches!(
                            config.wizard_type,
                            WizardType::Warglock | WizardType::Randomancer | WizardType::RuneCaster
                        ) {
                            spawn_button(
                                buttons,
                                "Spells",
                                HudButtonAction::OpenSpellBook,
                                &BUTTON_STYLE,
                            );
                        }
                        spawn_button(
                            buttons,
                            "Cauldron",
                            HudButtonAction::OpenCauldronMenu,
                            &BUTTON_STYLE,
                        );
                        // Buff tracker container (buff boxes will be spawned dynamically)
                        buttons.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(BUFF_BOX_GAP),
                                align_items: AlignItems::FlexStart,
                                ..default()
                            },
                            BuffTrackerContainer,
                        ));
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

                        // Wave display (only show if more than 1 wave)
                        if wave_state.total_waves > 1 {
                            level_container.spawn((
                                Text::new(format!(
                                    "Wave: {} / {}",
                                    wave_state.current_wave + 1,
                                    wave_state.total_waves
                                )),
                                TextFont::from_font_size(WAVE_DISPLAY_FONT_SIZE),
                                TextColor(WAVE_DISPLAY_COLOR),
                                WaveDisplay,
                            ));
                        }

                        // Level clock display
                        let clock_visibility = if config.show_level_clock {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        };
                        level_container.spawn((
                            Text::new("0:00"),
                            TextFont::from_font_size(LEVEL_CLOCK_FONT_SIZE),
                            TextColor(LEVEL_CLOCK_COLOR),
                            clock_visibility,
                            LevelClockDisplay,
                        ));
                    });
                });

            // King health bar (middle, between top row and bottom bars)
            spawn_king_health_bar(parent);

            let is_gunslinger = config.wizard_type == WizardType::Warglock;

            // Bottom-right bars container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: HUD_ELEMENT_GAP,
                    ..default()
                })
                .with_children(|bars| {
                    if is_gunslinger {
                        // Ammo display (replaces mana bar)
                        bars.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(3.0),
                                align_items: AlignItems::Center,
                                height: MANA_BAR_HEIGHT,
                                ..default()
                            },
                            AmmoDisplayContainer,
                        ))
                        .with_children(|ammo_row| {
                            // Ammo counter text
                            ammo_row.spawn((
                                Text::new("60 / 60"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                                AmmoCounterText,
                            ));

                            // Individual ammo pieces (will be spawned/updated dynamically)
                            let initial_pieces = GunType::MachineGun.max_ammo()
                                / GunType::MachineGun.ammo_per_ui_piece();
                            for i in 0..initial_pieces {
                                ammo_row.spawn((
                                    Node {
                                        width: Val::Px(4.0),
                                        height: Val::Px(14.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.0, 0.8, 0.2, 0.9)),
                                    AmmoPiece { index: i },
                                ));
                            }
                        });
                    } else {
                        // Standard mana bar container (background)
                        bars.spawn((
                            Node {
                                width: MANA_BAR_WIDTH,
                                height: MANA_BAR_HEIGHT,
                                border: UiRect::all(Val::Px(2.0)),
                                flex_direction: FlexDirection::Row,
                                ..default()
                            },
                            BackgroundColor(MANA_BAR_BG_COLOR),
                        ))
                        .with_children(|parent| {
                            // Current mana fill (blue, grows left to right)
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(MANA_BAR_FILL_COLOR),
                                ManaBarFill,
                            ));
                            // Reserved mana section (dark purple, right side)
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    BackgroundColor(MANA_BAR_RESERVED_COLOR),
                                    ManaBarReservedFill,
                                ))
                                .with_children(|reserved| {
                                    reserved.spawn((
                                        Text::new("Concentrating"),
                                        TextFont::from_font_size(8.0),
                                        TextColor(Color::srgba(0.7, 0.6, 1.0, 0.8)),
                                        ManaBarReservedText,
                                    ));
                                });
                        });
                    }

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
                                    BrewingOverlayText,
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
pub(super) fn spawn_mp_hud(mut commands: Commands, config: Res<GameConfig>) {
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
                    // Button group (top-left) — Spells + Cauldron
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        if !matches!(
                            config.wizard_type,
                            WizardType::Warglock | WizardType::Randomancer | WizardType::RuneCaster
                        ) {
                            spawn_button(
                                buttons,
                                "Spells",
                                HudButtonAction::OpenSpellBook,
                                &BUTTON_STYLE,
                            );
                        }
                        spawn_button(
                            buttons,
                            "Cauldron",
                            HudButtonAction::OpenCauldronMenu,
                            &BUTTON_STYLE,
                        );
                    });

                    // Match clock (top-right) — reuses the single-player Level
                    // Clock: same `LevelClockDisplay` marker, `update_level_clock`
                    // updater, M:SS format, and `show_level_clock` setting. The
                    // row's `JustifyContent::SpaceBetween` pushes this opposite the
                    // button group, so the clock is the row's only right-side child
                    // and needs no wrapper (mirrors the SP `spawn_hud` clock node).
                    let clock_visibility = if config.show_level_clock {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                    row.spawn((
                        Text::new("0:00"),
                        TextFont::from_font_size(LEVEL_CLOCK_FONT_SIZE),
                        TextColor(LEVEL_CLOCK_COLOR),
                        clock_visibility,
                        LevelClockDisplay,
                    ));
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
                    // Warglock shows its gun ammo instead of a mana bar (mirrors SP).
                    if config.wizard_type == WizardType::Warglock {
                        bars.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(3.0),
                                align_items: AlignItems::Center,
                                height: MANA_BAR_HEIGHT,
                                ..default()
                            },
                            AmmoDisplayContainer,
                        ))
                        .with_children(|ammo_row| {
                            ammo_row.spawn((
                                Text::new("60 / 60"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                                AmmoCounterText,
                            ));
                            let initial_pieces = GunType::MachineGun.max_ammo()
                                / GunType::MachineGun.ammo_per_ui_piece();
                            for i in 0..initial_pieces {
                                ammo_row.spawn((
                                    Node {
                                        width: Val::Px(4.0),
                                        height: Val::Px(14.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.0, 0.8, 0.2, 0.9)),
                                    AmmoPiece { index: i },
                                ));
                            }
                        });
                    } else {
                        // Mana bar — matches the SP layout (Row flex with a
                        // current-mana child and a reserved-mana partition for
                        // Concentration). Without the reserved child, the
                        // "Concentrating" partition is invisible in MP.
                        bars.spawn((
                            Node {
                                width: MANA_BAR_WIDTH,
                                height: MANA_BAR_HEIGHT,
                                border: UiRect::all(Val::Px(2.0)),
                                flex_direction: FlexDirection::Row,
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
                            mana.spawn((
                                Node {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(MANA_BAR_RESERVED_COLOR),
                                ManaBarReservedFill,
                            ))
                            .with_children(|reserved| {
                                reserved.spawn((
                                    Text::new("Concentrating"),
                                    TextFont::from_font_size(8.0),
                                    TextColor(Color::srgba(0.7, 0.6, 1.0, 0.8)),
                                    ManaBarReservedText,
                                ));
                            });
                        });
                    }

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

                        // Brewing/reload overlay (hidden by default) — without this
                        // the cast bar never showed "Brewing..." in multiplayer.
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
                                    BrewingOverlayText,
                                ));
                            });
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
    config: Res<crate::config::GameConfig>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                HudButtonAction::OpenSpellBook => {
                    // RuneCaster and Randomancer can only cast via their own mechanics
                    if config.wizard_type.uses_exclusive_casting() {
                        continue;
                    }
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::SpellBook);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::SpellBook);
                    }
                }
                HudButtonAction::OpenCauldronMenu => {
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::CauldronMenu);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::CauldronMenu);
                    }
                }
            }
        }
    }
}
