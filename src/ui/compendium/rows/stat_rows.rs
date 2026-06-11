use crate::ui::constants::efficiency_color;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;

// ---------------------------------------------------------------------------
// Setup systems
// ---------------------------------------------------------------------------

pub(crate) fn spawn_item_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    text_color: Color,
    item_id: CompendiumItemId,
    selected: &Option<CompendiumItemId>,
) {
    let is_selected = *selected == Some(item_id.clone());
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(ITEM_BG),
            BorderColor::all(ITEM_BORDER),
            crate::ui::components::ButtonColors {
                background: ITEM_BG,
                border: ITEM_BORDER,
            },
            crate::ui::focus::Focusable,
            ItemButton(item_id),
        ))
        .insert_if(crate::ui::components::ButtonActive, || is_selected)
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(text_color),
            ));
        });
}

// ---------------------------------------------------------------------------
// Stat rows (for stats tab)
// ---------------------------------------------------------------------------

pub(crate) fn spawn_stat_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font_size: STAT_SECTION_FONT_SIZE,
            ..default()
        },
        TextColor(STAT_SECTION_COLOR),
        Node {
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(2.0)),
            ..default()
        },
    ));
}

pub(crate) fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), TEXT_COLOR);
}

pub(crate) fn spawn_stat_text_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    spawn_stat_value_row(parent, label, value, TEXT_COLOR);
}

pub(crate) fn spawn_insight_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), INSIGHT_COLOR);
}

pub(super) fn spawn_stat_value_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    value_color: Color,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(STAT_LABEL_COLOR),
            ));
            row.spawn((
                Text::new(value),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(value_color),
            ));
        });
}

// ---------------------------------------------------------------------------
// Level history for stats detail panel
// ---------------------------------------------------------------------------

pub(crate) fn spawn_level_history_rows(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) {
    use crate::game::constants::boss_name_for_level;

    let Some(save) = save else {
        parent.spawn((
            Text::new("No data yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    };

    // Aggregate best efficiency per level across all wizards
    let mut best_efficiency: std::collections::BTreeMap<u32, f32> =
        std::collections::BTreeMap::new();

    for wizard in &save.wizards {
        for (level_str, &eff) in &wizard.efficiency_ratios {
            if let Ok(level) = level_str.parse::<u32>() {
                let entry = best_efficiency.entry(level).or_insert(0.0);
                if eff > *entry {
                    *entry = eff;
                }
            }
        }
    }

    if best_efficiency.is_empty() {
        parent.spawn((
            Text::new("No levels completed yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (level, eff) in &best_efficiency {
        let label = if let Some(boss) = boss_name_for_level(*level) {
            format!("Level {} ({}):", level, boss)
        } else {
            format!("Level {}:", level)
        };
        let pct_text = format!("{:.0}%", eff * 100.0);
        let color = efficiency_color(*eff);

        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 1.0,
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(pct_text),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        min_width: Val::Px(40.0),
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Right),
                ));
            });
    }
}
