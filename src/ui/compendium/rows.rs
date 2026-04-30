//! Compendium row builders and detail panel update.

use super::setup::team_label_color;
use crate::ui::constants::efficiency_color;
use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::components::{CompendiumSpriteSpec, SHEET_ROW_FORWARD_FACING};
use crate::game::units::wizard::components::Spell;
use crate::ui::components::{ButtonColors, UnitCompendiumSpriteAssets};

use super::components::*;
use super::constants::*;

// ---------------------------------------------------------------------------
// Setup systems
// ---------------------------------------------------------------------------

pub(super) fn spawn_item_button(
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
            ButtonColors {
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

pub(super) fn spawn_stat_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
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

pub(super) fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), TEXT_COLOR);
}

pub(super) fn spawn_stat_text_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    spawn_stat_value_row(parent, label, value, TEXT_COLOR);
}

pub(super) fn spawn_insight_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
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

pub(super) fn spawn_level_history_rows(
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

// ---------------------------------------------------------------------------
// Detail panel update
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(super) fn update_detail_panel(
    state: &CompendiumState,
    icon_assets: &crate::ui::components::SpellIconAssets,
    unit_sprite_assets: &UnitCompendiumSpriteAssets,
    unlocked_spells: &[String],
    unlocked_ingredients: &[String],
    unlocked_units: &[String],
    unlocked_wizard_types: &[String],
    unlocked_achievements: &[String],
    title_q: &mut Query<
        &mut Text,
        (
            With<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    category_q: &mut Query<
        &mut Text,
        (
            With<DetailCategory>,
            Without<DetailTitle>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    desc_q: &mut Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailFlavor>,
        ),
    >,
    flavor_q: &mut Query<
        &mut Text,
        (
            With<DetailFlavor>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
        ),
    >,
    cat_color_q: &mut Query<&mut TextColor, (With<DetailCategory>, Without<DetailTitle>)>,
    detail_icon: &mut Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>,
) {
    // Helper to hide the icon and reset all per-item state so subsequent
    // shows start from a clean baseline (no leftover tint/atlas/size/corona).
    let hide_icon =
        |detail_icon: &mut Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>| {
            if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                node.display = Display::None;
                node.width = Val::Px(DETAIL_ICON_SIZE);
                node.height = Val::Px(DETAIL_ICON_SIZE);
                img.color = Color::WHITE;
                img.texture_atlas = None;
                shadow.0.clear();
            }
        };

    let apply_locked_appearance =
        |img: &mut ImageNode, shadow: &mut BoxShadow, is_unlocked: bool, base_tint: Color| {
            if is_unlocked {
                img.color = base_tint;
                shadow.0.clear();
            } else {
                img.color = Color::BLACK;
                shadow.0 = vec![locked_silhouette_corona()];
            }
        };

    let Some(ref item) = state.selected_item else {
        hide_icon(detail_icon);

        let (title, subtitle) = match state.active_tab {
            CompendiumTab::Stats => ("Level History", "Best efficiency per level"),
            CompendiumTab::Endless => ("Endless Mode", "Select a wizard to view stats"),
            CompendiumTab::Roguelite => ("Roguelite Mode", "Select a run to view details"),
            _ => ("Select an item", ""),
        };
        if let Ok(mut t) = title_q.single_mut() {
            **t = title.to_string();
        }
        if let Ok(mut t) = category_q.single_mut() {
            **t = subtitle.to_string();
        }
        if let Ok(mut t) = desc_q.single_mut() {
            **t = String::new();
        }
        if let Ok(mut t) = flavor_q.single_mut() {
            **t = String::new();
        }
        return;
    };

    match item {
        CompendiumItemId::Spell(debug_name) => {
            let spell = Spell::all()
                .iter()
                .find(|s| format!("{:?}", s) == *debug_name);
            if let Some(spell) = spell {
                let is_unlocked =
                    spell.research_cost() == 0 || unlocked_spells.contains(debug_name);

                // Show spell icon — silhouette + corona when locked, full color when unlocked.
                if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                    match icon_assets.get(spell) {
                        Some(handle) => {
                            img.image = handle.clone();
                            img.texture_atlas = None;
                            node.width = Val::Px(DETAIL_ICON_SIZE);
                            node.height = Val::Px(DETAIL_ICON_SIZE);
                            node.display = Display::Flex;
                            apply_locked_appearance(
                                &mut img,
                                &mut shadow,
                                is_unlocked,
                                Color::WHITE,
                            );
                        }
                        None => {
                            node.display = Display::None;
                            shadow.0.clear();
                        }
                    }
                }

                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        spell.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        format!(
                            "{} - {}",
                            spell.category().display_name(),
                            spell.damage_type().display_name()
                        )
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut c) = cat_color_q.single_mut() {
                    c.0 = if is_unlocked {
                        crate::ui::constants::spell_category_color(spell.category())
                    } else {
                        DESCRIPTION_COLOR
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        format!("{}\n\n{}", spell.description(), spell.instructions())
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = spell.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Ingredient(debug_name) => {
            hide_icon(detail_icon);
            let ingredient = Ingredient::all()
                .iter()
                .find(|i| format!("{:?}", i) == *debug_name);
            if let Some(ingredient) = ingredient {
                let is_unlocked = unlocked_ingredients.contains(debug_name);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.category().display_name().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        ingredient.description()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = ingredient.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Unit(debug_name) => {
            let unit_type = UnitType::all()
                .iter()
                .find(|u| format!("{:?}", u) == *debug_name);
            if let Some(unit_type) = unit_type {
                let is_unlocked = unlocked_units.contains(debug_name);

                // Render the unit portrait — colored when unlocked, BLACK silhouette
                // with a white corona when locked.
                if let Ok((mut img, mut node, mut shadow)) = detail_icon.single_mut() {
                    match unit_sprite_assets.images.get(unit_type) {
                        Some(image_handle) => {
                            let spec = unit_type.compendium_sprite();
                            img.image = image_handle.clone();
                            img.texture_atlas = match spec {
                                CompendiumSpriteSpec::Atlas {
                                    sheet_size,
                                    frame_px,
                                    ..
                                } => unit_sprite_assets.layouts.get(unit_type).map(|layout| {
                                    let cols = (sheet_size.0 / frame_px) as usize;
                                    TextureAtlas {
                                        layout: layout.clone(),
                                        index: SHEET_ROW_FORWARD_FACING * cols,
                                    }
                                }),
                                CompendiumSpriteSpec::Static { .. } => None,
                            };
                            let (base_tint, size_mul) = match spec {
                                CompendiumSpriteSpec::Static {
                                    size_multiplier, ..
                                } => (Color::WHITE, size_multiplier),
                                CompendiumSpriteSpec::Atlas {
                                    tint,
                                    size_multiplier,
                                    ..
                                } => (tint, size_multiplier),
                            };
                            apply_locked_appearance(&mut img, &mut shadow, is_unlocked, base_tint);
                            let px = DETAIL_UNIT_ICON_SIZE * size_mul;
                            node.width = Val::Px(px);
                            node.height = Val::Px(px);
                            node.display = Display::Flex;
                        }
                        None => hide_icon(detail_icon),
                    }
                }
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.team_label().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut c) = cat_color_q.single_mut() {
                    c.0 = if is_unlocked {
                        team_label_color(unit_type.team_label())
                    } else {
                        DESCRIPTION_COLOR
                    };
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.description().to_string()
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = if is_unlocked {
                        unit_type.flavor_text().to_string()
                    } else {
                        unit_type.locked_description().to_string()
                    };
                }
                return; // Early return to skip the category color reset below
            }
        }
        CompendiumItemId::Wizard(debug_name) => {
            hide_icon(detail_icon);
            let wizard_type = WizardType::all()
                .iter()
                .find(|w| format!("{:?}", w) == *debug_name);
            if let Some(wizard_type) = wizard_type {
                let is_unlocked = *wizard_type == WizardType::BoringOleMage
                    || unlocked_wizard_types.contains(debug_name);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        wizard_type.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = "Wizard Type".to_string();
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = if is_unlocked {
                        format!(
                            "{}\n\n{}",
                            wizard_type.description(),
                            wizard_type.long_description()
                        )
                    } else {
                        String::new()
                    };
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = wizard_type.locked_description().to_string();
                }
            }
        }
        CompendiumItemId::Achievement(id_str) => {
            hide_icon(detail_icon);
            let achievement = AchievementId::all()
                .iter()
                .find(|a| a.id() == id_str.as_str());
            if let Some(achievement) = achievement {
                let is_unlocked = unlocked_achievements.contains(id_str);
                if let Ok(mut t) = title_q.single_mut() {
                    **t = if is_unlocked {
                        achievement.display_name().to_string()
                    } else {
                        "???".to_string()
                    };
                }
                if let Ok(mut t) = category_q.single_mut() {
                    **t = "Achievement".to_string();
                }
                if let Ok(mut t) = desc_q.single_mut() {
                    **t = achievement.description().to_string();
                }
                if let Ok(mut t) = flavor_q.single_mut() {
                    **t = if is_unlocked {
                        achievement.unlock_reward().unwrap_or("").to_string()
                    } else {
                        String::new()
                    };
                }
            }
        }
        CompendiumItemId::EndlessWizardType(debug_name) => {
            hide_icon(detail_icon);
            let wizard_type = WizardType::all()
                .iter()
                .find(|w| format!("{:?}", w) == *debug_name);
            if let Ok(mut t) = title_q.single_mut() {
                **t = wizard_type
                    .map(|w| format!("{} — Endless", w.display_name()))
                    .unwrap_or_else(|| "Endless Mode".to_string());
            }
            if let Ok(mut t) = category_q.single_mut() {
                **t = "Best stats per level".to_string();
            }
            // desc/flavor hidden by level history display
            if let Ok(mut t) = desc_q.single_mut() {
                **t = String::new();
            }
            if let Ok(mut t) = flavor_q.single_mut() {
                **t = String::new();
            }
        }
        CompendiumItemId::RogueliteRun(_) => {
            hide_icon(detail_icon);
            if let Ok(mut t) = title_q.single_mut() {
                **t = "Run Details".to_string();
            }
            if let Ok(mut t) = category_q.single_mut() {
                **t = "Level-by-level breakdown".to_string();
            }
            // desc/flavor hidden by level history display
            if let Ok(mut t) = desc_q.single_mut() {
                **t = String::new();
            }
            if let Ok(mut t) = flavor_q.single_mut() {
                **t = String::new();
            }
        }
    }

    // Reset category color to default for non-unit items
    if let Ok(mut c) = cat_color_q.single_mut()
        && !matches!(state.selected_item, Some(CompendiumItemId::Unit(_)))
    {
        c.0 = DESCRIPTION_COLOR;
    }
}
