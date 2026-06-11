use bevy::prelude::*;

use super::detail_panel::spawn_detail_panel;
use super::ingredient_list::spawn_ingredient_list;
use crate::config::GameConfig;
use crate::config::save_data::load_unified_save;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::resources::PhilosophersStoneUsed;
use crate::ui::cauldron_menu::components::*;
use crate::ui::cauldron_menu::constants::*;
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

/// Spawns the cauldron menu UI when entering the CauldronMenu state.
pub(crate) fn spawn_cauldron_menu_ui(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
    config: Res<GameConfig>,
    stone_used: Res<PhilosophersStoneUsed>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    build_menu(&mut commands, is_brewing, &selection, &config, &stone_used);
}

/// Despawns the menu when the cauldron state changes (e.g. brew completes in urgent mode).
/// `respawn_menu_on_toggle` will rebuild the menu next frame with the updated state.
pub(crate) fn rebuild_menu_on_brew_state_change(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, (With<Cauldron>, Changed<CauldronState>)>,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    if let Ok(state) = cauldron_query.single()
        && !state.is_brewing()
    {
        for entity in &menu_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Re-spawns the menu UI if it was despawned by a toggle action.
pub(crate) fn respawn_menu_on_toggle(
    mut commands: Commands,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
    config: Res<GameConfig>,
    stone_used: Res<PhilosophersStoneUsed>,
) {
    if menu_query.iter().next().is_none() {
        let is_brewing = cauldron_query
            .single()
            .is_ok_and(|state| state.is_brewing());

        build_menu(&mut commands, is_brewing, &selection, &config, &stone_used);
    }
}

/// Builds the cauldron menu UI tree with a two-panel layout.
fn build_menu(
    commands: &mut Commands,
    is_brewing: bool,
    selection: &IngredientSelection,
    config: &GameConfig,
    stone_used: &PhilosophersStoneUsed,
) {
    // Load save data once for unlocked ingredients and combos
    let save = load_unified_save();
    let unlocked_ingredients = save
        .as_ref()
        .map(|s| s.player.unlocked_content.ingredients.clone())
        .unwrap_or_default();
    let unlocked_combos = save
        .as_ref()
        .map(|s| s.player.unlocked_content.combos.clone())
        .unwrap_or_default();

    // Page container (standard overlay with content box). `ModalOverlay`
    // scopes focus to this menu so HUD buttons behind it aren't reachable.
    let content = spawn_page_container(
        commands,
        OnCauldronMenuScreen,
        false,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(LAYOUT_PADDING)),
            row_gap: Val::Px(16.0),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );
    commands
        .entity(content)
        .insert(crate::ui::focus::ModalOverlay);

    commands.entity(content).with_children(|root| {
        // Header row: title left, Back button right
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|header| {
            spawn_title_with_shadow(
                header,
                "Cauldron",
                TITLE_FONT_SIZE,
                TITLE_COLOR,
                Node::default(),
            );
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            if is_brewing {
                spawn_button(
                    header,
                    "Cancel Brew",
                    CauldronMenuButtonAction::CancelBrew,
                    &CANCEL_BUTTON_STYLE,
                );
            }
            spawn_button(
                header,
                "Back",
                (
                    CauldronMenuButtonAction::Close,
                    crate::ui::focus::NoGamepadFocus,
                ),
                &crate::ui::main_menu::BACK_BUTTON_STYLE,
            );
        });

        // Content area: two-panel row (centered when brewing)
        root.spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(COLUMN_GAP),
            justify_content: if is_brewing {
                JustifyContent::Center
            } else {
                JustifyContent::default()
            },
            align_items: if is_brewing {
                AlignItems::Center
            } else {
                AlignItems::default()
            },
            ..default()
        })
        .with_children(|content| {
            // === Left panel: detail/preview ===
            spawn_detail_panel(
                content,
                is_brewing,
                selection,
                &unlocked_combos,
                config,
                stone_used,
            );

            // === Right panel: categorized ingredient grid ===
            if !is_brewing {
                spawn_ingredient_list(content, selection, &unlocked_ingredients);
            }
        });
    });
}
