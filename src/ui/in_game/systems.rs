//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::brews::BrewEffect;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::game::messages::WaveSpawnedMessage;
use crate::game::resources::{CurrentLevel, KillStats, WaveState};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::hags::components::{Hag, HagIdentity, PermanentlyDead};
use crate::game::units::boss::lich::Lich;
use crate::game::units::boss::lich::components::{LichPhase, SoulPower};
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::archetypes::gunslinger::{GunState, GunType};
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
    current_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    // Don't handle Escape in multiplayer — mp_escape_key_handler does it
    if mp_state.is_some() {
        return;
    }
    // Only handle escape when actually running (not in menus with urgent mode)
    if *current_state.get() != InGameState::Running {
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
                        if !matches!(config.wizard_type, WizardType::Warglock | WizardType::Randomancer | WizardType::RuneCaster) {
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
                    // Button group (top-left) — Spells only, no Cauldron in MP
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        if !matches!(config.wizard_type, WizardType::Warglock | WizardType::Randomancer | WizardType::RuneCaster) {
                            spawn_button(
                                buttons,
                                "Spells",
                                HudButtonAction::OpenSpellBook,
                                &BUTTON_STYLE,
                            );
                        }
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
                    // Cauldron only exists in single-player
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::CauldronMenu);
                    }
                }
            }
        }
    }
}

/// Updates the mana bar width and reserved section based on current mana and concentration.
pub(super) fn update_mana_bar(
    wizard_query: Query<&Mana, With<LocalWizard>>,
    concentration_spells: Query<&ConcentrationSpell>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
    mut reserved_bar_query: Query<
        &mut Node,
        (With<ManaBarReservedFill>, Without<ManaBarFill>),
    >,
) {
    if let Ok(mana) = wizard_query.single() {
        let reserved: f32 = concentration_spells.iter().map(|c| c.mana_cost).sum();
        let reserved_pct = (reserved / mana.max).min(1.0) * 100.0;
        let mana_pct = (mana.percentage() * 100.0).min(100.0 - reserved_pct);

        if let Ok(mut node) = mana_bar_query.single_mut() {
            node.width = Val::Percent(mana_pct);
        }
        if let Ok(mut node) = reserved_bar_query.single_mut() {
            node.width = Val::Percent(reserved_pct);
        }
    }
}

/// Updates the ammo display for the gunslinger archetype.
pub(super) fn update_ammo_display(
    gun_state: Option<Res<GunState>>,
    mut ammo_pieces: Query<(&AmmoPiece, &mut BackgroundColor)>,
    mut counter_text: Query<&mut Text, With<AmmoCounterText>>,
) {
    let Some(gs) = gun_state else {
        return;
    };

    let gun = gs.selected_gun;
    let ammo = gs.current_ammo();
    let per_piece = gun.ammo_per_ui_piece();
    let max_pieces = ammo.max / per_piece;

    // Update counter text
    if let Ok(mut text) = counter_text.single_mut() {
        **text = format!("{} / {}", ammo.current, ammo.max);
    }

    // Update ammo piece colors
    let lit_color = Color::srgba(1.0, 0.8, 0.2, 0.9);
    let dim_color = Color::srgba(0.3, 0.3, 0.3, 0.4);
    let reload_color = Color::srgba(0.5, 0.7, 1.0, 0.7);

    for (piece, mut bg) in &mut ammo_pieces {
        if piece.index >= max_pieces {
            bg.0 = Color::NONE;
            continue;
        }

        let ammo_at_piece = (piece.index + 1) * per_piece;

        if ammo.reloading {
            // During reload, progressively light up pieces
            let reloaded_ammo = (ammo.reload_progress() * ammo.max as f32) as u32;
            bg.0 = if ammo_at_piece <= reloaded_ammo {
                reload_color
            } else {
                dim_color
            };
        } else {
            bg.0 = if ammo_at_piece <= ammo.current {
                lit_color
            } else {
                dim_color
            };
        }
    }
}

/// Updates the cast bar width based on current wizard casting progress, brewing progress,
/// or reload progress for the gunslinger.
pub(super) fn update_cast_bar(
    wizard_query: Query<(&CastingState, &PrimedSpell), With<LocalWizard>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    gun_state: Option<Res<GunState>>,
    mut cast_bar_query: Query<(&mut Node, &mut BackgroundColor), With<CastBarFill>>,
    mut overlay_query: Query<&mut Visibility, With<BrewingOverlay>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    // Check if gunslinger is reloading
    let reload_progress = gun_state.as_ref().and_then(|gs| {
        let ammo = gs.current_ammo();
        if ammo.reloading {
            Some(ammo.reload_progress())
        } else {
            None
        }
    });

    if let Ok((mut node, mut bg_color)) = cast_bar_query.single_mut() {
        if let Some(progress) = reload_progress {
            node.width = Val::Percent(progress * 100.0);
            bg_color.0 = RELOAD_BAR_COLOR;
        } else if is_brewing {
            if let Ok(state) = cauldron_query.single() {
                let progress_percent = state.progress() * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_BREWING_FILL_COLOR;
        } else {
            if let Ok((casting_state, primed_spell)) = wizard_query.single() {
                let progress_percent = casting_state.progress(primed_spell.cast_time) * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_FILL_COLOR;
        }
    }

    // Toggle brewing/reload overlay visibility and text
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = if reload_progress.is_some() || is_brewing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Updates the overlay text to show "Reloading..." or "Brewing..." as appropriate.
pub(super) fn update_overlay_text(
    gun_state: Option<Res<GunState>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut text_query: Query<&mut Text, With<BrewingOverlayText>>,
) {
    let is_reloading = gun_state
        .as_ref()
        .is_some_and(|gs| gs.current_ammo().reloading);
    let is_brewing = cauldron_query.single().is_ok_and(|s| s.is_brewing());

    if let Ok(mut text) = text_query.single_mut() {
        if is_reloading {
            **text = "Reloading...".to_string();
        } else if is_brewing {
            **text = "Brewing...".to_string();
        }
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

/// Updates the level clock display text with elapsed time.
///
/// Only updates text when the displayed second changes to avoid per-frame allocations.
/// Only updates visibility when the config setting changes.
pub(super) fn update_level_clock(
    kill_stats: Res<KillStats>,
    config: Res<GameConfig>,
    mut clock_query: Query<(&mut Text, &mut Visibility), With<LevelClockDisplay>>,
) {
    for (mut text, mut visibility) in &mut clock_query {
        if config.is_changed() {
            let target = if config.show_level_clock {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            *visibility = target;
        }
        if config.show_level_clock && kill_stats.is_changed() {
            let total_secs = kill_stats.elapsed_time as u32;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            let new_text = format!("{mins}:{secs:02}");
            if text.0 != new_text {
                text.0 = new_text;
            }
        }
    }
}

/// Spawns the boss health bar when a boss appears and no bar exists yet.
pub(super) fn spawn_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<&Health, (With<Boss>, Without<Corpse>)>,
    hag_query: Query<&HagIdentity, (With<Hag>, Without<Corpse>, Without<PermanentlyDead>)>,
    lich_query: Query<&LichPhase, (With<Lich>, Without<Corpse>)>,
    dark_mage_query: Query<
        Entity,
        (
            With<crate::game::units::boss::dark_mage::DarkMage>,
            Without<Corpse>,
        ),
    >,
    ray_query: Query<
        Entity,
        (
            With<crate::game::units::boss::ray::Ray>,
            Without<Corpse>,
        ),
    >,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
) {
    let boss_exists = boss_query.iter().next().is_some();
    let bar_exists = bar_query.iter().next().is_some();
    let is_hags = hag_query.iter().next().is_some();
    let is_lich = lich_query.iter().next().is_some();
    let is_dark_mage = dark_mage_query.iter().next().is_some();
    let is_ray = ray_query.iter().next().is_some();

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
                if is_hags {
                    // "The Hags" title
                    parent.spawn((
                        Text::new("The Hags"),
                        TextFont::from_font_size(BOSS_NAME_FONT_SIZE),
                        TextColor(Color::WHITE),
                    ));

                    // Three-section bar container
                    parent
                        .spawn(Node {
                            width: BOSS_HEALTH_BAR_WIDTH,
                            height: BOSS_HEALTH_BAR_HEIGHT,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(HAG_BAR_SECTION_GAP),
                            ..default()
                        })
                        .with_children(|bar_row| {
                            // Spawn one section per hag
                            for (identity, name, color) in [
                                (HagIdentity::Justina, "Justina", HAG_JUSTINA_BAR_COLOR),
                                (HagIdentity::Martina, "Martina", HAG_MARTINA_BAR_COLOR),
                                (HagIdentity::Josephina, "Josephina", HAG_JOSEPHINA_BAR_COLOR),
                            ] {
                                spawn_hag_bar_section(bar_row, identity, name, color);
                            }
                        });
                } else if is_lich {
                    // Lich: starts as "Soul Power" bar, switches to HP in Phase 2
                    parent.spawn((
                        Text::new("Soul Power"),
                        TextFont::from_font_size(BOSS_HEALTH_TEXT_FONT_SIZE),
                        TextColor(Color::srgba(0.6, 0.9, 0.4, 0.8)),
                        LichBarLabel,
                    ));
                    spawn_simple_boss_bar(
                        parent,
                        "The Lich",
                        crate::game::units::boss::lich::constants::SOUL_POWER_BAR_BORDER_COLOR,
                        crate::game::units::boss::lich::constants::SOUL_POWER_BAR_COLOR,
                        0.0,
                        "0%",
                    );
                } else if is_dark_mage {
                    spawn_simple_boss_bar(
                        parent,
                        "Dark Mage",
                        BOSS_HEALTH_BAR_BORDER_COLOR,
                        BOSS_HEALTH_BAR_FILL_COLOR,
                        100.0,
                        "100%",
                    );
                } else if is_ray {
                    use crate::game::units::boss::ray::RayEyeType;

                    parent.spawn((
                        Text::new("Ray"),
                        TextFont::from_font_size(BOSS_NAME_FONT_SIZE),
                        TextColor(Color::WHITE),
                    ));

                    parent
                        .spawn(Node {
                            width: BOSS_HEALTH_BAR_WIDTH,
                            height: BOSS_HEALTH_BAR_HEIGHT,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(HAG_BAR_SECTION_GAP),
                            ..default()
                        })
                        .with_children(|bar_row| {
                            let sections = [
                                (RayEyeType::Petrification, "Pet", Color::srgb(0.7, 0.7, 0.7)),
                                (RayEyeType::Disintegration, "Dis", Color::srgb(1.0, 0.6, 0.1)),
                                (RayEyeType::Fear, "Fear", Color::srgb(0.6, 0.0, 0.8)),
                                (RayEyeType::MindControl, "MC", Color::srgb(1.0, 0.3, 0.6)),
                                (RayEyeType::Teleportation, "Tele", Color::srgb(0.0, 1.0, 0.7)),
                            ];
                            for (eye_type, name, color) in sections {
                                spawn_ray_eye_bar_section(bar_row, eye_type, name, color);
                            }
                        });
                } else {
                    spawn_simple_boss_bar(
                        parent,
                        "Ogre",
                        BOSS_HEALTH_BAR_BORDER_COLOR,
                        BOSS_HEALTH_BAR_FILL_COLOR,
                        100.0,
                        "100%",
                    );
                }
            });
    }
}

/// Spawns a boss health bar with a title, colored fill, and percentage text.
/// Used by Ogre and Lich (Hags use a separate three-section layout).
fn spawn_simple_boss_bar(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    border_color: Color,
    fill_color: Color,
    initial_percent: f32,
    initial_text: &str,
) {
    parent.spawn((
        Text::new(name.to_string()),
        TextFont::from_font_size(BOSS_NAME_FONT_SIZE),
        TextColor(Color::WHITE),
    ));

    parent
        .spawn((
            Node {
                width: BOSS_HEALTH_BAR_WIDTH,
                height: BOSS_HEALTH_BAR_HEIGHT,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(3.0)),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(initial_percent),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
                BorderRadius::all(Val::Px(2.0)),
                BossHealthBarFill,
            ));

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
                        Text::new(initial_text.to_string()),
                        TextFont::from_font_size(BOSS_HEALTH_TEXT_FONT_SIZE),
                        TextColor(Color::WHITE),
                        BossHealthBarText,
                    ));
                });
        });
}

/// Spawns a single hag health bar section within the three-part bar.
fn spawn_hag_bar_section(
    parent: &mut ChildSpawnerCommands,
    identity: HagIdentity,
    name: &str,
    fill_color: Color,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|section| {
            // Name label
            section.spawn((
                Text::new(name.to_string()),
                TextFont::from_font_size(BOSS_HEALTH_TEXT_FONT_SIZE),
                TextColor(Color::WHITE),
            ));

            // Bar background
            section
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(BOSS_HEALTH_BAR_BORDER_COLOR),
                    BorderRadius::all(Val::Px(2.0)),
                ))
                .with_children(|bar| {
                    // Fill
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(fill_color),
                        BorderRadius::all(Val::Px(1.0)),
                        HagHealthBarFill { identity },
                    ));

                    // Text overlay
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
                                HagHealthBarText { identity },
                            ));
                        });
                });
        });
}

fn spawn_ray_eye_bar_section(
    parent: &mut ChildSpawnerCommands,
    eye_type: crate::game::units::boss::ray::RayEyeType,
    name: &str,
    fill_color: Color,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|section| {
            section.spawn((
                Text::new(name.to_string()),
                TextFont::from_font_size(BOSS_HEALTH_TEXT_FONT_SIZE),
                TextColor(Color::WHITE),
            ));

            section
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(BOSS_HEALTH_BAR_BORDER_COLOR),
                    BorderRadius::all(Val::Px(2.0)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(fill_color),
                        BorderRadius::all(Val::Px(1.0)),
                        RayEyeHealthBarFill { eye_type },
                    ));

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
                                RayEyeHealthBarText { eye_type },
                            ));
                        });
                });
        });
}

pub(super) fn update_ray_eye_health_bar(
    ray_eyes: Query<
        (&crate::game::units::boss::ray::RayEye, &Health),
        Without<crate::game::units::boss::ray::RayEyeDying>,
    >,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor, &RayEyeHealthBarFill)>,
    mut text_query: Query<(&mut Text, &RayEyeHealthBarText)>,
    mut last_pct: Local<[i16; 5]>,
) {
    let mut health_by_eye: [Option<(f32, f32)>; 5] = [None; 5];
    for (eye, health) in ray_eyes.iter() {
        health_by_eye[eye.eye_type.index()] = Some((health.current, health.max));
    }

    let pct_int: [i16; 5] = std::array::from_fn(|i| match health_by_eye[i] {
        Some((current, max)) => ((current / max).clamp(0.0, 1.0) * 100.0) as i16,
        None => -1,
    });

    for (mut node, mut bg, marker) in fill_query.iter_mut() {
        let idx = marker.eye_type.index();
        if pct_int[idx] == last_pct[idx] {
            continue;
        }
        if pct_int[idx] < 0 {
            node.width = Val::Percent(0.0);
        } else {
            node.width = Val::Percent(pct_int[idx] as f32);
            if bg.0.alpha() < 0.5 {
                bg.0 = bg.0.with_alpha(1.0);
            }
        }
    }

    for (mut text, marker) in text_query.iter_mut() {
        let idx = marker.eye_type.index();
        if pct_int[idx] == last_pct[idx] {
            continue;
        }
        text.0 = format!("{}%", pct_int[idx].max(0));
    }

    *last_pct = pct_int;
}

/// Updates the boss health bar fill and text. Despawns the bar when the boss dies.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<&Health, (With<Boss>, Without<Corpse>)>,
    hag_query: Query<
        (&HagIdentity, &Health),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    lich_query: Query<(&Health, &SoulPower, &LichPhase), (With<Lich>, Without<Corpse>)>,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
    mut fill_query: Query<&mut Node, (With<BossHealthBarFill>, Without<HagHealthBarFill>)>,
    mut text_query: Query<
        &mut Text,
        (
            With<BossHealthBarText>,
            Without<HagHealthBarText>,
            Without<LichBarLabel>,
        ),
    >,
    mut hag_fill_query: Query<
        (&mut Node, &mut BackgroundColor, &HagHealthBarFill),
        Without<BossHealthBarFill>,
    >,
    mut hag_text_query: Query<
        (&mut Text, &HagHealthBarText),
        (Without<BossHealthBarText>, Without<LichBarLabel>),
    >,
    mut label_query: Query<
        &mut Text,
        (
            With<LichBarLabel>,
            Without<BossHealthBarText>,
            Without<HagHealthBarText>,
        ),
    >,
    mut fill_bg_query: Query<
        &mut BackgroundColor,
        (With<BossHealthBarFill>, Without<HagHealthBarFill>),
    >,
) {
    // Build a fixed-size lookup of living hag health (no heap allocation)
    let mut living = [None::<f32>; 3]; // indexed: Justina=0, Martina=1, Josephina=2
    let mut any_hag = false;
    for (identity, health) in &hag_query {
        any_hag = true;
        let idx = match identity {
            HagIdentity::Justina => 0,
            HagIdentity::Martina => 1,
            HagIdentity::Josephina => 2,
        };
        living[idx] = Some((health.current / health.max * 100.0).clamp(0.0, 100.0));
    }

    if any_hag {
        // Single pass over UI elements — update or dim based on living lookup
        for (mut node, mut bg, fill) in &mut hag_fill_query {
            let idx = match fill.identity {
                HagIdentity::Justina => 0,
                HagIdentity::Martina => 1,
                HagIdentity::Josephina => 2,
            };
            if let Some(hp_percent) = living[idx] {
                node.width = Val::Percent(hp_percent);
            } else {
                node.width = Val::Percent(0.0);
                bg.0 = HAG_BAR_DEAD_COLOR;
            }
        }
        for (mut text, text_marker) in &mut hag_text_query {
            let idx = match text_marker.identity {
                HagIdentity::Justina => 0,
                HagIdentity::Martina => 1,
                HagIdentity::Josephina => 2,
            };
            if let Some(hp_percent) = living[idx] {
                **text = format!("{:.0}%", hp_percent);
            } else {
                **text = "Dead".to_string();
            }
        }

        // Remove bar when all hags are dead
        if living.iter().all(|h| h.is_none()) {
            for entity in &bar_query {
                commands.entity(entity).try_despawn();
            }
        }
    } else if let Ok((health, soul_power, phase)) = lich_query.single() {
        // Lich: show soul power in Phase 1, HP in Phase 2
        match phase {
            LichPhase::Approaching | LichPhase::Summoning => {
                // Soul power bar (filling from 0 to 100%)
                let percent = soul_power.percent();
                if let Ok(mut node) = fill_query.single_mut() {
                    node.width = Val::Percent(percent);
                }
                if let Ok(mut text) = text_query.single_mut() {
                    **text = format!("{:.0}%", percent);
                }
                if let Ok(mut label) = label_query.single_mut() {
                    **label = "Soul Power".to_string();
                }
            }
            LichPhase::Combat => {
                // Switch to HP display
                let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
                if let Ok(mut node) = fill_query.single_mut() {
                    node.width = Val::Percent(hp_percent);
                }
                if let Ok(mut text) = text_query.single_mut() {
                    **text = format!("{:.0}%", hp_percent);
                }
                // Update label and bar color to HP mode
                if let Ok(mut label) = label_query.single_mut() {
                    **label = "Health".to_string();
                }
                if let Ok(mut bg) = fill_bg_query.single_mut() {
                    bg.0 = BOSS_HEALTH_BAR_FILL_COLOR;
                }
            }
        }
    } else if let Some(health) = boss_query.iter().next() {
        // Original ogre update
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
            commands.entity(entity).try_despawn();
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

/// Updates the wave counter display text from the current WaveState.
pub(super) fn update_wave_display(
    wave_state: Res<WaveState>,
    mut wave_display_query: Query<&mut Text, With<WaveDisplay>>,
) {
    if wave_state.is_changed()
        && let Ok(mut text) = wave_display_query.single_mut()
    {
        **text = format!(
            "Wave: {} / {}",
            wave_state.current_wave + 1,
            wave_state.total_waves
        );
    }
}

/// Spawns a "Wave X incoming!" flash when a new wave spawns.
pub(super) fn spawn_wave_incoming_flash(
    mut commands: Commands,
    mut wave_events: MessageReader<WaveSpawnedMessage>,
    existing_flash: Query<Entity, With<WaveIncomingFlash>>,
) {
    for event in wave_events.read() {
        spawn_flash_banner(
            &mut commands,
            &existing_flash,
            &format!("Wave {} incoming!", event.wave_number),
            WAVE_FLASH_COLOR,
        );
    }
}

/// Spawns a "The King calls for a retreat!" flash when retreat triggers.
pub(super) fn spawn_retreat_flash(
    mut commands: Commands,
    mut retreat_events: MessageReader<crate::game::messages::RetreatMessage>,
    existing_flash: Query<Entity, With<WaveIncomingFlash>>,
) {
    for _event in retreat_events.read() {
        spawn_flash_banner(
            &mut commands,
            &existing_flash,
            "The King calls for a retreat!",
            RETREAT_FLASH_COLOR,
        );
    }
}

/// Spawns a centered flash banner at the top of the screen.
///
/// Removes any existing flash before spawning the new one.
fn spawn_flash_banner(
    commands: &mut Commands,
    existing_flash: &Query<Entity, With<WaveIncomingFlash>>,
    text: &str,
    color: Color,
) {
    for entity in existing_flash {
        commands.entity(entity).try_despawn();
    }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(15.0),
            left: Val::Percent(5.0),
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(text),
        TextFont::from_font_size(WAVE_FLASH_FONT_SIZE),
        TextColor(color),
        WaveIncomingFlash {
            timer: WAVE_FLASH_DURATION,
        },
        Pickable::IGNORE,
        GlobalZIndex(998),
        crate::game::components::OnGameplayScreen,
    ));
}

/// Fades and despawns the wave incoming flash.
pub(super) fn update_wave_incoming_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut WaveIncomingFlash, &mut TextColor)>,
) {
    for (entity, mut flash, mut text_color) in &mut flash_query {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            // Fade out over the last second
            let opacity = (flash.timer / 1.0).min(1.0);
            let mut c = text_color.0.to_srgba();
            c.alpha = opacity;
            text_color.0 = c.into();
        }
    }
}

// ===== Buff Tracker Systems =====

/// Returns the primary abbreviation for a buff (from its first effect).
fn buff_abbreviation(effects: &[BrewEffect]) -> &'static str {
    effects.first().map(|e| e.abbreviation()).unwrap_or("??")
}

/// Returns the background color for a buff box based on the brew's averaged color.
fn buff_box_color(effects: &[BrewEffect]) -> Color {
    // Use a muted version of the first effect's type color
    match effects.first() {
        Some(BrewEffect::ManaRegenMultiplier(_) | BrewEffect::MaxManaMultiplier(_)) => {
            Color::srgba(0.2, 0.3, 0.6, 0.7)
        }
        Some(
            BrewEffect::SpellPowerMultiplier(_)
            | BrewEffect::SpellRangeMultiplier(_)
            | BrewEffect::CastSpeedMultiplier(_),
        ) => Color::srgba(0.4, 0.2, 0.5, 0.7),
        Some(
            BrewEffect::DefenderDamageBonus(_)
            | BrewEffect::AttackSpeedMultiplier(_)
            | BrewEffect::EffectivenessBonus(_),
        ) => Color::srgba(0.5, 0.3, 0.1, 0.7),
        Some(
            BrewEffect::DefenderHealPerSecond(_)
            | BrewEffect::DamageResistancePercent(_)
            | BrewEffect::DefenderShieldPerSecond(_),
        ) => Color::srgba(0.1, 0.4, 0.2, 0.7),
        Some(
            BrewEffect::DefenderSpeedBonus(_)
            | BrewEffect::AttackerSlowPercent(_)
            | BrewEffect::BuffDurationMultiplier(_),
        ) => Color::srgba(0.3, 0.3, 0.15, 0.7),
        None => Color::srgba(0.2, 0.2, 0.2, 0.7),
    }
}

/// Rebuilds buff tracker boxes when the CauldronBuffs resource changes.
///
/// Despawns all existing boxes and re-creates them from the current active buffs.
pub(super) fn update_buff_tracker(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    container_query: Query<Entity, With<BuffTrackerContainer>>,
    existing_boxes: Query<Entity, With<BuffTrackerBox>>,
    existing_tooltips: Query<Entity, With<BuffTooltip>>,
) {
    if !cauldron_buffs.is_changed() {
        return;
    }

    // Despawn existing buff boxes
    for entity in &existing_boxes {
        commands.entity(entity).try_despawn();
    }
    // Despawn any lingering tooltips
    for entity in &existing_tooltips {
        commands.entity(entity).try_despawn();
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    // Spawn new boxes for each active buff
    for (i, buff) in cauldron_buffs.active_buffs.iter().enumerate() {
        let abbr = buff_abbreviation(&buff.effects);
        let bg_color = buff_box_color(&buff.effects);

        commands.entity(container).with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(BUFF_BOX_SIZE),
                        height: Val::Px(BUFF_BOX_SIZE),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(BUFF_BOX_BORDER_WIDTH)),
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    BackgroundColor(bg_color),
                    BorderColor::all(BUFF_BOX_BORDER_COLOR),
                    BorderRadius::all(Val::Px(4.0)),
                    BuffTrackerBox(i),
                ))
                .with_children(|box_node| {
                    // Abbreviation label
                    box_node.spawn((
                        Text::new(abbr),
                        TextFont::from_font_size(BUFF_LABEL_FONT_SIZE),
                        TextColor(Color::WHITE),
                    ));
                    // Timer text
                    box_node.spawn((
                        Text::new(format!("{:.0}s", buff.time_remaining)),
                        TextFont::from_font_size(BUFF_TIMER_FONT_SIZE),
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                        BuffTimerText(i),
                    ));
                });
        });
    }
}

/// Updates buff timer text every frame.
pub(super) fn update_buff_timers(
    cauldron_buffs: Res<CauldronBuffs>,
    mut timer_query: Query<(&BuffTimerText, &mut Text)>,
) {
    for (timer, mut text) in &mut timer_query {
        if let Some(buff) = cauldron_buffs.active_buffs.get(timer.0) {
            **text = format!("{:.0}s", buff.time_remaining.ceil());
        }
    }
}

/// Shows a tooltip when hovering over a buff tracker box.
pub(super) fn show_buff_tooltip(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    box_query: Query<(&Interaction, &BuffTrackerBox), Changed<Interaction>>,
    existing_tooltips: Query<Entity, With<BuffTooltip>>,
) {
    for (interaction, buff_box) in &box_query {
        match interaction {
            Interaction::Hovered => {
                // Despawn any existing tooltip
                for entity in &existing_tooltips {
                    commands.entity(entity).try_despawn();
                }

                let Some(buff) = cauldron_buffs.active_buffs.get(buff_box.0) else {
                    continue;
                };

                // Build tooltip text
                let tooltip_lines: Vec<String> =
                    buff.effects.iter().map(|e| e.display_text()).collect();
                let tooltip_text = tooltip_lines.join("\n");

                // Spawn tooltip as absolute-positioned node
                commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(BUFF_BOX_SIZE + BUFF_BOX_GAP + 20.0 + 10.0),
                            left: Val::Px(20.0),
                            max_width: Val::Px(BUFF_TOOLTIP_MAX_WIDTH),
                            padding: UiRect::all(Val::Px(BUFF_TOOLTIP_PADDING)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(BUFF_TOOLTIP_BG),
                        BorderColor::all(BUFF_TOOLTIP_BORDER),
                        BorderRadius::all(Val::Px(4.0)),
                        GlobalZIndex(999),
                        BuffTooltip,
                        Pickable::IGNORE,
                        OnGameplayScreen,
                    ))
                    .with_children(|tooltip| {
                        tooltip.spawn((
                            Text::new(tooltip_text),
                            TextFont::from_font_size(BUFF_TOOLTIP_FONT_SIZE),
                            TextColor(Color::WHITE),
                        ));
                    });
            }
            Interaction::None => {
                // Despawn tooltip when hover ends
                for entity in &existing_tooltips {
                    commands.entity(entity).try_despawn();
                }
            }
            Interaction::Pressed => {}
        }
    }
}
