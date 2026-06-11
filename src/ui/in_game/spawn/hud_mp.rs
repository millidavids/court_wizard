use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::hud_sp::spawn_king_health_bar;
use crate::config::{GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::ui::systems::spawn_button;

/// Spawns a simplified HUD for multiplayer games.
///
/// Like `spawn_hud` but without the Cauldron button, level display, and past victory display
/// (those are single-player only concepts).
pub(crate) fn spawn_mp_hud(mut commands: Commands, config: Res<GameConfig>) {
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
                        // Buff tracker container (buff boxes spawned dynamically) —
                        // mirrors the single-player HUD so brewed buffs are visible
                        // in multiplayer too.
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
