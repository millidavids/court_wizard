use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::AssignSpellToSlot;
use super::radial::{ease, linear_pos, radial_pos};
use crate::config::input_bindings::{InputBindings, key_display_name};
use crate::config::{ConfigChanged, GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::archetypes::gunslinger::GunState;
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::game::units::wizard::archetypes::gunslinger::messages::SelectGunMessage;
use crate::game::units::wizard::messages::PrimeSpellMessage;
use crate::ui::color_utils::border_bright;
use crate::ui::components::{
    ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, GunIconAssets, SpellIconAssets,
};
use crate::ui::constants::{
    BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_PRESSED_OUTLINE,
};
use crate::ui::systems::scale_font_by_text_width;

#[cfg(debug_assertions)]
const DEBUG_BUTTON_SIZE: f32 = 30.0;
#[cfg(debug_assertions)]
const DEBUG_BUTTON_GAP: f32 = 8.0;
#[cfg(debug_assertions)]
const DEBUG_BUTTON_BG_OFF: Color = Color::srgba(0.2, 0.1, 0.1, 0.8);
#[cfg(debug_assertions)]
const DEBUG_BUTTON_BG_ON: Color = Color::srgba(0.1, 0.5, 0.1, 0.9);
#[cfg(debug_assertions)]
const DEBUG_BUTTON_BORDER: Color = Color::srgba(0.6, 0.3, 0.3, 1.0);

/// Calculates the appropriate font size for action bar spell names based on max line width.
fn calculate_action_bar_font_size(name: &str) -> f32 {
    let max_line_width = name.lines().map(|line| line.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_line_width, 6.0, 11.0, 0.65, SPELL_NAME_FONT_SIZE)
}

/// Spawns the action bar UI at the bottom-left of the screen.
/// Clears action bar spells that are blocked by the current wizard type.
/// Runs once when entering gameplay, before the action bar is spawned.
pub(super) fn clear_blocked_action_bar_spells(mut config: ResMut<GameConfig>) {
    if config.wizard_type == WizardType::Shepherd {
        for slot in &mut config.action_bar_slots {
            if let Some(spell) = slot
                && !spell.is_shepherd_allowed()
            {
                *slot = None;
            }
        }
    }
}

pub(super) fn spawn_action_bar(
    mut commands: Commands,
    config: Res<GameConfig>,
    bindings: Res<InputBindings>,
    icon_assets: Res<SpellIconAssets>,
    gun_icon_assets: Res<GunIconAssets>,
    layout_progress: Res<ActionBarLayoutProgress>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(240.0),
                ..default()
            },
            ActionBarRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            {
                let is_gunslinger = config.wizard_type == WizardType::Warglock;
                let guns = GunType::all();

                let slot_bindings: [Option<KeyCode>; 5] = [
                    bindings.universal.action_slot_1,
                    bindings.universal.action_slot_2,
                    bindings.universal.action_slot_3,
                    bindings.universal.action_slot_4,
                    bindings.universal.action_slot_5,
                ];

                for slot in 0..5 {
                    let hotkey_label = &key_display_name(slot_bindings[slot as usize]).to_string();

                    // For gunslinger, render gun icons (no name fallback);
                    // every gun has a dedicated icon now.
                    let (slot_name, icon_handle): (&str, Option<Handle<Image>>) = if is_gunslinger {
                        let gun = guns[slot as usize];
                        ("", gun_icon_assets.get(&gun).cloned())
                    } else {
                        let spell = config.action_bar_slots[slot as usize];
                        let icon = spell.and_then(|s| icon_assets.get(&s).cloned());
                        let name = spell.map(|s| s.name()).unwrap_or("");
                        (name, icon)
                    };

                    // Compute initial position from the already-settled
                    // layout progress (set by `reset_layout_progress` on
                    // gameplay entry) so the slots render in their final
                    // layout — linear on KB+M, radial on controller —
                    // from the very first frame. Avoids the visible
                    // linear→radial animation every time a controller
                    // user starts a run.
                    let t = ease(layout_progress.0);
                    let init_pos = linear_pos(slot).lerp(radial_pos(slot), t);
                    let init_scale = 1.0 + (RADIAL_SLOT_SCALE - 1.0) * t;
                    let init_w = SLOT_BUTTON_STYLE.width * init_scale;
                    let init_h = SLOT_BUTTON_STYLE.height * init_scale;
                    let init_border = SLOT_BUTTON_STYLE.border_width * init_scale;
                    let init_padding = 2.0 * init_scale;
                    let bg_color = if is_gunslinger {
                        WARGLOCK_SLOT_BACKGROUND
                    } else {
                        SLOT_BUTTON_STYLE.background
                    };
                    parent
                        .spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(init_pos.x),
                                bottom: Val::Px(init_pos.y),
                                width: Val::Px(init_w),
                                height: Val::Px(init_h),
                                min_width: Val::Px(0.0),
                                min_height: Val::Px(0.0),
                                border: UiRect::all(Val::Px(init_border)),
                                flex_direction: FlexDirection::Column,
                                justify_content: if layout_progress.0 > 0.5 {
                                    JustifyContent::Center
                                } else {
                                    JustifyContent::SpaceBetween
                                },
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(init_padding)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BorderColor::all(SLOT_BUTTON_STYLE.border),
                            BackgroundColor(bg_color),
                            ButtonColors {
                                background: bg_color,
                                border: SLOT_BUTTON_STYLE.border,
                            },
                            ActionBarSlot { slot },
                        ))
                        .with_children(|button| {
                            // Hidden at spawn when the radial layout is
                            // already settled — `animate_layout_morph`'s
                            // `last_applied` early-out won't re-hide on
                            // slot respawn if `progress.0` hasn't changed.
                            button.spawn((
                                Text::new(hotkey_label),
                                TextFont::from_font_size(HOTKEY_FONT_SIZE),
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                Node {
                                    display: if layout_progress.0 > 0.5 {
                                        Display::None
                                    } else {
                                        Display::Flex
                                    },
                                    ..default()
                                },
                                ActionBarHotkeyText,
                            ));

                            // Spell icon or name in center. Scale the
                            // icon by the current layout progress so a
                            // controller-first spawn renders its icons
                            // already at radial size (no brief oversized
                            // flash before the animate system catches up).
                            if let Some(handle) = icon_handle {
                                let icon_px = SPELL_ICON_SIZE * init_scale;
                                button.spawn((
                                    ImageNode::new(handle),
                                    Node {
                                        width: Val::Px(icon_px),
                                        height: Val::Px(icon_px),
                                        flex_grow: 1.0,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ActionBarSlotIcon { slot },
                                ));
                            }

                            // Spell names have been removed from the
                            // action bar — icons are the identity, and
                            // long names like "Crescent Strike" /
                            // "Forged in Fire" overflow the 50x50
                            // button. The hotkey text above the icon is
                            // all that remains.
                            let _ = slot_name;
                        });
                }

                // Debug: infinite mana toggle — sits at the end of the
                // linear row, hidden while the gamepad radial is active.
                #[cfg(debug_assertions)]
                {
                    let inf_left = ACTION_BAR_LEFT_MARGIN
                        + 5.0 * (SLOT_BUTTON_STYLE.width + SLOT_GAP)
                        + DEBUG_BUTTON_GAP;
                    let inf_bottom = ACTION_BAR_BOTTOM_MARGIN
                        + (SLOT_BUTTON_STYLE.height - DEBUG_BUTTON_SIZE) / 2.0;
                    parent
                        .spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(inf_left),
                                bottom: Val::Px(inf_bottom),
                                width: Val::Px(DEBUG_BUTTON_SIZE),
                                height: Val::Px(DEBUG_BUTTON_SIZE),
                                border: UiRect::all(Val::Px(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BorderColor::all(DEBUG_BUTTON_BORDER),
                            BackgroundColor(DEBUG_BUTTON_BG_OFF),
                            ButtonColors {
                                background: DEBUG_BUTTON_BG_OFF,
                                border: DEBUG_BUTTON_BORDER,
                            },
                            DebugManaButton,
                            Visibility::Hidden,
                        ))
                        .with_child((
                            Text::new("INF"),
                            TextFont::from_font_size(7.0),
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                        ));
                }
            }
        });
}

/// Handles action bar slot clicks by priming the assigned spell (or selecting a gun for gunslinger).
pub(super) fn handle_slot_click(
    mut button_clicked: MessageReader<MouseClicked>,
    slot_query: Query<&ActionBarSlot>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut select_gun: MessageWriter<SelectGunMessage>,
) {
    let is_gunslinger = config.wizard_type == WizardType::Warglock;
    let guns = GunType::all();

    for event in button_clicked.read() {
        if let Ok(slot) = slot_query.get(event.button) {
            let slot_idx = slot.slot as usize;
            if is_gunslinger {
                if slot_idx < 5 {
                    select_gun.write(SelectGunMessage {
                        gun: guns[slot_idx],
                    });
                }
            } else if config.wizard_type.uses_exclusive_casting() {
                // RuneCaster/Randomancer can only cast via their own mechanics
            } else if let Some(spell) = config.action_bar_slots[slot_idx] {
                prime_spell.write(PrimeSpellMessage {
                    spell: spell.primed_config(),
                });
            }
        }
    }
}

/// Handles keyboard input for action bar slots by priming the assigned spell (or selecting a gun).
pub(super) fn handle_keyboard_input(
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut select_gun: MessageWriter<SelectGunMessage>,
) {
    let is_gunslinger = config.wizard_type == WizardType::Warglock;
    let guns = GunType::all();

    for event in action_bar_key.read() {
        let slot_idx = event.slot as usize;
        if is_gunslinger {
            if slot_idx < 5 {
                select_gun.write(SelectGunMessage {
                    gun: guns[slot_idx],
                });
            }
        } else if matches!(
            config.wizard_type,
            WizardType::RuneCaster | WizardType::Randomancer
        ) {
            // RuneCaster/Randomancer can only cast via their own mechanics
        } else if let Some(spell) = config.action_bar_slots[slot_idx] {
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config(),
            });
        }
    }
}

/// Updates action bar slot text and icons when config changes.
/// For the gunslinger, highlights the currently selected gun slot.
pub(super) fn update_action_bar_slots(
    config: Res<GameConfig>,
    icon_assets: Res<SpellIconAssets>,
    gun_icon_assets: Res<GunIconAssets>,
    gun_state: Option<Res<GunState>>,
    mut slot_text_query: Query<(
        &mut Text,
        &mut TextFont,
        &mut Visibility,
        &mut Node,
        &ActionBarSlotText,
    )>,
    mut slot_icon_query: Query<
        (
            &mut ImageNode,
            &mut Visibility,
            &mut Node,
            &ActionBarSlotIcon,
        ),
        Without<ActionBarSlotText>,
    >,
    mut slot_button_query: Query<(&ActionBarSlot, &mut BorderColor, &ButtonColors)>,
) {
    let is_gunslinger = config.wizard_type == WizardType::Warglock;

    // Update slot highlighting for gunslinger — only when the gun
    // selection changes, so the per-frame write doesn't stomp on the radial
    // hover color written by `highlight_radial_hovered_slot`.
    if is_gunslinger
        && let Some(ref gs) = gun_state
        && gs.is_changed()
    {
        let guns = GunType::all();
        for (slot, mut border_color, colors) in &mut slot_button_query {
            let slot_idx = slot.slot as usize;
            if slot_idx < 5 && guns[slot_idx] == gs.selected_gun {
                *border_color = BorderColor::all(Color::srgb(1.0, 0.8, 0.2));
            } else {
                *border_color = BorderColor::all(colors.border);
            }
        }
    }

    if config.is_changed() {
        if is_gunslinger {
            // Show gun icons in slots, hide name text.
            let guns = GunType::all();
            for (mut text, _text_font, mut visibility, mut node, _slot_text) in &mut slot_text_query
            {
                **text = String::new();
                *visibility = Visibility::Inherited;
                node.display = Display::None;
                node.flex_grow = 0.0;
            }
            for (mut image_node, mut visibility, mut node, slot_icon) in &mut slot_icon_query {
                let slot_idx = slot_icon.slot as usize;
                let handle = guns
                    .get(slot_idx)
                    .and_then(|gun| gun_icon_assets.get(gun).cloned());
                if let Some(handle) = handle {
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
            return;
        }

        for (mut text, mut text_font, mut visibility, mut node, slot_text) in &mut slot_text_query {
            let spell = config.action_bar_slots[slot_text.slot as usize];
            let spell_name = spell.map(|s| s.name()).unwrap_or("");
            **text = spell_name.to_string();
            text_font.font_size = calculate_action_bar_font_size(spell_name);

            let has_icon = spell.is_some_and(|s| icon_assets.get(&s).is_some());
            // Use Display::None (not Visibility::Hidden) for hidden text —
            // otherwise the text still reserves layout space and pushes the
            // icon/hotkey stack past the button's fixed height, which shows
            // up as the edge/front-face layers disagreeing about size.
            *visibility = Visibility::Inherited;
            if has_icon {
                node.display = Display::None;
                node.flex_grow = 0.0;
            } else {
                node.display = Display::Flex;
                node.flex_grow = 1.0;
            }
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
#[cfg(debug_assertions)]
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

/// Highlights action bar buttons when their corresponding keyboard key is held down.
/// Drives the 3D press animation and updates the front face border + edge outline.
pub(super) fn highlight_keyboard_pressed_slots(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut slots: Query<(
        Entity,
        &ActionBarSlot,
        &ButtonColors,
        &Children,
        Option<&mut ButtonAnimState>,
        Has<KeyboardHighlighted>,
    )>,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    let keys: [Option<KeyCode>; 5] = [
        bindings.universal.action_slot_1,
        bindings.universal.action_slot_2,
        bindings.universal.action_slot_3,
        bindings.universal.action_slot_4,
        bindings.universal.action_slot_5,
    ];

    for (entity, slot, colors, children, anim, is_highlighted) in &mut slots {
        let slot_idx = slot.slot as usize;
        if slot_idx >= keys.len() {
            continue;
        }

        let pressed = keys[slot_idx].is_some_and(|key| keyboard.pressed(key));

        if pressed && !is_highlighted {
            commands.entity(entity).insert(KeyboardHighlighted);
            if let Some(mut anim) = anim {
                anim.target = BUTTON_3D_OFFSET_PRESSED;
            }
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(border_bright(colors.border));
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_PRESSED_OUTLINE;
                }
            }
        } else if !pressed && is_highlighted {
            commands.entity(entity).remove::<KeyboardHighlighted>();
            if let Some(mut anim) = anim {
                anim.target = BUTTON_3D_OFFSET_REST;
            }
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(colors.border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = crate::ui::constants::BUTTON_REST_OUTLINE;
                }
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
        // Shepherd cannot assign damage-dealing spells
        if config.wizard_type == WizardType::Shepherd && !event.spell.is_shepherd_allowed() {
            continue;
        }
        let slot_idx = event.slot as usize;
        if let Some(slot) = config.action_bar_slots.get_mut(slot_idx) {
            *slot = Some(event.spell);
            config_changed.write(ConfigChanged);
        }
    }
}
