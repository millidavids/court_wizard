//! In-game bar updates: mana, ammo, cast, boss health, king health.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::resources::{CurrentLevel, KillStats};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::hags::components::{Hag, HagIdentity, PermanentlyDead};
use crate::game::units::boss::lich::Lich;
use crate::game::units::boss::lich::components::{LichPhase, SoulPower};
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::archetypes::gunslinger::GunState;
use crate::game::units::wizard::components::{CastingState, LocalWizard, Mana, PrimedSpell};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::networking::session::MultiplayerSession;

/// Blocks spell input when the mouse is interacting with a UI button so
/// clicking a HUD button doesn't simultaneously fire a spell. With a
/// gamepad, spells are triggered by RT (not by a UI cursor), and the hidden
/// OS cursor may sit over a HUD button at any time — gating on mouse mode
/// prevents those incidental hovers from silently killing RT casts.
pub(super) fn update_mana_bar(
    wizard_query: Query<&Mana, With<LocalWizard>>,
    concentration_spells: Query<&ConcentrationSpell>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
    mut reserved_bar_query: Query<&mut Node, (With<ManaBarReservedFill>, Without<ManaBarFill>)>,
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
    ray_query: Query<Entity, (With<crate::game::units::boss::ray::Ray>, Without<Corpse>)>,
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
                                (
                                    RayEyeType::Disintegration,
                                    "Dis",
                                    Color::srgb(1.0, 0.6, 0.1),
                                ),
                                (RayEyeType::Fear, "Fear", Color::srgb(0.6, 0.0, 0.8)),
                                (RayEyeType::MindControl, "MC", Color::srgb(1.0, 0.3, 0.6)),
                                (
                                    RayEyeType::Teleportation,
                                    "Tele",
                                    Color::srgb(0.0, 1.0, 0.7),
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

/// Updates the king health bar fill based on the local player's team's king health.
pub(super) fn update_king_health_bar(
    king_query: Query<(&Health, &Team), (With<King>, Without<Corpse>)>,
    mut fill_query: Query<&mut Node, With<KingHealthBarFill>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };

    // The team this player commands: Defenders in single-player and as the
    // multiplayer host, Attackers as the guest. The multiplayer wizard has no
    // `Team` component (only the single-player wizard does), so the old
    // `Query<&Team, With<LocalWizard>>` never matched in MP and the bar stayed
    // empty. Derive the team from the peer role instead.
    let local_team = local_player_team(session.as_deref());

    // Find the king matching the local player's team
    for (health, team) in &king_query {
        if *team == local_team {
            let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
            fill_node.height = Val::Percent(hp_percent);
            return;
        }
    }

    // No matching king found (dead) — show empty bar
    fill_node.height = Val::Percent(0.0);
}
