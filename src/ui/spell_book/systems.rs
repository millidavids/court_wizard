use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::config::save_data::load_unified_save;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::components::{Spell, SpellCategory};
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::networking::session::MultiplayerSession;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::action_bar::messages::AssignSpellToSlot;
use crate::ui::components::ButtonColors;
use crate::ui::concentration::ConcentrationUIRoot;
use crate::ui::systems::spawn_button;

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
    asset_server: Res<AssetServer>,
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

    let is_unlocked = move |spell: &Spell| {
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

    // Root container (full screen, two-column row layout)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(LAYOUT_PADDING)),
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnSpellBookScreen,
        ))
        .with_children(|root| {
            // === Left panel: detail + buttons ===
            spawn_detail_panel(root, initial_spell, &config);

            // === Right panel: categorized spell list ===
            spawn_spell_list(root, initial_spell, &is_unlocked, &asset_server);
        });
}

/// Spawns the left detail panel showing spell info, hotkeys, and action buttons.
fn spawn_detail_panel(parent: &mut ChildSpawnerCommands, spell: Spell, config: &GameConfig) {
    parent
        .spawn(Node {
            width: Val::Px(LEFT_PANEL_WIDTH),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            row_gap: Val::Px(16.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|left| {
            // Detail panel with border
            left.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(DETAIL_PADDING)),
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(DETAIL_BORDER_WIDTH)),
                    ..default()
                },
                BackgroundColor(DETAIL_BG),
                BorderColor::all(DETAIL_BORDER),
                BorderRadius::all(Val::Px(DETAIL_BORDER_RADIUS)),
            ))
            .with_children(|panel| {
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
                                    let is_active =
                                        config.action_bar_slots[slot as usize] == Some(spell);
                                    let (bg, border, text_color) = if is_active {
                                        (HOTKEY_ACTIVE_BG, HOTKEY_ACTIVE_BORDER, HOTKEY_ACTIVE_TEXT)
                                    } else {
                                        (
                                            HOTKEY_INACTIVE_BG,
                                            HOTKEY_INACTIVE_BORDER,
                                            HOTKEY_INACTIVE_TEXT,
                                        )
                                    };

                                    row.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(HOTKEY_BOX_SIZE),
                                            height: Val::Px(HOTKEY_BOX_SIZE),
                                            border: UiRect::all(Val::Px(1.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderColor::all(border),
                                        BorderRadius::all(Val::Px(4.0)),
                                        ButtonColors {
                                            background: bg,
                                            border,
                                        },
                                        HotkeySlotButton(slot),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(format!("{}", slot + 1)),
                                            TextFont::from_font_size(HOTKEY_FONT_SIZE),
                                            TextColor(text_color),
                                        ));
                                    });
                                }
                            });
                    });
            });

            // Button row: Select + Close
            left.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|buttons| {
                spawn_button(
                    buttons,
                    "Select",
                    SpellBookButtonAction::CastSpell,
                    &SELECT_BUTTON_STYLE,
                );
                spawn_button(
                    buttons,
                    "Close",
                    SpellBookButtonAction::Close,
                    &CLOSE_BUTTON_STYLE,
                );
            });
        });
}

/// Spawns the right panel with a vertically scrollable list of spells grouped by category.
fn spawn_spell_list(
    parent: &mut ChildSpawnerCommands,
    selected: Spell,
    is_unlocked: &dyn Fn(&Spell) -> bool,
    asset_server: &AssetServer,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                border: UiRect::all(Val::Px(LIST_BORDER_WIDTH)),
                padding: UiRect::all(Val::Px(LIST_PADDING)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            BorderRadius::all(Val::Px(LIST_BORDER_RADIUS)),
            ScrollPosition::default(),
            ScrollableSpellList,
        ))
        .with_children(|list| {
            // Inner content column (needed for scroll to work with overflow)
            list.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(CATEGORY_GAP),
                ..default()
            })
            .with_children(|content| {
                for category in SpellCategory::all() {
                    let unlocked_in_category: Vec<&Spell> = category
                        .spells()
                        .iter()
                        .filter(|s| is_unlocked(s))
                        .collect();

                    if unlocked_in_category.is_empty() {
                        continue;
                    }

                    // Category section
                    content
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(LIST_ITEM_GAP),
                            ..default()
                        })
                        .with_children(|section| {
                            // Category header (full width)
                            section.spawn((
                                Text::new(format!("-- {} --", category.display_name())),
                                TextFont::from_font_size(CATEGORY_FONT_SIZE),
                                TextColor(CATEGORY_COLOR),
                                TextLayout::new_with_justify(Justify::Center),
                                Node {
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                            ));

                            // Spell buttons in a wrapping row grid
                            section
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::Wrap,
                                    column_gap: Val::Px(LIST_ITEM_GAP),
                                    row_gap: Val::Px(LIST_ITEM_GAP),
                                    ..default()
                                })
                                .with_children(|grid| {
                                    for spell in &unlocked_in_category {
                                        let is_selected = **spell == selected;
                                        let border = if is_selected {
                                            SPELL_BUTTON_SELECTED_BORDER
                                        } else {
                                            SPELL_BUTTON_BORDER
                                        };

                                        grid.spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(SPELL_BUTTON_WIDTH),
                                                height: Val::Px(SPELL_BUTTON_HEIGHT),
                                                border: UiRect::all(Val::Px(
                                                    SPELL_BUTTON_BORDER_WIDTH,
                                                )),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                flex_direction: FlexDirection::Row,
                                                column_gap: Val::Px(6.0),
                                                ..default()
                                            },
                                            BackgroundColor(SPELL_BUTTON_BG),
                                            BorderColor::all(border),
                                            BorderRadius::all(Val::Px(4.0)),
                                            ButtonColors {
                                                background: SPELL_BUTTON_BG,
                                                border: SPELL_BUTTON_BORDER,
                                            },
                                            SpellBookButtonAction::SelectSpell(**spell),
                                            SpellListButton(**spell),
                                        ))
                                        .with_children(
                                            |btn| {
                                                if let Some(icon_path) = spell.icon_path() {
                                                    btn.spawn((
                                                        ImageNode::new(
                                                            asset_server.load(icon_path),
                                                        ),
                                                        Node {
                                                            width: Val::Px(
                                                                SPELL_ICON_SIZE,
                                                            ),
                                                            height: Val::Px(
                                                                SPELL_ICON_SIZE,
                                                            ),
                                                            ..default()
                                                        },
                                                    ));
                                                }
                                                btn.spawn((
                                                    Text::new(spell.display_name()),
                                                    TextFont::from_font_size(
                                                        SPELL_BUTTON_FONT_SIZE,
                                                    ),
                                                    TextColor(SPELL_BUTTON_TEXT_COLOR),
                                                ));
                                            },
                                        );
                                    }
                                });
                        });
                }
            });
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
                SpellBookButtonAction::SelectSpell(spell) => {
                    selected.0 = *spell;
                }
                SpellBookButtonAction::CastSpell => {
                    prime_spell.write(PrimeSpellMessage {
                        spell: selected.0.primed_config(),
                    });
                    if let Some(ref mut next_sp) = next_in_game_state {
                        next_sp.set(InGameState::Running);
                    }
                    if let Some(ref mut next_mp) = next_mp_state {
                        next_mp.set(MultiplayerGameState::Running);
                    }
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
    mut hotkey_text_query: Query<(&ChildOf, &mut TextColor)>,
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

    // Apply text color updates to hotkey button children
    for (btn_entity, new_text_color) in &hotkey_text_updates {
        for (child_of, mut text_color) in &mut hotkey_text_query {
            if child_of.get() == *btn_entity {
                text_color.0 = *new_text_color;
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
    mut button_clicked: MessageReader<MouseClicked>,
    hotkey_query: Query<&HotkeySlotButton>,
    selected: Res<SelectedSpellPreview>,
    mut assign_spell: MessageWriter<AssignSpellToSlot>,
) {
    for event in button_clicked.read() {
        if let Ok(slot_btn) = hotkey_query.get(event.button) {
            assign_spell.write(AssignSpellToSlot {
                slot: slot_btn.0,
                spell: selected.0,
            });
        }
    }
}

/// Handles number key presses to assign the selected spell to an action bar slot.
pub(super) fn handle_number_key_assignment(
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    selected: Res<SelectedSpellPreview>,
    mut assign_spell: MessageWriter<AssignSpellToSlot>,
) {
    for event in action_bar_key.read() {
        if event.slot < 5 {
            assign_spell.write(AssignSpellToSlot {
                slot: event.slot,
                spell: selected.0,
            });
        }
    }
}

/// Handles mouse wheel scrolling for the spell list.
pub(super) fn handle_spell_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableSpellList>>,
    parent_query: Query<&ChildOf>,
) {
    const LINE_HEIGHT: f32 = 30.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        break;
                    }

                    match parent_query.get(current_entity) {
                        Ok(parent) => current_entity = parent.get(),
                        Err(_) => break,
                    }
                }
            }
        }
    }
}

/// Handles keyboard input (ESC to close).
pub(super) fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::Running);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::Running);
        }
    }
}

/// Despawns spell book UI when exiting the SpellBook state.
pub(super) fn despawn_spell_book_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnSpellBookScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SelectedSpellPreview>();
}

/// Consumes the mouse button when exiting spell book to prevent click bleed-through.
pub(super) fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
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
