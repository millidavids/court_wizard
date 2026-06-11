use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::config::{GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::resources::{CurrentLevel, WaveState};
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::ui::systems::spawn_button;

/// Spawns the king health bar as a vertical bar.
///
/// Used by both SP and MP HUDs.
pub(super) fn spawn_king_health_bar(parent: &mut ChildSpawnerCommands) {
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
pub(crate) fn spawn_hud(
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
