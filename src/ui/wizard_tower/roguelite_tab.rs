use std::collections::HashSet;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use rand::Rng;

use crate::config::save_data::{self, RogueliteRun};
use crate::config::{ActiveSave, ConfigChanged, GameConfig};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{
    ActiveToggles, GameMode, RogueliteModifiers, RogueliteRunState, RunAggregateStats,
    ToggleModifier, format_time,
};
use crate::game::input::messages::MouseClicked;
use crate::state::AppState;
use crate::ui::components::{ButtonActive, ButtonColors, ButtonStyle};
use crate::ui::constants::{SLIDER_TRACK_WIDTH, TEXT_DISABLED, TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::{SliderRowConfig, spawn_button, spawn_slider_row};

use super::constants::*;

// ===========================================================================
// Constants
// ===========================================================================

/// Maximum seed value (10 digits, fits in the text box).
const MAX_SEED: u64 = 10_000_000_000;
/// Maximum number of characters in the seed input field.
const MAX_SEED_CHARS: usize = 10;

/// Text color for modifier labels.
const LABEL_COLOR: Color = TEXT_PRIMARY;

/// Subtitle / description color.
const DESCRIPTION_COLOR: Color = TEXT_MUTED;

/// Spacing between sections.
const SECTION_MARGIN: f32 = 20.0;

/// Smaller spacing inside rows.
const ROW_GAP: f32 = 10.0;

/// Section header font size.
const SECTION_HEADER_FONT_SIZE: f32 = 13.0;

/// Summary title font size in the left panel.
const SUMMARY_TITLE_FONT_SIZE: f32 = 16.0;

/// Font size for modifier entries in the summary.
const SUMMARY_ITEM_FONT_SIZE: f32 = 11.0;

/// Color for modifier entries in the summary.
const SUMMARY_ITEM_COLOR: Color = Color::hsla(270.0, 0.15, 0.70, 1.0);

/// Placeholder text color when no modifiers are active.
const SUMMARY_PLACEHOLDER_COLOR: Color = TEXT_MUTED;

// Toggle row colors
const TOGGLE_LOCKED_BG: Color = Color::hsla(0.0, 0.0, 0.08, 0.8);
const TOGGLE_LOCKED_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 0.6);
const TOGGLE_OFF_BG: Color = Color::hsla(270.0, 0.10, 0.10, 0.6);
const TOGGLE_OFF_BORDER: Color = Color::hsla(270.0, 0.10, 0.20, 0.5);
const TOGGLE_ON_BG: Color = Color::hsla(270.0, 0.25, 0.16, 0.8);
const TOGGLE_ON_BORDER: Color = Color::hsla(270.0, 0.40, 0.40, 0.8);
const TOGGLE_NAME_FONT_SIZE: f32 = 12.0;
const TOGGLE_DESC_FONT_SIZE: f32 = 10.0;
const TOGGLE_SMALL_BUTTON_FONT_SIZE: f32 = 10.0;

// Confirmation popup
const POPUP_OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
const POPUP_BOX_BG: Color = Color::hsla(20.0, 0.10, 0.10, 0.95);
const POPUP_BOX_BORDER: Color = Color::hsla(270.0, 0.55, 0.50, 1.0);
const POPUP_FONT_SIZE: f32 = 14.0;

/// Start Run button style.
const START_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 45.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(270.0, 0.20, 0.18, 0.75),
    border: Color::hsla(270.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(270.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Change Wizard Type button style — reuse shared muted style.
const CHANGE_WIZARD_BUTTON_STYLE: ButtonStyle = SWITCH_WIZARD_BUTTON_STYLE;

/// Continue Run button style.
const CONTINUE_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 45.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(120.0, 0.20, 0.18, 0.75),
    border: Color::hsla(120.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(120.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// End Run button style.
const END_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 10.0,
    background: Color::hsla(0.0, 0.20, 0.14, 0.6),
    border: Color::hsla(0.0, 0.40, 0.35, 0.6),
    text_color: Color::hsla(0.0, 0.15, 0.70, 1.0),
    text_shadow: true,
};

/// Confirm button style for popups.
const CONFIRM_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 2.0,
    font_size: 12.0,
    background: Color::hsla(25.0, 0.18, 0.14, 0.85),
    border: Color::hsla(25.0, 0.30, 0.30, 1.0),
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Cancel button style for popups.
const CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 12.0,
    background: Color::hsla(25.0, 0.15, 0.07, 0.6),
    border: Color::hsla(270.0, 0.20, 0.25, 0.6),
    text_color: TEXT_MUTED,
    text_shadow: false,
};

/// Active run stats font sizes.
const RUN_STATS_LABEL_FONT: f32 = 12.0;
const RUN_STATS_VALUE_FONT: f32 = 11.0;
const RUN_STATS_VALUE_COLOR: Color = Color::hsla(270.0, 0.15, 0.70, 1.0);

// ===========================================================================
// Components
// ===========================================================================

/// Actions for roguelite tab buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RogueliteAction {
    StartRun,
    EndRun,
    ContinueRun,
    ChangeWizardType,
}

/// Identifies which modifier a slider controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub(super) enum ModifierSliderValue {
    GameSpeed,
    EnemyEffectiveness,
    EnemyCount,
    TerrainDensity,
}

impl ModifierSliderValue {
    pub fn get(&self, modifiers: &RogueliteModifiers) -> f32 {
        match self {
            Self::GameSpeed => modifiers.game_speed,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness,
            Self::EnemyCount => modifiers.enemy_count,
            Self::TerrainDensity => modifiers.terrain_density,
        }
    }

    pub fn set(&self, modifiers: &mut RogueliteModifiers, value: f32) {
        match self {
            Self::GameSpeed => modifiers.game_speed = value,
            Self::EnemyEffectiveness => modifiers.enemy_effectiveness = value,
            Self::EnemyCount => modifiers.enemy_count = value,
            Self::TerrainDensity => modifiers.terrain_density = value,
        }
    }

    pub fn min_value(&self) -> f32 {
        0.2
    }

    pub fn max_value(&self) -> f32 {
        3.0
    }

    pub fn step(&self) -> f32 {
        0.1
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::GameSpeed => "Wave Speed",
            Self::EnemyEffectiveness => "Enemy Strength",
            Self::EnemyCount => "Enemy Count",
            Self::TerrainDensity => "Terrain",
        }
    }
}

/// Component for modifier slider value display text.
#[derive(Component)]
pub(super) struct ModifierSliderText {
    pub value: ModifierSliderValue,
}

/// Button to decrease a modifier slider value.
#[derive(Component)]
pub(super) struct ModifierSliderDownButton {
    pub value: ModifierSliderValue,
}

/// Button to increase a modifier slider value.
#[derive(Component)]
pub(super) struct ModifierSliderUpButton {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider track.
#[derive(Component)]
pub(super) struct ModifierSliderTrack {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider fill.
#[derive(Component)]
pub(super) struct ModifierSliderFill {
    pub value: ModifierSliderValue,
}

/// Component for modifier slider handle.
#[derive(Component)]
pub(super) struct ModifierSliderHandle {
    pub value: ModifierSliderValue,
    pub is_dragging: bool,
}

/// Marker for the seed input text display.
#[derive(Component)]
pub(super) struct SeedInputText;

/// Marker for the seed input box background (clickable to focus).
#[derive(Component)]
pub(crate) struct SeedInputBox;

/// Resource tracking the seed input state.
#[derive(Resource, Default)]
pub(super) struct SeedInputState {
    pub text: String,
    pub focused: bool,
}

/// Marker for the "Random" toggle button.
#[derive(Component)]
pub(super) struct SeedRandomButton;

// ── Toggle Modifier Components ──────────────────────────────────────────────

/// Expand/collapse arrow button for a toggle row.
#[derive(Component)]
pub(super) struct ToggleExpandButton(pub ToggleModifier);

/// Insight cost text for a locked toggle (despawned on unlock).
#[derive(Component)]
pub(super) struct ToggleUnlockButton(pub ToggleModifier);

/// The expandable description text (hidden by default).
#[derive(Component)]
pub(super) struct ToggleDescriptionNode(pub ToggleModifier);

/// The row container for a toggle (used for visual updates).
#[derive(Component)]
pub(super) struct ToggleRowContainer(pub ToggleModifier);

/// Resource tracking which toggle descriptions are expanded.
#[derive(Resource, Default)]
pub(super) struct ExpandedToggles(pub HashSet<ToggleModifier>);

/// Resource tracking which toggles are enabled for the pending run.
#[derive(Resource, Default, Clone)]
pub(super) struct PendingToggles {
    pub enabled: Vec<ToggleModifier>,
}

impl PendingToggles {
    pub fn is_enabled(&self, toggle: ToggleModifier) -> bool {
        self.enabled.contains(&toggle)
    }

    pub fn toggle(&mut self, toggle: ToggleModifier) {
        if let Some(pos) = self.enabled.iter().position(|t| *t == toggle) {
            self.enabled.remove(pos);
        } else {
            self.enabled.push(toggle);
        }
    }
}

/// Marker for the run summary text container in the left panel.
#[derive(Component)]
pub(super) struct RunSummaryContent;

/// Marker for the confirmation popup overlay.
#[derive(Component)]
pub(super) struct ConfirmUnlockPopup;

/// Actions for confirmation popup buttons.
#[derive(Component, Clone, Copy)]
pub(super) enum ConfirmUnlockAction {
    Confirm(ToggleModifier),
    Cancel,
}

/// Marker for the scrollable content area on the right panel.
#[derive(Component)]
pub(super) struct RogueliteScrollableContent;

/// Marker for the scrollable left panel.
#[derive(Component)]
pub(crate) struct RogueliteScrollableLeft;

// ===========================================================================
// Helpers
// ===========================================================================

fn random_seed() -> u64 {
    rand::rng().random_range(0..MAX_SEED)
}

// ===========================================================================
// Panel builders — no active run
// ===========================================================================

/// Builds the right panel content for the "no active run" state.
/// Shows modifier sliders, toggle section, seed input, and action buttons.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_roguelite_no_run_right_panel(
    commands: &mut Commands,
    right_panel_entity: Entity,
    modifiers: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
    seed_text: &str,
    // Co-op gating: Some(false) disables the start button ("Guest Not Ready").
    guest_pending: Option<bool>,
) {
    commands.entity(right_panel_entity).with_children(|right| {
        // Padding wrapper (parent panel handles scrolling)
        right
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(SECTION_PADDING)),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|right| {
                // Action buttons at the top (stacked vertically)
                right
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        align_items: AlignItems::FlexStart,
                        margin: UiRect::bottom(Val::Px(SECTION_MARGIN)),
                        ..default()
                    })
                    .with_children(|buttons| {
                        spawn_coop_gated_button(
                            buttons,
                            guest_pending,
                            "Start Run",
                            RogueliteAction::StartRun,
                            &START_RUN_BUTTON_STYLE,
                        );
                        spawn_button(
                            buttons,
                            "Switch Wizard",
                            RogueliteAction::ChangeWizardType,
                            &CHANGE_WIZARD_BUTTON_STYLE,
                        );
                    });

                // Seed input row
                spawn_seed_input_row(right, seed_text);

                // Sliders
                let sliders = [
                    ModifierSliderValue::GameSpeed,
                    ModifierSliderValue::EnemyEffectiveness,
                    ModifierSliderValue::EnemyCount,
                    ModifierSliderValue::TerrainDensity,
                ];
                for slider_value in sliders {
                    spawn_modifier_slider(right, slider_value, modifiers);
                }

                // Toggle Modifiers section header
                right.spawn((
                    Text::new("Toggle Modifiers"),
                    TextFont::from_font_size(SECTION_HEADER_FONT_SIZE),
                    TextColor(LABEL_COLOR),
                    Node {
                        margin: UiRect::new(
                            Val::ZERO,
                            Val::ZERO,
                            Val::Px(SECTION_MARGIN),
                            Val::Px(ROW_GAP),
                        ),
                        ..default()
                    },
                ));

                // Toggle modifier rows
                let unlocked_ids = save_data::get_unlocked_toggles();
                for &toggle in ToggleModifier::all() {
                    let is_unlocked = unlocked_ids.iter().any(|id| id == toggle.id());
                    let is_enabled = is_unlocked && pending_toggles.is_enabled(toggle);
                    spawn_toggle_row(right, toggle, is_unlocked, is_enabled);
                }
            });
    });
}

/// Builds the left panel content for the "no active run" state.
/// Shows the selected wizard name, difficulty cost summary, and toggle bonuses.
pub(super) fn build_roguelite_no_run_left_panel(
    commands: &mut Commands,
    left_panel_entity: Entity,
    config: &GameConfig,
    modifiers: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
) {
    commands.entity(left_panel_entity).with_children(|panel| {
        // Wizard name
        panel.spawn((
            Text::new(config.wizard_type.display_name()),
            TextFont::from_font_size(SUMMARY_TITLE_FONT_SIZE),
            TextColor(LABEL_COLOR),
        ));

        // Summary content container (rebuilt dynamically)
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
                RunSummaryContent,
            ))
            .with_children(|summary| {
                spawn_summary_items(summary, modifiers, pending_toggles);
            });
    });
}

// ===========================================================================
// Panel builders — active run
// ===========================================================================

/// Builds the right panel content for the "active run" state.
/// Shows Continue and End Run buttons.
pub(super) fn build_roguelite_active_run_right_panel(
    commands: &mut Commands,
    right_panel_entity: Entity,
    // Co-op gating: Some(false) disables Continue Run ("Guest Not Ready").
    guest_pending: Option<bool>,
) {
    commands
        .entity(right_panel_entity)
        .with_children(|wrapper| {
            wrapper
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(SECTION_PADDING)),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|right| {
                    // Centered button container
                    right
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            flex_grow: 1.0,
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_coop_gated_button(
                                buttons,
                                guest_pending,
                                "Continue Run",
                                RogueliteAction::ContinueRun,
                                &CONTINUE_RUN_BUTTON_STYLE,
                            );
                            spawn_button(
                                buttons,
                                "Abandon Run",
                                RogueliteAction::EndRun,
                                &END_RUN_BUTTON_STYLE,
                            );
                        });
                });
        });
}

/// Builds the left panel content for the "active run" state.
/// Shows current wizard, level progress, per-level stats, and aggregates.
pub(super) fn build_roguelite_active_run_left_panel(
    commands: &mut Commands,
    left_panel_entity: Entity,
    config: &GameConfig,
    run_state: &RogueliteRunState,
) {
    commands.entity(left_panel_entity).with_children(|panel| {
        // Wizard name
        panel.spawn((
            Text::new(config.wizard_type.display_name()),
            TextFont::from_font_size(SUMMARY_TITLE_FONT_SIZE),
            TextColor(LABEL_COLOR),
        ));

        // Current level
        let levels_done = run_state.level_stats.len() as u32;
        let next_level = levels_done + 1;
        panel.spawn((
            Text::new(format!("Level {} (next: {})", levels_done, next_level)),
            TextFont::from_font_size(RUN_STATS_LABEL_FONT),
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));

        // Level-by-level stats
        if !run_state.level_stats.is_empty() {
            panel.spawn((
                Text::new("Level Stats"),
                TextFont::from_font_size(RUN_STATS_LABEL_FONT),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));

            for stat in &run_state.level_stats {
                panel.spawn((
                    Text::new(format!(
                        "  Lv{}: {}% eff, {} kills, {}",
                        stat.level,
                        (stat.efficiency * 100.0) as u32,
                        stat.total_kills(),
                        format_time(stat.elapsed_time),
                    )),
                    TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                    TextColor(RUN_STATS_VALUE_COLOR),
                ));
            }

            // Aggregate stats
            let agg = RunAggregateStats::from_level_stats(&run_state.level_stats);
            panel.spawn((
                Text::new("Totals"),
                TextFont::from_font_size(RUN_STATS_LABEL_FONT),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
            panel.spawn((
                Text::new(format!("  Total Kills: {}", agg.total_kills)),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
            panel.spawn((
                Text::new(format!(
                    "  Avg Efficiency: {}%",
                    (agg.avg_efficiency * 100.0) as u32
                )),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
            panel.spawn((
                Text::new(format!("  Total Time: {}", format_time(agg.total_time))),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
        }
    });
}

// ===========================================================================
// Resource initialization
// ===========================================================================

/// Initializes roguelite tab resources when the tab is first shown.
/// Call this when entering the roguelite tab (no active run).
pub(super) fn init_roguelite_tab_resources(
    commands: &mut Commands,
    config: &mut GameConfig,
    existing_modifiers: Option<&RogueliteModifiers>,
    existing_pending: Option<&PendingToggles>,
    existing_active_toggles: Option<&ActiveToggles>,
) {
    let mods = existing_modifiers.cloned().unwrap_or_default();
    if existing_modifiers.is_none() {
        commands.insert_resource(mods.clone());
    }

    // Only preserve pending toggles if ActiveToggles exists (returning from wizard select).
    // Otherwise start fresh to prevent stale selections.
    let pending_toggles = if existing_active_toggles.is_some() {
        existing_pending.cloned().unwrap_or_default()
    } else {
        PendingToggles::default()
    };
    commands.insert_resource(pending_toggles);

    // Always generate a fresh random seed when opening this tab
    let seed = random_seed();
    config.seed = Some(seed);
    let seed_text = seed.to_string();
    commands.insert_resource(SeedInputState {
        text: seed_text,
        focused: false,
    });

    commands.insert_resource(ExpandedToggles::default());
}

// ===========================================================================
// Spawners (slider, seed, toggle rows)
// ===========================================================================

/// Spawns the seed input row with label, text input box, and Randomize button.
fn spawn_seed_input_row(parent: &mut ChildSpawnerCommands, seed_text: &str) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(crate::ui::constants::SLIDER_GAP),
            margin: UiRect::bottom(Val::Px(SECTION_MARGIN)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new("Seed"),
                TextFont::from_font_size(crate::ui::constants::SLIDER_LABEL_FONT_SIZE),
                TextColor(LABEL_COLOR),
                Node {
                    min_width: Val::Px(200.0),
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Input text field
            let seed_bg = Color::hsla(270.0, 0.08, 0.08, 1.0);
            row.spawn((
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0)),
                BackgroundColor(seed_bg),
                SeedInputBox,
                crate::ui::focus::Focusable,
                crate::ui::focus::FocusableFlatBackground { base: seed_bg },
            ))
            .with_children(|input_box| {
                input_box.spawn((
                    Text::new(seed_text),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    SeedInputText,
                ));
            });

            // Random button
            row.spawn((
                Button,
                Node {
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0)),
                BackgroundColor(Color::hsla(270.0, 0.08, 0.10, 1.0)),
                ButtonColors {
                    background: Color::hsla(270.0, 0.08, 0.10, 1.0),
                    border: Color::hsla(270.0, 0.35, 0.35, 1.0),
                },
                SeedRandomButton,
                crate::ui::focus::Focusable,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Randomize"),
                    TextFont::from_font_size(12.0),
                    TextColor(LABEL_COLOR),
                ));
            });
        });
}

/// Spawns a single modifier slider using the shared slider row helper.
fn spawn_modifier_slider(
    parent: &mut ChildSpawnerCommands,
    slider_value: ModifierSliderValue,
    modifiers: &RogueliteModifiers,
) {
    let current_value = slider_value.get(modifiers);

    spawn_slider_row(
        parent,
        SliderRowConfig {
            label: slider_value.label(),
            current_value,
            min_value: slider_value.min_value(),
            max_value: slider_value.max_value(),
            text_component: ModifierSliderText {
                value: slider_value,
            },
            down_button: ModifierSliderDownButton {
                value: slider_value,
            },
            up_button: ModifierSliderUpButton {
                value: slider_value,
            },
            slider_track: ModifierSliderTrack {
                value: slider_value,
            },
            slider_fill: ModifierSliderFill {
                value: slider_value,
            },
            slider_handle: ModifierSliderHandle {
                value: slider_value,
                is_dragging: false,
            },
        },
    );
}

/// Spawns a single toggle modifier row.
fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    toggle: ToggleModifier,
    is_unlocked: bool,
    is_enabled: bool,
) {
    let (bg, border) = if !is_unlocked {
        (TOGGLE_LOCKED_BG, TOGGLE_LOCKED_BORDER)
    } else if is_enabled {
        (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
    } else {
        (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(8.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Toggle button (takes remaining space)
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(6.0)),
                    row_gap: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
                ButtonColors {
                    background: bg,
                    border,
                },
                ToggleRowContainer(toggle),
                crate::ui::focus::Focusable,
            ))
            .insert_if(ButtonActive, || is_enabled)
            .with_children(|toggle_btn| {
                // Header: [Name ... (Insight cost if locked)]
                toggle_btn
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|header| {
                        let name_color = if is_unlocked {
                            LABEL_COLOR
                        } else {
                            TEXT_DISABLED
                        };
                        header.spawn((
                            Text::new(toggle.display_name()),
                            TextFont::from_font_size(TOGGLE_NAME_FONT_SIZE),
                            TextColor(name_color),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));

                        if !is_unlocked {
                            header.spawn((
                                Text::new(format!("{} Insight", toggle.insight_cost())),
                                TextFont::from_font_size(TOGGLE_SMALL_BUTTON_FONT_SIZE),
                                TextColor(crate::ui::constants::INSIGHT_COLOR),
                                ToggleUnlockButton(toggle),
                            ));
                        }
                    });

                // Description (always present, revealed when expanded)
                toggle_btn.spawn((
                    Text::new(toggle.description()),
                    TextFont::from_font_size(TOGGLE_DESC_FONT_SIZE),
                    TextColor(DESCRIPTION_COLOR),
                    Node {
                        display: Display::None,
                        max_width: Val::Percent(95.0),
                        padding: UiRect::left(Val::Px(4.0)),
                        ..default()
                    },
                    ToggleDescriptionNode(toggle),
                ));
            });

            // Expand caret button (▾) — matches toggle row height
            let expand_bg = Color::hsla(270.0, 0.10, 0.10, 0.6);
            let expand_border = Color::hsla(270.0, 0.10, 0.20, 0.5);
            row.spawn((
                Button,
                Node {
                    width: Val::Px(36.0),
                    min_height: Val::Px(36.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(expand_border),
                BackgroundColor(expand_bg),
                ButtonColors {
                    background: expand_bg,
                    border: expand_border,
                },
                ToggleExpandButton(toggle),
                crate::ui::focus::Focusable,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("?"),
                    TextFont::from_font_size(14.0),
                    TextColor(TEXT_MUTED),
                ));
            });
        });
}

/// Builds the run-summary lines (non-default sliders + enabled toggles, or a
/// "Default settings" placeholder) as plain strings. Shared by the local panel
/// (`spawn_summary_items`) and the co-op host broadcast (so the guest's mirror
/// shows the same summary the host sees).
pub(crate) fn roguelite_summary_lines(
    mods: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (label, pct) in mods.non_default_entries() {
        lines.push(format!("{}: {}%", label, pct));
    }
    for toggle in &pending_toggles.enabled {
        lines.push(format!(
            "{} (+{}% Insight)",
            toggle.display_name(),
            toggle.insight_bonus_percent()
        ));
    }
    if lines.is_empty() {
        lines.push("Default settings".to_string());
    }
    lines
}

/// Spawns text items inside the run summary content container.
fn spawn_summary_items(
    parent: &mut ChildSpawnerCommands,
    mods: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
) {
    let lines = roguelite_summary_lines(mods, pending_toggles);
    // Colour the lines as the "Default settings" placeholder when nothing is
    // active — derived from the inputs, not a fragile string-compare on output.
    let is_placeholder =
        mods.non_default_entries().is_empty() && pending_toggles.enabled.is_empty();
    let color = if is_placeholder {
        SUMMARY_PLACEHOLDER_COLOR
    } else {
        SUMMARY_ITEM_COLOR
    };
    for line in lines {
        parent.spawn((
            Text::new(line),
            TextFont::from_font_size(SUMMARY_ITEM_FONT_SIZE),
            TextColor(color),
        ));
    }
}

// ===========================================================================
// Confirmation Popup
// ===========================================================================

/// Spawns a confirmation popup for unlocking a toggle modifier.
fn spawn_unlock_popup(commands: &mut Commands, toggle: ToggleModifier) {
    let current_insight = save_data::get_insight();
    let cost = toggle.insight_cost();
    let can_afford = current_insight >= cost;

    let message = if can_afford {
        format!(
            "Unlock \"{}\" for {} Insight?\n(You have {} Insight)",
            toggle.display_name(),
            cost,
            current_insight
        )
    } else {
        format!(
            "Not enough Insight to unlock \"{}\".\nCost: {} | You have: {}",
            toggle.display_name(),
            cost,
            current_insight
        )
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            ConfirmUnlockPopup,
            crate::ui::focus::ModalOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(350.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(POPUP_BOX_BORDER),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(message),
                        TextFont::from_font_size(POPUP_FONT_SIZE),
                        TextColor(LABEL_COLOR),
                        TextLayout::new_with_justify(Justify::Center),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            if can_afford {
                                spawn_button(
                                    buttons,
                                    "Confirm",
                                    ConfirmUnlockAction::Confirm(toggle),
                                    &CONFIRM_BUTTON_STYLE,
                                );
                            }
                            spawn_button(
                                buttons,
                                "Cancel",
                                ConfirmUnlockAction::Cancel,
                                &CANCEL_BUTTON_STYLE,
                            );
                        });
                });
        });
}

// ===========================================================================
// Systems
// ===========================================================================

/// Handles roguelite action button clicks (Start Run, End Run, Continue, Change Wizard).
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_roguelite_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&RogueliteAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut config: ResMut<GameConfig>,
    mut active_save: ResMut<ActiveSave>,
    pending_toggles: Option<Res<PendingToggles>>,
    roguelite_run: Option<Res<RogueliteRunState>>,
    roguelite_modifiers: Option<Res<RogueliteModifiers>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut config_events: MessageWriter<ConfigChanged>,
    // Co-op: when a guest is connected, starting/continuing a run brings them in.
    mut connection: ResMut<crate::networking::resources::NetworkConnection>,
    lobby: Option<Res<super::multiplayer_tab::state::MultiplayerLobby>>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
            RogueliteAction::StartRun => {
                channel_change.write(ChannelChangeMessage);

                // Insert game mode
                commands.insert_resource(GameMode::Roguelite);

                // Create ActiveToggles from pending
                let toggles = pending_toggles
                    .as_ref()
                    .map(|p| ActiveToggles::new(p.enabled.clone()))
                    .unwrap_or_default();
                commands.insert_resource(toggles);

                save_data::load_or_create_wizard(config.wizard_type, &mut config, &mut active_save);

                // Initialize roguelite run state (normally done by init_roguelite_run
                // on OnEnter(MetaGame), but we're already in MetaGame)
                config.current_level = 1;
                config.saved_walls.clear();
                config.saved_crystals.clear();
                config.saved_flora.clear();
                config.saved_trampling = Default::default();
                config.saved_trees.clear();
                config.saved_ponds.clear();
                config.saved_bushes.clear();
                config.saved_boulders.clear();
                config.efficiency_ratios.clear();
                commands.insert_resource(RogueliteRunState {
                    started_at: save_data::current_timestamp(),
                    level_stats: vec![],
                });

                config_events.write(ConfigChanged);

                // Co-op: bring the connected guest into this roguelite run.
                if let Some(gw) = super::multiplayer_tab::state::connected_coop_guest_wizard(
                    &connection,
                    lobby.as_deref(),
                ) {
                    crate::game::multiplayer::coop::start_coop_host(
                        &mut commands,
                        &mut connection,
                        &mut config,
                        gw,
                        crate::networking::session::SessionMode::CoopRoguelite,
                    );
                }

                // Transition to loading
                next_app_state.set(AppState::Loading);
            }
            RogueliteAction::ContinueRun => {
                channel_change.write(ChannelChangeMessage);
                if let Some(gw) = super::multiplayer_tab::state::connected_coop_guest_wizard(
                    &connection,
                    lobby.as_deref(),
                ) {
                    crate::game::multiplayer::coop::start_coop_host(
                        &mut commands,
                        &mut connection,
                        &mut config,
                        gw,
                        crate::networking::session::SessionMode::CoopRoguelite,
                    );
                }
                next_app_state.set(AppState::Loading);
            }
            RogueliteAction::EndRun => {
                // Save run to history
                if let Some(ref run) = roguelite_run {
                    let roguelite_run_data = RogueliteRun {
                        victory: false,
                        levels_completed: run.level_stats.len() as u32,
                        started_at: run.started_at,
                        ended_at: save_data::current_timestamp(),
                        wizard_type: config.wizard_type,
                        saved: false,
                        level_stats: run.level_stats.clone(),
                        modifiers: roguelite_modifiers.as_ref().map(|m| m.as_ref().clone()),
                        seed: config.seed,
                        active_toggles: active_toggles
                            .as_ref()
                            .map(|t| t.to_ids())
                            .unwrap_or_default(),
                        accessibility_assists: config.has_accessibility_assists(),
                        // Roguelite co-op tagging lands with multi-level co-op
                        // continuation (WS6); co-op is single-level today.
                        played_coop: false,
                        coop_peer_name: None,
                    };
                    save_data::save_roguelite_run(&active_save, roguelite_run_data);
                }

                // Clear the dormant run from disk
                save_data::clear_current_roguelite_run(&active_save);

                // Remove run resources
                commands.remove_resource::<RogueliteRunState>();
                commands.remove_resource::<RogueliteModifiers>();
                commands.remove_resource::<ActiveToggles>();
                commands.remove_resource::<GameMode>();

                // Panel rebuild is triggered automatically by the
                // resource_removed::<RogueliteRunState> run condition.
            }
            RogueliteAction::ChangeWizardType => {
                // Switch the right panel view to the wizard select grid
                commands.insert_resource(super::layout::RightPanelView::WizardSelect);
            }
        }
    }
}

/// Handles slider +/- button clicks.
pub(super) fn slider_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    down_buttons: Query<&ModifierSliderDownButton>,
    up_buttons: Query<&ModifierSliderUpButton>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    for event in button_clicked.read() {
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut modifiers, new_value);
        } else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut modifiers, new_value);
        }
    }
}

pub(super) fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut ModifierSliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &ModifierSliderTrack)>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    // Stop dragging when mouse is released
    if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
        for (_interaction, mut slider_handle) in &mut slider_handles {
            slider_handle.is_dragging = false;
        }
        return;
    }

    // Check if track was clicked (start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, _cursor_pos, track) in &slider_tracks {
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }

        // Also start dragging if the handle itself was clicked
        for (interaction, mut slider_handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                slider_handle.is_dragging = true;
            }
        }
    }

    // While dragging, use the track's RelativeCursorPosition to set the value.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging && let Some(pos) = cursor_pos.normalized {
            let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

            let min = track.value.min_value();
            let max = track.value.max_value();
            let range = max - min;
            let new_value = (min + normalized * range).clamp(min, max);

            // Snap to nearest step
            let step = track.value.step();
            let snapped = (new_value / step).round() * step;
            let snapped = snapped.clamp(min, max);

            if (track.value.get(&modifiers) - snapped).abs() > f32::EPSILON {
                track.value.set(&mut modifiers, snapped);
            }
        }
    }
}

/// Updates slider text displays when modifiers change.
pub(super) fn update_slider_text(
    modifiers: Res<RogueliteModifiers>,
    mut slider_texts: Query<(&mut Text, &ModifierSliderText)>,
) {
    if modifiers.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&modifiers);
            text.0 = format!("{}%", (value * 100.0) as u32);
        }
    }
}

/// Updates slider fill widths and handle positions when modifiers change.
pub(super) fn update_sliders(
    modifiers: Res<RogueliteModifiers>,
    mut slider_fills: Query<(&mut Node, &ModifierSliderFill), Without<ModifierSliderHandle>>,
    mut slider_handles: Query<(&mut Node, &ModifierSliderHandle), Without<ModifierSliderFill>>,
) {
    if modifiers.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&modifiers);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&modifiers);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0);
        }
    }
}

/// Toggles the expand/collapse state of a toggle modifier's description.
pub(super) fn toggle_expand_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut expanded: ResMut<ExpandedToggles>,
    mut descriptions: Query<(&ToggleDescriptionNode, &mut Node)>,
    expand_btn_entities: Query<(Entity, &ToggleExpandButton)>,
) {
    for event in button_clicked.read() {
        let Ok(btn) = expand_buttons.get(event.button) else {
            continue;
        };
        let toggle = btn.0;

        let is_expanded = expanded.0.contains(&toggle);
        if is_expanded {
            expanded.0.remove(&toggle);
        } else {
            expanded.0.insert(toggle);
        }
        let now_expanded = !is_expanded;

        // Update description visibility
        for (desc, mut node) in &mut descriptions {
            if desc.0 == toggle {
                node.display = if now_expanded {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // Toggle ButtonActive on the expand button
        for (entity, expand_btn) in &expand_btn_entities {
            if expand_btn.0 == toggle {
                if now_expanded {
                    commands.entity(entity).insert(ButtonActive);
                } else {
                    commands.entity(entity).remove::<ButtonActive>();
                }
            }
        }
    }
}

/// Toggles ON/OFF for unlocked toggle modifiers by clicking the row itself.
/// For locked toggles, clicking the row opens the unlock confirmation popup.
pub(super) fn toggle_row_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    row_query: Query<&ToggleRowContainer>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    existing_popup: Query<Entity, With<ConfirmUnlockPopup>>,
) {
    for event in button_clicked.read() {
        // Skip if the click was on the expand button
        if expand_buttons.get(event.button).is_ok() {
            continue;
        }

        let Ok(row) = row_query.get(event.button) else {
            continue;
        };
        let toggle = row.0;
        let is_unlocked = save_data::is_toggle_unlocked(toggle);

        if !is_unlocked {
            if existing_popup.is_empty() {
                spawn_unlock_popup(&mut commands, toggle);
            }
            continue;
        }

        // Unlocked -- toggle on/off
        pending_toggles.toggle(toggle);
        let now_enabled = pending_toggles.is_enabled(toggle);

        let (bg, border) = if now_enabled {
            (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
        } else {
            (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
        };

        for (entity, container, mut bg_color, mut border_color, mut btn_colors) in
            &mut row_containers
        {
            if container.0 == toggle {
                bg_color.0 = bg;
                *border_color = BorderColor::all(border);
                btn_colors.background = bg;
                btn_colors.border = border;

                if now_enabled {
                    commands.entity(entity).insert(ButtonActive);
                } else {
                    commands.entity(entity).remove::<ButtonActive>();
                }
            }
        }
    }
}

/// Handles confirm/cancel actions in the unlock confirmation popup.
pub(super) fn handle_unlock_confirmation(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&ConfirmUnlockAction>,
    popup_query: Query<Entity, With<ConfirmUnlockPopup>>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    unlock_cost_texts: Query<(Entity, &ToggleUnlockButton)>,
) {
    for event in button_clicked.read() {
        let Ok(action) = action_query.get(event.button) else {
            continue;
        };

        match action {
            ConfirmUnlockAction::Confirm(toggle) => {
                let toggle = *toggle;
                if save_data::unlock_toggle(toggle) {
                    pending_toggles.enabled.push(toggle);

                    for (entity, container, mut bg_color, mut border_color, mut btn_colors) in
                        &mut row_containers
                    {
                        if container.0 == toggle {
                            bg_color.0 = TOGGLE_ON_BG;
                            *border_color = BorderColor::all(TOGGLE_ON_BORDER);
                            commands.entity(entity).insert(ButtonActive);
                            btn_colors.background = TOGGLE_ON_BG;
                            btn_colors.border = TOGGLE_ON_BORDER;
                        }
                    }

                    for (entity, unlock_btn) in &unlock_cost_texts {
                        if unlock_btn.0 == toggle {
                            commands.entity(entity).try_despawn();
                            break;
                        }
                    }
                }
            }
            ConfirmUnlockAction::Cancel => {}
        }

        // Dismiss popup
        for entity in &popup_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Rebuilds the left panel summary text when modifiers or pending toggles change.
pub(super) fn update_run_summary(
    mut commands: Commands,
    modifiers: Option<Res<RogueliteModifiers>>,
    pending_toggles: Option<Res<PendingToggles>>,
    summary_query: Query<Entity, With<RunSummaryContent>>,
    children_query: Query<&Children>,
) {
    let (Some(modifiers), Some(pending_toggles)) = (modifiers, pending_toggles) else {
        return;
    };
    if !modifiers.is_changed() && !pending_toggles.is_changed() {
        return;
    }

    let Ok(summary_entity) = summary_query.single() else {
        return;
    };

    // Despawn all existing children
    if let Ok(children) = children_query.get(summary_entity) {
        for child in children.iter() {
            commands.entity(child).try_despawn();
        }
    }

    // Re-spawn fresh text nodes
    commands.entity(summary_entity).with_children(|summary| {
        spawn_summary_items(summary, &modifiers, &pending_toggles);
    });
}

/// Handles clicking the seed input box to focus it, or clicking Random / other buttons to unfocus.
pub(super) fn seed_input_click(
    mut button_clicked: MessageReader<MouseClicked>,
    input_boxes: Query<Entity, With<SeedInputBox>>,
    random_buttons: Query<Entity, With<SeedRandomButton>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    for event in button_clicked.read() {
        if input_boxes.get(event.button).is_ok() {
            seed_state.focused = !seed_state.focused;
            if seed_state.focused {
                seed_state.text.clear();
                for mut text in &mut text_query {
                    text.0 = String::new();
                }
            }
        } else if random_buttons.get(event.button).is_ok() {
            let new_seed = random_seed();
            seed_state.text = new_seed.to_string();
            seed_state.focused = false;
            config.seed = Some(new_seed);
            for mut text in &mut text_query {
                text.0 = new_seed.to_string();
            }
        } else if seed_state.focused {
            seed_state.focused = false;
            if seed_state.text.is_empty() {
                let new_seed = random_seed();
                seed_state.text = new_seed.to_string();
                config.seed = Some(new_seed);
                for mut text in &mut text_query {
                    text.0 = new_seed.to_string();
                }
            }
        }
    }

    // Update border color to indicate focus
    for mut border in &mut border_query {
        if seed_state.focused {
            *border = BorderColor::all(Color::hsla(270.0, 0.65, 0.55, 1.0));
        } else {
            *border = BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0));
        }
    }
}

/// Handles keyboard input when the seed field is focused.
pub(super) fn seed_input_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    // Ctrl+C: Copy seed to clipboard (works even when not focused)
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyC)
        && !seed_state.text.is_empty()
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        let _ = clipboard.set_text(seed_state.text.clone());
    }

    if !seed_state.focused {
        return;
    }

    let mut changed = false;

    // Ctrl+V: Paste seed from clipboard
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyV)
        && let Ok(text) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
    {
        let digits: String = text
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(MAX_SEED_CHARS)
            .collect();
        if !digits.is_empty() {
            seed_state.text = digits;
            changed = true;
        }
    }

    // Number keys (main keyboard)
    for (key, digit) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Numpad keys
    for (key, digit) in [
        (KeyCode::Numpad0, '0'),
        (KeyCode::Numpad1, '1'),
        (KeyCode::Numpad2, '2'),
        (KeyCode::Numpad3, '3'),
        (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'),
        (KeyCode::Numpad6, '6'),
        (KeyCode::Numpad7, '7'),
        (KeyCode::Numpad8, '8'),
        (KeyCode::Numpad9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        seed_state.text.pop();
        changed = true;
    }

    // Enter/Escape to unfocus
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        seed_state.focused = false;
        if seed_state.text.is_empty() {
            let new_seed = random_seed();
            seed_state.text = new_seed.to_string();
            config.seed = Some(new_seed);
            for mut text in &mut text_query {
                text.0 = new_seed.to_string();
            }
        }
        for mut border in &mut border_query {
            *border = BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0));
        }
        return;
    }

    if changed {
        config.seed = seed_state.text.parse::<u64>().ok();

        for mut text in &mut text_query {
            text.0 = seed_state.text.clone();
        }
    }
}

// ===========================================================================
// Cleanup
// ===========================================================================

/// Removes roguelite tab resources.
pub(super) fn cleanup_roguelite_tab_resources(mut commands: Commands) {
    commands.remove_resource::<SeedInputState>();
    commands.remove_resource::<ExpandedToggles>();
    commands.remove_resource::<PendingToggles>();
}
