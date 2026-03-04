use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::AssignSpellToSlot;
use crate::config::{ConfigChanged, GameConfig};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::ui::components::{ButtonColors, SpellIconAssets};
use crate::ui::systems::scale_font_by_text_width;

const DEBUG_BUTTON_SIZE: f32 = 30.0;
const DEBUG_BUTTON_GAP: f32 = 8.0;
const DEBUG_BUTTON_BG_OFF: Color = Color::srgba(0.2, 0.1, 0.1, 0.8);
const DEBUG_BUTTON_BG_ON: Color = Color::srgba(0.1, 0.5, 0.1, 0.9);
const DEBUG_BUTTON_BORDER: Color = Color::srgba(0.6, 0.3, 0.3, 1.0);

/// Calculates the appropriate font size for action bar spell names based on max line width.
fn calculate_action_bar_font_size(name: &str) -> f32 {
    let max_line_width = name.lines().map(|line| line.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_line_width, 6.0, 11.0, 0.65, SPELL_NAME_FONT_SIZE)
}

/// Spawns the action bar UI at the bottom-left of the screen.
pub(super) fn spawn_action_bar(
    mut commands: Commands,
    config: Res<GameConfig>,
    icon_assets: Res<SpellIconAssets>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(ACTION_BAR_BOTTOM_MARGIN),
                left: Val::Px(ACTION_BAR_LEFT_MARGIN),
                ..default()
            },
            ActionBarRoot,
            OnGameplayScreen,
        ))
        .with_children(|wrapper| {
            // Inner container with the actual slot buttons
            wrapper
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(SLOT_GAP),
                    align_items: AlignItems::End,
                    ..default()
                })
                .with_children(|parent| {
                    for slot in 0..5 {
                        let hotkey_label = &(slot + 1).to_string();
                        let spell = config.action_bar_slots[slot as usize];

                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(SLOT_BUTTON_STYLE.width),
                                    height: Val::Px(SLOT_BUTTON_STYLE.height),
                                    border: UiRect::all(Val::Px(SLOT_BUTTON_STYLE.border_width)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderColor::all(SLOT_BUTTON_STYLE.border),
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(SLOT_BUTTON_STYLE.background),
                                ButtonColors {
                                    background: SLOT_BUTTON_STYLE.background,
                                    border: SLOT_BUTTON_STYLE.border,
                                },
                                ActionBarSlot { slot },
                            ))
                            .with_children(|button| {
                                // Hotkey indicator at top
                                button.spawn((
                                    Text::new(hotkey_label),
                                    TextFont::from_font_size(HOTKEY_FONT_SIZE),
                                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                    ActionBarHotkeyText,
                                ));

                                // Spell icon or name in center
                                let icon_handle = spell
                                    .and_then(|s| icon_assets.get(&s).cloned());
                                let has_icon = icon_handle.is_some();
                                if let Some(handle) = icon_handle {
                                    button.spawn((
                                        ImageNode::new(handle),
                                        Node {
                                            width: Val::Px(SPELL_ICON_SIZE),
                                            height: Val::Px(SPELL_ICON_SIZE),
                                            flex_grow: 1.0,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        ActionBarSlotIcon { slot },
                                    ));
                                }

                                let spell_name = spell.map(|s| s.name()).unwrap_or("");
                                let font_size = calculate_action_bar_font_size(spell_name);
                                button
                                    .spawn((
                                        Text::new(spell_name),
                                        TextFont::from_font_size(font_size),
                                        TextColor(SLOT_BUTTON_STYLE.text_color),
                                        TextLayout::new_with_justify(Justify::Center),
                                        ActionBarSlotText { slot },
                                        if has_icon {
                                            Visibility::Hidden
                                        } else {
                                            Visibility::Inherited
                                        },
                                    ))
                                    .insert(Node {
                                        flex_grow: if has_icon { 0.0 } else { 1.0 },
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    });
                            });
                    }

                    // Debug: infinite mana toggle button
                    parent
                        .spawn(Node {
                            width: Val::Px(DEBUG_BUTTON_GAP),
                            ..default()
                        });
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(DEBUG_BUTTON_SIZE),
                                height: Val::Px(DEBUG_BUTTON_SIZE),
                                border: UiRect::all(Val::Px(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(DEBUG_BUTTON_BORDER),
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(DEBUG_BUTTON_BG_OFF),
                            ButtonColors {
                                background: DEBUG_BUTTON_BG_OFF,
                                border: DEBUG_BUTTON_BORDER,
                            },
                            DebugManaButton,
                        ))
                        .with_child((
                            Text::new("INF"),
                            TextFont::from_font_size(7.0),
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                        ));
                });
        });
}

/// Handles action bar slot clicks by priming the assigned spell.
pub(super) fn handle_slot_click(
    mut button_clicked: MessageReader<MouseClicked>,
    slot_query: Query<&ActionBarSlot>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(slot) = slot_query.get(event.button)
            && let Some(spell) = config.action_bar_slots[slot.slot as usize]
        {
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config(),
            });
        }
    }
}

/// Handles keyboard input for action bar slots by priming the assigned spell.
pub(super) fn handle_keyboard_input(
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
) {
    for event in action_bar_key.read() {
        if let Some(spell) = config.action_bar_slots[event.slot as usize] {
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config(),
            });
        }
    }
}

/// Updates action bar slot text and icons when config changes.
pub(super) fn update_action_bar_slots(
    config: Res<GameConfig>,
    icon_assets: Res<SpellIconAssets>,
    mut slot_text_query: Query<(
        &mut Text,
        &mut TextFont,
        &mut Visibility,
        &mut Node,
        &ActionBarSlotText,
    )>,
    mut slot_icon_query: Query<(&mut ImageNode, &mut Visibility, &mut Node, &ActionBarSlotIcon), Without<ActionBarSlotText>>,
) {
    if config.is_changed() {
        for (mut text, mut text_font, mut visibility, mut node, slot_text) in &mut slot_text_query {
            let spell = config.action_bar_slots[slot_text.slot as usize];
            let spell_name = spell.map(|s| s.name()).unwrap_or("");
            **text = spell_name.to_string();
            text_font.font_size = calculate_action_bar_font_size(spell_name);

            let has_icon = spell.is_some_and(|s| icon_assets.get(&s).is_some());
            *visibility = if has_icon {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            node.flex_grow = if has_icon { 0.0 } else { 1.0 };
        }

        for (mut image_node, mut visibility, mut node, slot_icon) in &mut slot_icon_query {
            let spell = config.action_bar_slots[slot_icon.slot as usize];
            if let Some(handle) = spell.and_then(|s| icon_assets.get(&s).cloned()) {
                *image_node = ImageNode::new(handle);
                *visibility = Visibility::Inherited;
                node.width = Val::Px(SPELL_ICON_SIZE);
                node.height = Val::Px(SPELL_ICON_SIZE);
                node.flex_grow = 1.0;
            } else {
                *visibility = Visibility::Hidden;
                node.flex_grow = 0.0;
                node.width = Val::Px(0.0);
                node.height = Val::Px(0.0);
            }
        }
    }
}

/// Handles clicks on the debug infinite mana button.
pub(super) fn handle_debug_mana_click(
    mut button_clicked: MessageReader<MouseClicked>,
    debug_button_query: Query<Entity, With<DebugManaButton>>,
    mut infinite_mana: ResMut<InfiniteMana>,
    mut bg_query: Query<(&mut BackgroundColor, &mut ButtonColors), With<DebugManaButton>>,
) {
    for event in button_clicked.read() {
        if debug_button_query.get(event.button).is_ok() {
            infinite_mana.0 = !infinite_mana.0;
            let new_bg = if infinite_mana.0 {
                DEBUG_BUTTON_BG_ON
            } else {
                DEBUG_BUTTON_BG_OFF
            };
            for (mut bg, mut colors) in bg_query.iter_mut() {
                bg.0 = new_bg;
                colors.background = new_bg;
            }
        }
    }
}

/// Handles spell assignment to action bar slots.
pub(super) fn handle_spell_assignment(
    mut assign_spell: MessageReader<AssignSpellToSlot>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    for event in assign_spell.read() {
        let slot_idx = event.slot as usize;
        if let Some(slot) = config.action_bar_slots.get_mut(slot_idx) {
            *slot = Some(event.spell);
            config_changed.write(ConfigChanged);
        }
    }
}
