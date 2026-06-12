use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::hags::components::{Hag, HagIdentity, PermanentlyDead};
use crate::game::units::boss::lich::Lich;
use crate::game::units::boss::lich::components::LichPhase;
use crate::game::units::components::{Corpse, Health};

/// Spawns the boss health bar when a boss appears and no bar exists yet.
pub(crate) fn spawn_boss_health_bar(
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
    ray_query: Query<Entity, (With<crate::game::units::boss::ray::Ray>, Without<Corpse>)>,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
) {
    let boss_exists = !boss_query.is_empty();
    let bar_exists = !bar_query.is_empty();
    let is_hags = !hag_query.is_empty();
    let is_lich = !lich_query.is_empty();
    let is_dark_mage = !dark_mage_query.is_empty();
    let is_ray = !ray_query.is_empty();

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
                                (
                                    RayEyeType::Petrification,
                                    "Pet",
                                    RAY_EYE_PETRIFICATION_COLOR,
                                ),
                                (
                                    RayEyeType::Disintegration,
                                    "Dis",
                                    RAY_EYE_DISINTEGRATION_COLOR,
                                ),
                                (RayEyeType::Fear, "Fear", RAY_EYE_FEAR_COLOR),
                                (RayEyeType::MindControl, "MC", RAY_EYE_MIND_CONTROL_COLOR),
                                (
                                    RayEyeType::Teleportation,
                                    "Tele",
                                    RAY_EYE_TELEPORTATION_COLOR,
                                ),
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
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
            BorderColor::all(border_color),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(initial_percent),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(fill_color),
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
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(BOSS_HEALTH_BAR_BORDER_COLOR),
                ))
                .with_children(|bar| {
                    // Fill
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(fill_color),
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
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BOSS_HEALTH_BAR_BG_COLOR),
                    BorderColor::all(BOSS_HEALTH_BAR_BORDER_COLOR),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(fill_color),
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
