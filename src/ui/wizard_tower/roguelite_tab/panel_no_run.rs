use bevy::prelude::*;

use crate::config::GameConfig;
use crate::config::save_data;
use crate::game::game_mode::components::{RogueliteModifiers, ToggleModifier};
use crate::ui::systems::spawn_button;

use super::super::constants::SECTION_PADDING;
use super::super::constants::spawn_coop_gated_button;
use super::components::RogueliteAction;
use super::components::{ModifierSliderValue, PendingToggles, RunSummaryContent};
use super::constants::{
    CHANGE_WIZARD_BUTTON_STYLE, LABEL_COLOR, SECTION_HEADER_FONT_SIZE, SECTION_MARGIN,
    START_RUN_BUTTON_STYLE, SUMMARY_ITEM_COLOR, SUMMARY_ITEM_FONT_SIZE, SUMMARY_PLACEHOLDER_COLOR,
    SUMMARY_TITLE_FONT_SIZE,
};
use super::seed_input::spawn_seed_input_row;
use super::slider::spawn_modifier_slider;
use super::toggle_spawn::spawn_toggle_row;

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
pub(super) fn spawn_summary_items(
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

/// Builds the right panel content for the "no active run" state.
/// Shows modifier sliders, toggle section, seed input, and action buttons.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_roguelite_no_run_right_panel(
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
                            Val::Px(super::constants::ROW_GAP),
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
pub(crate) fn build_roguelite_no_run_left_panel(
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
