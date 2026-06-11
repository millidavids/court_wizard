use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::game::game_mode::components::{
    ActiveToggles, GameMode, ROGUELITE_MAX_LEVEL, RogueliteModifiers, ToggleModifier, format_time,
};
use crate::game::resources::{CurrentLevel, KillStats, WaveState};
use crate::ui::systems::{
    default_content_node, spawn_button, spawn_page_container, spawn_title_with_shadow,
};

use super::super::components::{OnPauseMainScreen, PauseMenuButtonAction, ScrollablePauseStats};
use super::super::constants::*;

/// Pre-collected data for the left panel (avoids borrow conflicts with Commands).
pub(crate) struct LeftPanelData {
    pub(crate) stats: Vec<(&'static str, String)>,
    pub(crate) section: Option<LeftPanelSection>,
}

pub(crate) enum LeftPanelSection {
    Modifiers {
        sliders: Vec<(&'static str, String)>,
        toggles: Vec<&'static str>,
    },
    EndlessBest {
        stats: Vec<(&'static str, String)>,
    },
}

/// Sets up the pause menu main screen UI with a two-panel layout.
#[allow(clippy::too_many_arguments)]
pub fn setup(
    mut commands: Commands,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    wave_state: Option<Res<WaveState>>,
    game_mode: Option<Res<GameMode>>,
    roguelite_modifiers: Option<Res<RogueliteModifiers>>,
    active_toggles: Option<Res<ActiveToggles>>,
    active_save: Res<ActiveSave>,
    initial_defenders: Option<Res<crate::game::resources::InitialDefenderCount>>,
) {
    let content = spawn_page_container(
        &mut commands,
        OnPauseMainScreen,
        true,
        default_content_node(),
    );
    // Trap gamepad focus to the pause menu so controller nav doesn't leak
    // through to the in-game HUD buttons (Spells / Cauldron / etc.) sitting
    // behind the overlay.
    commands
        .entity(content)
        .insert(crate::ui::focus::ModalOverlay);

    // Collect left panel data before building UI (avoids borrow conflicts)
    let left_panel_data = collect_left_panel_data(
        &kill_stats,
        &current_level,
        wave_state.as_deref(),
        game_mode.as_deref(),
        roguelite_modifiers.as_deref(),
        active_toggles.as_deref(),
        &active_save,
        initial_defenders.as_deref(),
    );

    commands.entity(content).with_children(|root| {
        // Title
        spawn_title_with_shadow(
            root,
            "Paused",
            TITLE_FONT_SIZE,
            TEXT_COLOR,
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        );

        // Two-panel row
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(crate::ui::constants::TWO_PANEL_GAP),
            ..default()
        })
        .with_children(|row| {
            spawn_left_panel(row, &left_panel_data);
            spawn_right_panel(row, game_seed.as_deref());
        });
    });
}

/// Collects all data needed for the left panel from game resources.
#[allow(clippy::too_many_arguments)]
fn collect_left_panel_data(
    kill_stats: &KillStats,
    current_level: &CurrentLevel,
    wave_state: Option<&WaveState>,
    game_mode: Option<&GameMode>,
    roguelite_modifiers: Option<&RogueliteModifiers>,
    active_toggles: Option<&ActiveToggles>,
    active_save: &ActiveSave,
    initial_defenders: Option<&crate::game::resources::InitialDefenderCount>,
) -> LeftPanelData {
    let is_roguelite = game_mode.is_some_and(|m| m.is_roguelite());
    let is_endless = game_mode.is_some_and(|m| m.is_endless());
    let level = current_level.0;

    let mut stats = Vec::new();

    // Level
    if is_roguelite {
        stats.push(("Level", format!("{} / {}", level, ROGUELITE_MAX_LEVEL)));
    } else {
        stats.push(("Level", format!("{}", level)));
    }

    // Time
    stats.push(("Time", format_time(kill_stats.elapsed_time)));

    // Kills
    let total_kills = kill_stats.attackers_killed + kill_stats.undead_killed;
    stats.push(("Kills", format!("{}", total_kills)));

    // Efficiency (use actual initial count which accounts for Veteran/Attrition toggles)
    let total_defenders = initial_defenders.map_or(
        (crate::game::constants::INITIAL_DEFENDER_COUNT
            + crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT) as f32,
        |d| d.0 as f32,
    );
    let efficiency = if total_defenders > 0.0 {
        (1.0 - kill_stats.defenders_killed as f32 / total_defenders) * 100.0
    } else {
        100.0
    };
    stats.push(("Efficiency", format!("{:.0}%", efficiency)));

    // Wave progress
    if let Some(ws) = wave_state {
        let display_wave = ws.current_wave + 1;
        stats.push(("Wave", format!("{} / {}", display_wave, ws.total_waves)));
    }

    // Mode-specific section
    let section = if is_roguelite {
        let mut sliders = Vec::new();
        if let Some(mods) = roguelite_modifiers {
            for (label, pct) in mods.non_default_entries() {
                sliders.push((label, format!("{}%", pct)));
            }
        }
        let mut toggles = Vec::new();
        if let Some(t) = active_toggles {
            for toggle in ToggleModifier::all() {
                if t.is_active(*toggle) {
                    toggles.push(toggle.display_name());
                }
            }
        }
        if !sliders.is_empty() || !toggles.is_empty() {
            Some(LeftPanelSection::Modifiers { sliders, toggles })
        } else {
            None
        }
    } else if is_endless {
        let best = active_save
            .0
            .as_ref()
            .and_then(|_| crate::config::save_data::get_endless_best_stats(level));
        best.map(|b| {
            let mut best_stats = Vec::new();
            best_stats.push((
                "Best Efficiency",
                format!("{:.0}%", b.best_efficiency * 100.0),
            ));
            best_stats.push((
                "Best Kills",
                format!("{}", b.attackers_killed + b.undead_killed),
            ));
            best_stats.push(("Best Time", format_time(b.elapsed_time)));
            LeftPanelSection::EndlessBest { stats: best_stats }
        })
    } else {
        None
    };

    LeftPanelData { stats, section }
}

/// Spawns the left panel with run statistics and modifier info.
fn spawn_left_panel(parent: &mut ChildSpawnerCommands, data: &LeftPanelData) {
    let detail_box =
        crate::ui::systems::spawn_scrollable_left_detail_panel(parent, ScrollablePauseStats);

    parent.commands().entity(detail_box).with_children(|panel| {
        // Battle stats
        for (label, value) in &data.stats {
            spawn_stat_row(panel, label, value);
        }

        // Mode-specific section
        match &data.section {
            Some(LeftPanelSection::Modifiers { sliders, toggles }) => {
                spawn_section_divider(panel, "Modifiers");
                for (label, value) in sliders {
                    spawn_stat_row(panel, label, value);
                }
                for name in toggles {
                    panel.spawn((
                        Text::new(*name),
                        TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                        TextColor(STAT_VALUE_COLOR),
                    ));
                }
            }
            Some(LeftPanelSection::EndlessBest { stats }) => {
                spawn_section_divider(panel, "Level Best");
                for (label, value) in stats {
                    spawn_stat_row(panel, label, value);
                }
            }
            None => {}
        }
    });
}

/// Spawns a stat label + value row.
fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(STAT_LABEL_FONT_SIZE),
                TextColor(STAT_LABEL_COLOR),
            ));
            row.spawn((
                Text::new(value),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(STAT_VALUE_COLOR),
            ));
        });
}

/// Spawns a section divider line with a centered label.
fn spawn_section_divider(parent: &mut ChildSpawnerCommands, label: &str) {
    parent.spawn((
        Text::new(format!("-- {} --", label)),
        TextFont::from_font_size(SECTION_DIVIDER_FONT_SIZE),
        TextColor(SECTION_DIVIDER_COLOR),
        Node {
            margin: UiRect::vertical(Val::Px(8.0)),
            align_self: AlignSelf::Center,
            ..default()
        },
    ));
}

/// Spawns the right panel with pause menu buttons.
fn spawn_right_panel(
    parent: &mut ChildSpawnerCommands,
    game_seed: Option<&crate::game::seeded_rng::resources::GameSeed>,
) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(MARGIN),
            ..default()
        })
        .with_children(|right| {
            spawn_button(
                right,
                "Continue",
                PauseMenuButtonAction::Continue,
                &BUTTON_STYLE,
            );
            spawn_button(
                right,
                "Settings",
                PauseMenuButtonAction::Settings,
                &BUTTON_STYLE,
            );
            spawn_button(
                right,
                "Manual",
                PauseMenuButtonAction::Manual,
                &BUTTON_STYLE,
            );
            spawn_button(
                right,
                "Compendium",
                PauseMenuButtonAction::Compendium,
                &BUTTON_STYLE,
            );
            spawn_button(
                right,
                "Exit to Menu",
                PauseMenuButtonAction::Exit,
                &BUTTON_STYLE,
            );

            // Seed display
            if let Some(seed) = game_seed {
                right.spawn((
                    Text::new(format!("Seed: {}", seed.0)),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                    Node {
                        margin: UiRect::top(Val::Px(MARGIN)),
                        ..default()
                    },
                ));
            }
        });
}
