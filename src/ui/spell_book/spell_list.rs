//! The right panel: unlocked spells grouped into scrollable category columns.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::units::wizard::components::{Spell, SpellCategory};
use crate::ui::components::{ButtonColors, SpellIconAssets};

/// Moves the selection border to the previewed spell's row.
pub(super) fn refresh_spell_list_selection(
    mut commands: Commands,
    selected: Res<SelectedSpellPreview>,
    mut rows: Query<(Entity, &SpellListButton, &mut BorderColor)>,
) {
    if !selected.is_changed() {
        return;
    }

    for (entity, row, mut border) in rows.iter_mut() {
        let is_selected = row.0 == selected.0;
        *border = BorderColor::all(if is_selected {
            SPELL_BUTTON_SELECTED_BORDER
        } else {
            SPELL_BUTTON_BORDER
        });
        if is_selected {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
        }
    }
}

/// Spawns the right panel with 4 category columns, each scrollable.
pub(super) fn spawn_spell_list(
    parent: &mut ChildSpawnerCommands,
    selected: Spell,
    is_unlocked: &dyn Fn(&Spell) -> bool,
    icon_assets: &SpellIconAssets,
) {
    let right_id = crate::ui::systems::spawn_right_scroll_panel(
        parent,
        ScrollableSpellList,
        FlexDirection::Row,
        LIST_ITEM_GAP,
    );

    parent.commands().entity(right_id).with_children(|list| {
        for category in SpellCategory::all() {
            let mut unlocked_in_category: Vec<Spell> = category
                .spells()
                .iter()
                .copied()
                .filter(|s| is_unlocked(s))
                .collect();

            if unlocked_in_category.is_empty() {
                continue;
            }

            // Sort alphabetically by display name
            unlocked_in_category.sort_by_key(|s| s.display_name());

            // Category column
            list.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(LIST_ITEM_GAP),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|column| {
                // Category header
                column.spawn((
                    Text::new(category.display_name()),
                    TextFont::from_font_size(CATEGORY_FONT_SIZE),
                    TextColor(crate::ui::constants::spell_category_color(*category)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));

                // One spell button per row
                for spell in &unlocked_in_category {
                    spawn_spell_button(column, *spell, *spell == selected, icon_assets);
                }
            });
        }
    });
}

/// Spawns one spell row: icon plus display name.
fn spawn_spell_button(
    column: &mut ChildSpawnerCommands,
    spell: Spell,
    is_selected: bool,
    icon_assets: &SpellIconAssets,
) {
    let border = if is_selected {
        SPELL_BUTTON_SELECTED_BORDER
    } else {
        SPELL_BUTTON_BORDER
    };

    column
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(SPELL_BUTTON_HEIGHT),
                border: UiRect::all(Val::Px(SPELL_BUTTON_BORDER_WIDTH)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                padding: UiRect::horizontal(Val::Px(6.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(SPELL_BUTTON_BG),
            BorderColor::all(border),
            ButtonColors {
                background: SPELL_BUTTON_BG,
                border: SPELL_BUTTON_BORDER,
            },
            SpellBookButtonAction::SelectSpell(spell),
            SpellListButton(spell),
            crate::ui::focus::Focusable,
            // Left / Right can cross to the left detail panel regardless of
            // row alignment.
            crate::ui::focus::CrossRowHorizontalNav,
        ))
        .with_children(|btn| {
            if let Some(icon_handle) = icon_assets.get(&spell) {
                btn.spawn((
                    ImageNode::new(icon_handle.clone()),
                    Node {
                        width: Val::Px(SPELL_ICON_SIZE),
                        height: Val::Px(SPELL_ICON_SIZE),
                        ..default()
                    },
                ));
            }
            btn.spawn((
                Text::new(spell.display_name()),
                TextFont::from_font_size(SPELL_BUTTON_FONT_SIZE),
                TextColor(SPELL_BUTTON_TEXT_COLOR),
            ));
        });
}
