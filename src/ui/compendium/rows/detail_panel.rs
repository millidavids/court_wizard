use super::super::setup::team_label_color;
use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::components::{CompendiumSpriteSpec, SHEET_ROW_FORWARD_FACING};
use crate::game::units::wizard::components::{
    Spell, effective_status_effects, spawn_status_effects_section,
};
use crate::ui::components::UnitCompendiumSpriteAssets;

use super::super::components::*;
use super::super::constants::*;

// ---------------------------------------------------------------------------
// Detail panel update
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_detail_panel(
    state: &CompendiumState,
    icon_assets: &crate::ui::components::SpellIconAssets,
    unit_sprite_assets: &UnitCompendiumSpriteAssets,
    unlocked_spells: &[String],
    unlocked_ingredients: &[String],
    unlocked_units: &[String],
    unlocked_wizard_types: &[String],
    unlocked_achievements: &[String],
    wizard_type: WizardType,
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
    status_container_q: &Query<Entity, With<DetailStatusContainer>>,
    commands: &mut Commands,
) {
    // Always start from a clean status section. Only the spell branch
    // re-populates it below.
    if let Ok(status_container) = status_container_q.single() {
        commands
            .entity(status_container)
            .despawn_related::<Children>();
    }

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
                .find(|s| s.save_key() == debug_name.as_str());
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
                if is_unlocked && let Ok(status_container) = status_container_q.single() {
                    let effects = effective_status_effects(*spell, wizard_type);
                    commands.entity(status_container).with_children(|parent| {
                        spawn_status_effects_section(
                            parent,
                            effects,
                            DETAIL_DESC_FONT_SIZE,
                            UNLOCKED_COLOR,
                            None,
                        );
                    });
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
                .find(|i| i.save_key() == debug_name.as_str());
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
                .find(|u| u.save_key() == debug_name.as_str());
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
                .find(|w| w.save_key() == debug_name.as_str());
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
                .find(|w| w.save_key() == debug_name.as_str());
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
