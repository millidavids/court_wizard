use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::spawn::{calculate_action_bar_font_size, effective_slot, slot_icon};
use crate::config::input_bindings::InputBindings;
use crate::config::{GameConfig, WizardType};
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::ui::color_utils::border_bright;
use crate::ui::components::{
    ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, GunIconAssets, SpellIconAssets,
};
use crate::ui::constants::{
    BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_PRESSED_OUTLINE,
};

/// Updates action bar slot text and icons when config changes.
/// For the gunslinger, highlights the currently selected gun slot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_action_bar_slots(
    config: Res<GameConfig>,
    // The icon size must match the radial-morph scale, or assigning/casting a spell
    // while a controller is active (radial mode) snaps the icon to full linear size
    // inside the shrunken button (the morph's early-out won't re-correct it).
    layout_progress: Res<ActionBarLayoutProgress>,
    icon_assets: Res<SpellIconAssets>,
    gun_icon_assets: Res<GunIconAssets>,
    mp_session: Option<Res<crate::networking::session::MultiplayerSession>>,
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
) {
    if config.is_changed() {
        let is_gunslinger = config.wizard_type == WizardType::Warglock;
        // Match the radial-morph scale so the icon size stays consistent whether a
        // controller (radial) or mouse/keyboard (linear) is active.
        let icon_px = SPELL_ICON_SIZE
            * (1.0 + (RADIAL_SLOT_SCALE - 1.0) * super::super::radial::ease(layout_progress.0));
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
                    node.width = Val::Px(icon_px);
                    node.height = Val::Px(icon_px);
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
            let spell = effective_slot(&config, slot_text.slot as usize, mp_session.as_deref());
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

        for (mut image_node, mut visibility, mut node, marker) in &mut slot_icon_query {
            if let Some(handle) = slot_icon(
                &config,
                marker.slot as usize,
                mp_session.as_deref(),
                &icon_assets,
            ) {
                *image_node = ImageNode::new(handle);
                *visibility = Visibility::Inherited;
                node.width = Val::Px(icon_px);
                node.height = Val::Px(icon_px);
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

/// Highlights action bar buttons when their corresponding keyboard key is held down.
/// Drives the 3D press animation and updates the front face border + edge outline.
pub(crate) fn highlight_keyboard_pressed_slots(
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

/// Resets every action-bar slot to its rest 3D look when the active input
/// device changes. A leftover keyboard-press highlight or mouse-hover raise
/// from before the switch would otherwise linger into the new device's mode —
/// e.g. a slot stuck "depressed" after switching from keyboard to controller,
/// or a button that reads half-pressed in the radial layout. The per-frame
/// highlight systems re-apply the correct state the same frame (a held key
/// re-presses, the hovered radial slot re-lights), so the active slot snaps
/// back immediately while every stale one returns to rest. Slots mid commit
/// flash are skipped so a just-cast confirmation isn't cut short.
pub(crate) fn reset_action_bar_on_device_change(
    mut commands: Commands,
    mut slots: Query<
        (
            Entity,
            &ButtonColors,
            &Children,
            Option<&mut ButtonAnimState>,
        ),
        (With<ActionBarSlot>, Without<RadialCommitFlash>),
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (entity, colors, children, anim) in &mut slots {
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
