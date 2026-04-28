use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::save_data::load_unified_save;
use crate::config::{GameConfig, WizardType};
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::components::{Spell, SpellCategory};
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::networking::session::MultiplayerSession;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::components::{ButtonColors, SpellIconAssets};
use crate::ui::concentration::ConcentrationUIRoot;
use crate::ui::systems::{spawn_button, spawn_page_container};

/// Resource to track when we just entered the spell book.
/// Prevents spell casting on the same frame as opening the spell book.
#[derive(Resource, Default)]
pub(super) struct JustEnteredSpellBook(pub bool);

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

/// Spawns the spell book UI when entering the SpellBook state.
pub(super) fn spawn_spell_book_ui(
    mut commands: Commands,
    config: Res<GameConfig>,
    mp_session: Option<Res<MultiplayerSession>>,
    icon_assets: Res<SpellIconAssets>,
) {
    // In multiplayer, all spells are available regardless of single-player progression.
    let is_multiplayer = mp_session.is_some();

    let unlocked_spells: Vec<String> = if is_multiplayer {
        Vec::new() // Not needed — is_unlocked always returns true
    } else {
        load_unified_save()
            .map(|s| s.player.unlocked_content.spells)
            .unwrap_or_default()
    };

    let is_shepherd = config.wizard_type == WizardType::Shepherd;
    let is_unlocked = move |spell: &Spell| {
        if is_shepherd && !spell.is_shepherd_allowed() {
            return false;
        }
        if is_multiplayer {
            return true;
        }
        let debug_name = format!("{:?}", spell);
        unlocked_spells.contains(&debug_name)
    };

    // Pick initial selected spell: first unlocked spell, or MagicMissile as fallback
    let initial_spell = SpellCategory::all()
        .iter()
        .flat_map(|cat| cat.spells().iter())
        .find(|s| is_unlocked(s))
        .copied()
        .unwrap_or(Spell::MagicMissile);

    commands.insert_resource(SelectedSpellPreview(initial_spell));

    // Page container with column layout (header + two-panel content).
    // `ModalOverlay` scopes gamepad focus to this overlay so the Spells /
    // Cauldron HUD buttons rendering behind it don't pick up focus.
    let content = spawn_page_container(
        &mut commands,
        OnSpellBookScreen,
        false,
        crate::ui::systems::default_content_node(),
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
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|header| {
            crate::ui::systems::spawn_title_with_shadow(
                header,
                "Spells",
                36.0,
                crate::ui::constants::TEXT_PRIMARY,
                Node::default(),
            );
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_button(
                header,
                "Back",
                (
                    SpellBookButtonAction::Close,
                    crate::ui::focus::NoGamepadFocus,
                ),
                &crate::ui::main_menu::BACK_BUTTON_STYLE,
            );
        });

        // Two-panel content row
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(crate::ui::constants::TWO_PANEL_GAP),
            ..default()
        })
        .with_children(|panels| {
            // === Left panel: detail + buttons ===
            spawn_detail_panel(panels, initial_spell, &config);

            // === Right panel: categorized spell list ===
            spawn_spell_list(panels, initial_spell, &is_unlocked, &icon_assets);
        });
    });
}

/// Spawns the left detail panel showing spell info, hotkeys, and action buttons.
fn spawn_detail_panel(parent: &mut ChildSpawnerCommands, spell: Spell, config: &GameConfig) {
    let detail_box = crate::ui::systems::spawn_left_detail_panel(parent);

    parent.commands().entity(detail_box).with_children(|panel| {
        // Spell name
        panel.spawn((
            Text::new(spell.display_name()),
            TextFont::from_font_size(DETAIL_NAME_FONT_SIZE),
            TextColor(DETAIL_NAME_COLOR),
            DetailName,
        ));

        // Damage type
        panel.spawn((
            Text::new(spell.damage_type().display_name()),
            TextFont::from_font_size(DETAIL_TYPE_FONT_SIZE),
            TextColor(DETAIL_TYPE_COLOR),
            DetailDamageType,
        ));

        // Description
        panel.spawn((
            Text::new(spell.description()),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(DETAIL_DESC_COLOR),
            Node {
                max_width: Val::Px(LEFT_PANEL_WIDTH - DETAIL_PADDING * 2.0),
                ..default()
            },
            DetailDescription,
        ));

        // Instructions
        panel.spawn((
            Text::new(spell.instructions()),
            TextFont::from_font_size(DETAIL_INSTRUCTIONS_FONT_SIZE),
            TextColor(DETAIL_INSTRUCTIONS_COLOR),
            DetailInstructions,
        ));

        // Hotkey section
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|hotkey_section| {
                // Label
                hotkey_section.spawn((
                    Text::new("Assign Hotkey"),
                    TextFont::from_font_size(LABEL_FONT_SIZE),
                    TextColor(LABEL_COLOR),
                ));

                // Hotkey boxes row
                hotkey_section
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(HOTKEY_BOX_GAP),
                        ..default()
                    })
                    .with_children(|row| {
                        for slot in 0..5u8 {
                            let is_active = config.action_bar_slots[slot as usize] == Some(spell);
                            let (bg, border, text_color) = if is_active {
                                (HOTKEY_ACTIVE_BG, HOTKEY_ACTIVE_BORDER, HOTKEY_ACTIVE_TEXT)
                            } else {
                                (
                                    HOTKEY_INACTIVE_BG,
                                    HOTKEY_INACTIVE_BORDER,
                                    HOTKEY_INACTIVE_TEXT,
                                )
                            };

                            let mut hotkey_btn = row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(HOTKEY_BOX_SIZE),
                                    height: Val::Px(HOTKEY_BOX_SIZE),
                                    border: UiRect::all(Val::Px(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                BorderColor::all(border),
                                ButtonColors {
                                    background: bg,
                                    border,
                                },
                                HotkeySlotButton(slot),
                                crate::ui::focus::Focusable,
                                crate::ui::focus::CrossRowHorizontalNav,
                            ));
                            if is_active {
                                hotkey_btn.insert(crate::ui::components::ButtonActive);
                            }
                            hotkey_btn.with_children(|btn| {
                                btn.spawn((
                                    Text::new(format!("{}", slot + 1)),
                                    TextFont::from_font_size(HOTKEY_FONT_SIZE),
                                    TextColor(text_color),
                                ));
                            });
                        }
                    });
            });

        // Selecting a spell from the right list primes it and closes the
        // menu automatically, so no Select / Close buttons are needed here.
        // The header's Back button (B / Escape) still closes the menu.
    });
}

/// Spawns the right panel with 4 category columns, each scrollable.
fn spawn_spell_list(
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
                    TextLayout::new_with_justify(Justify::Center),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));

                // One spell button per row
                for spell in &unlocked_in_category {
                    let is_selected = *spell == selected;
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
                            SpellBookButtonAction::SelectSpell(*spell),
                            SpellListButton(*spell),
                            crate::ui::focus::Focusable,
                            // Left / Right can cross to the left detail
                            // panel regardless of row alignment.
                            crate::ui::focus::CrossRowHorizontalNav,
                        ))
                        .with_children(|btn| {
                            if let Some(icon_handle) = icon_assets.get(spell) {
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
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Interaction systems
// ---------------------------------------------------------------------------

/// Handles button click actions in the spell book.
///
/// Closes to the appropriate state (SP or MP) depending on which is active.
pub(super) fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SpellBookButtonAction>,
    mut selected: ResMut<SelectedSpellPreview>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                // Clicking a spell primes it but leaves the menu open so
                // the player can assign it to an action-bar slot before
                // returning to the battle. The menu is dismissed only via
                // the explicit close button or the back/B input.
                SpellBookButtonAction::SelectSpell(spell) => {
                    selected.0 = *spell;
                    prime_spell.write(PrimeSpellMessage {
                        spell: spell.primed_config(),
                    });
                }
                SpellBookButtonAction::Close => {
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::Running);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::Running);
                    }
                }
            }
        }
    }
}

/// Updates the detail panel text and hotkey highlights when the selected spell changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_detail_panel(
    selected: Res<SelectedSpellPreview>,
    config: Res<GameConfig>,
    mut name_query: Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut type_query: Query<
        &mut Text,
        (
            With<DetailDamageType>,
            Without<DetailName>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailInstructions>,
        ),
    >,
    mut instr_query: Query<
        &mut Text,
        (
            With<DetailInstructions>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
        ),
    >,
    mut hotkey_query: Query<(
        Entity,
        &HotkeySlotButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
    )>,
    children_query: Query<&Children>,
    mut hotkey_text_query: Query<&mut TextColor>,
    mut spell_list_query: Query<(&SpellListButton, &mut BorderColor), Without<HotkeySlotButton>>,
) {
    if !selected.is_changed() && !config.is_changed() {
        return;
    }

    let spell = selected.0;

    // Update detail text
    if let Ok(mut text) = name_query.single_mut() {
        **text = spell.display_name().to_string();
    }
    if let Ok(mut text) = type_query.single_mut() {
        **text = spell.damage_type().display_name().to_string();
    }
    if let Ok(mut text) = desc_query.single_mut() {
        **text = spell.description().to_string();
    }
    if let Ok(mut text) = instr_query.single_mut() {
        **text = spell.instructions().to_string();
    }

    // Update hotkey box highlights — collect entity→color mapping first
    let mut hotkey_text_updates: Vec<(Entity, Color)> = Vec::new();
    for (entity, slot_btn, mut bg, mut border, mut colors) in &mut hotkey_query {
        let is_active = config.action_bar_slots[slot_btn.0 as usize] == Some(spell);
        let (new_bg, new_border, new_text_color) = if is_active {
            (HOTKEY_ACTIVE_BG, HOTKEY_ACTIVE_BORDER, HOTKEY_ACTIVE_TEXT)
        } else {
            (
                HOTKEY_INACTIVE_BG,
                HOTKEY_INACTIVE_BORDER,
                HOTKEY_INACTIVE_TEXT,
            )
        };
        bg.0 = new_bg;
        *border = BorderColor::all(new_border);
        colors.background = new_bg;
        colors.border = new_border;
        hotkey_text_updates.push((entity, new_text_color));
    }

    // Apply text color updates to hotkey button descendants
    for (btn_entity, new_text_color) in &hotkey_text_updates {
        if let Ok(children) = children_query.get(*btn_entity) {
            for child in children.iter() {
                if let Ok(mut tc) = hotkey_text_query.get_mut(child) {
                    tc.0 = *new_text_color;
                }
                if let Ok(grandchildren) = children_query.get(child) {
                    for gc in grandchildren.iter() {
                        if let Ok(mut tc) = hotkey_text_query.get_mut(gc) {
                            tc.0 = *new_text_color;
                        }
                    }
                }
            }
        }
    }

    // Update spell list borders
    for (list_btn, mut border) in &mut spell_list_query {
        let is_selected = list_btn.0 == spell;
        *border = BorderColor::all(if is_selected {
            SPELL_BUTTON_SELECTED_BORDER
        } else {
            SPELL_BUTTON_BORDER
        });
    }
}

/// Handles clicking a hotkey slot button to assign the selected spell.
pub(super) fn handle_hotkey_click(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    hotkey_query: Query<(Entity, &HotkeySlotButton)>,
    all_hotkey_buttons: Query<(Entity, &HotkeySlotButton), With<Button>>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in button_clicked.read() {
        let Ok((_, slot_btn)) = hotkey_query.get(event.button) else {
            continue;
        };
        let slot_idx = slot_btn.0 as usize;

        // Toggle: if already assigned to this slot, unassign; otherwise assign
        if config.action_bar_slots.get(slot_idx) == Some(&Some(selected.0)) {
            config.action_bar_slots[slot_idx] = None;
        } else {
            config.action_bar_slots[slot_idx] = Some(selected.0);
        }
        config_changed.write(crate::config::ConfigChanged);

        // Update ButtonActive on all hotkey buttons to reflect new state
        for (entity, btn) in &all_hotkey_buttons {
            let is_active = config.action_bar_slots[btn.0 as usize] == Some(selected.0);
            if is_active {
                commands.entity(entity).insert((
                    crate::ui::components::ButtonActive,
                    ButtonColors {
                        background: HOTKEY_ACTIVE_BG,
                        border: HOTKEY_ACTIVE_BORDER,
                    },
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
                commands.entity(entity).insert(ButtonColors {
                    background: HOTKEY_INACTIVE_BG,
                    border: HOTKEY_INACTIVE_BORDER,
                });
            }
        }
    }
}

/// Handles number key presses to assign/unassign the selected spell to an action bar slot.
pub(super) fn handle_number_key_assignment(
    mut commands: Commands,
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    all_hotkey_buttons: Query<(Entity, &HotkeySlotButton), With<Button>>,
    selected: Res<SelectedSpellPreview>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<crate::config::ConfigChanged>,
) {
    for event in action_bar_key.read() {
        if event.slot >= 5 {
            continue;
        }
        let slot_idx = event.slot as usize;

        // Toggle: if already assigned, unassign; otherwise assign
        if config.action_bar_slots.get(slot_idx) == Some(&Some(selected.0)) {
            config.action_bar_slots[slot_idx] = None;
        } else {
            config.action_bar_slots[slot_idx] = Some(selected.0);
        }
        config_changed.write(crate::config::ConfigChanged);

        // Update ButtonActive on all hotkey buttons
        for (entity, btn) in &all_hotkey_buttons {
            let is_active = config.action_bar_slots[btn.0 as usize] == Some(selected.0);
            if is_active {
                commands.entity(entity).insert((
                    crate::ui::components::ButtonActive,
                    ButtonColors {
                        background: HOTKEY_ACTIVE_BG,
                        border: HOTKEY_ACTIVE_BORDER,
                    },
                ));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
                commands.entity(entity).insert(ButtonColors {
                    background: HOTKEY_INACTIVE_BG,
                    border: HOTKEY_INACTIVE_BORDER,
                });
            }
        }
    }
}

/// Despawns spell book UI when exiting the SpellBook state.
pub(super) fn despawn_spell_book_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnSpellBookScreen>>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
    commands.remove_resource::<SelectedSpellPreview>();
}

/// Sets the flag when entering spell book to prevent spell casting.
pub(super) fn set_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = true;
}

/// Clears the flag after one frame in SpellBook state.
pub(super) fn clear_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = false;
}

/// Hides the concentration UI when the spell book opens.
pub(super) fn hide_concentration_ui(mut query: Query<&mut Visibility, With<ConcentrationUIRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Hidden;
    }
}

/// Shows the concentration UI when the spell book closes.
pub(super) fn show_concentration_ui(mut query: Query<&mut Visibility, With<ConcentrationUIRoot>>) {
    for mut vis in &mut query {
        *vis = Visibility::Inherited;
    }
}
