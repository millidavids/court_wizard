use bevy::prelude::*;

use crate::config::save_data::load_unified_save;
use crate::ui::components::{ButtonFront, UnitCompendiumSpriteAssets};

use super::super::components::*;
use super::super::constants::*;
use super::super::rows::update_detail_panel;
use super::endless::{spawn_endless_detail_for_wizard, spawn_endless_items};
use super::item_spawners::{
    spawn_achievement_items, spawn_ingredient_items, spawn_spell_items, spawn_stats_items,
    spawn_unit_items, spawn_wizard_items,
};
use super::roguelite::{spawn_roguelite_items, spawn_roguelite_run_detail};

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn rebuild_on_state_change(
    mut commands: Commands,
    mut state: ResMut<CompendiumState>,
    // Bundled to dodge Bevy's 16-param SystemParam tuple limit.
    detail_panel_inputs: (
        Res<crate::ui::components::SpellIconAssets>,
        Res<UnitCompendiumSpriteAssets>,
        Res<crate::config::GameConfig>,
        Query<Entity, With<DetailStatusContainer>>,
    ),
    items_container: Query<Entity, With<ItemsContainer>>,
    tab_buttons: Query<(&TabButton, Entity, &Children)>,
    mut tab_bg: Query<(
        &mut BackgroundColor,
        &mut BorderColor,
        &mut crate::ui::components::ButtonColors,
    )>,
    mut front_bg: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (
            With<ButtonFront>,
            Without<crate::ui::components::ButtonColors>,
        ),
    >,
    mut detail_title: Query<
        &mut Text,
        (
            With<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_category: Query<
        &mut Text,
        (
            With<DetailCategory>,
            Without<DetailTitle>,
            Without<DetailDescription>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_desc: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailFlavor>,
        ),
    >,
    mut detail_flavor: Query<
        &mut Text,
        (
            With<DetailFlavor>,
            Without<DetailTitle>,
            Without<DetailCategory>,
            Without<DetailDescription>,
        ),
    >,
    mut detail_cat_color: Query<&mut TextColor, (With<DetailCategory>, Without<DetailTitle>)>,
    mut detail_icon: Query<(&mut ImageNode, &mut Node, &mut BoxShadow), With<DetailIcon>>,
    level_history: Query<Entity, With<LevelHistoryContainer>>,
    mut detail_desc_node: Query<
        &mut Node,
        (
            With<DetailDescription>,
            Without<DetailIcon>,
            Without<LevelHistoryContainer>,
        ),
    >,
    mut detail_flavor_node: Query<
        &mut Node,
        (
            With<DetailFlavor>,
            Without<DetailIcon>,
            Without<DetailDescription>,
            Without<LevelHistoryContainer>,
        ),
    >,
) {
    if !state.is_changed() {
        return;
    }

    let tab_changed = state.prev_tab != Some(state.active_tab);
    if tab_changed {
        state.prev_tab = Some(state.active_tab);
    }

    // Update tab button visuals (wrapper ButtonColors + front face colors).
    for (tab_btn, entity, children) in &tab_buttons {
        let is_active = tab_btn.0 == state.active_tab;
        let (bg, border) = if is_active {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
            (INACTIVE_TAB_BG, TAB_BORDER)
        };
        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_bg.get_mut(entity) {
            *bg_color = bg.into();
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }
        // Also update the 3D front face child.
        for child in children.iter() {
            if let Ok((mut front_bg_color, mut front_border_color)) = front_bg.get_mut(child) {
                *front_bg_color = crate::ui::systems::opaque(bg).into();
                *front_border_color = BorderColor::all(border);
            }
        }
    }

    // Load save data
    let save = load_unified_save();
    let unlocked_achievements: Vec<String> = save
        .as_ref()
        .map(|s| s.player.unlocked_achievements.clone())
        .unwrap_or_default();
    let unlocked_content = save
        .as_ref()
        .map(|s| s.player.unlocked_content.clone())
        .unwrap_or_default();
    let research_progress = save
        .as_ref()
        .map(|s| s.player.spell_research_progress.clone())
        .unwrap_or_default();

    // Rebuild items list
    // Only rebuild the item list when the tab changes — not on item selection.
    if tab_changed {
        match items_container.single() {
            Err(_) => {
                warn!(
                    "rebuild_on_state_change: expected exactly one ItemsContainer but found none or multiple; skipping item list rebuild"
                );
            }
            Ok(container) => {
                commands.entity(container).despawn_related::<Children>();
                commands
                    .entity(container)
                    .with_children(|parent| match state.active_tab {
                        CompendiumTab::Spells => spawn_spell_items(
                            parent,
                            &unlocked_content.spells,
                            &research_progress,
                            &state,
                        ),
                        CompendiumTab::Ingredients => {
                            spawn_ingredient_items(parent, &unlocked_content.ingredients, &state)
                        }
                        CompendiumTab::Units => {
                            spawn_unit_items(parent, &unlocked_content.units, &state)
                        }
                        CompendiumTab::Wizards => {
                            spawn_wizard_items(parent, &unlocked_content.wizard_types, &state)
                        }
                        CompendiumTab::Achievements => {
                            spawn_achievement_items(parent, &unlocked_achievements, &state)
                        }
                        CompendiumTab::Stats => spawn_stats_items(parent, save.as_ref()),
                        CompendiumTab::Endless => {
                            spawn_endless_items(parent, save.as_ref(), &state)
                        }
                        CompendiumTab::Roguelite => {
                            spawn_roguelite_items(parent, save.as_ref(), &state)
                        }
                    });
            }
        }
    }

    // Update detail panel (including icon)
    let (icon_assets, unit_sprite_assets, config, status_container_q) = (
        &detail_panel_inputs.0,
        &detail_panel_inputs.1,
        &detail_panel_inputs.2,
        &detail_panel_inputs.3,
    );
    update_detail_panel(
        &state,
        icon_assets,
        unit_sprite_assets,
        &unlocked_content.spells,
        &unlocked_content.ingredients,
        &unlocked_content.units,
        &unlocked_content.wizard_types,
        &unlocked_achievements,
        config.wizard_type,
        &mut detail_title,
        &mut detail_category,
        &mut detail_desc,
        &mut detail_flavor,
        &mut detail_cat_color,
        &mut detail_icon,
        status_container_q,
        &mut commands,
    );

    // Determine if the level history container should be shown
    // Stats tab: always show level history (no selectable items)
    // Endless/Roguelite: show detail when an item IS selected
    let show_level_history = match state.active_tab {
        CompendiumTab::Stats => state.selected_item.is_none(),
        CompendiumTab::Endless => matches!(
            state.selected_item,
            Some(CompendiumItemId::EndlessWizardType(_))
        ),
        CompendiumTab::Roguelite => {
            matches!(state.selected_item, Some(CompendiumItemId::RogueliteRun(_)))
        }
        _ => false,
    };
    if let Ok(container) = level_history.single() {
        commands.entity(container).despawn_related::<Children>();
        if show_level_history {
            // Show level history, hide description/flavor text
            commands.entity(container).insert(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::Flex,
                ..default()
            });
            let selected = state.selected_item.clone();
            commands.entity(container).with_children(|parent| {
                match (&state.active_tab, &selected) {
                    (CompendiumTab::Endless, Some(CompendiumItemId::EndlessWizardType(name))) => {
                        spawn_endless_detail_for_wizard(parent, save.as_ref(), name);
                    }
                    (
                        CompendiumTab::Roguelite,
                        Some(CompendiumItemId::RogueliteRun(started_at)),
                    ) => {
                        spawn_roguelite_run_detail(parent, save.as_ref(), *started_at);
                    }
                    _ => super::super::rows::spawn_level_history_rows(parent, save.as_ref()),
                }
            });
        } else {
            commands.entity(container).insert(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                display: Display::None,
                ..default()
            });
        }
    }

    // Hide/show description and flavor text based on level history display
    if let Ok(mut node) = detail_desc_node.single_mut() {
        node.display = if show_level_history {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut node) = detail_flavor_node.single_mut() {
        node.display = if show_level_history {
            Display::None
        } else {
            Display::Flex
        };
    }
}
