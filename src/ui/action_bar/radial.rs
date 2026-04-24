//! Layout-morphing logic for the action bar.
//!
//! A single set of `ActionBarSlot` button entities is spawned by
//! `spawn_action_bar`. This module is responsible for (1) ticking the
//! linear ↔ radial transition each frame based on `ActiveInputDevice`,
//! (2) highlighting the slot the right stick is pointing at while the
//! radial layout is active, and (3) pulsing a slot briefly when the
//! player commits it via right-stick + RT.

use bevy::prelude::*;
use std::f32::consts::TAU;

use super::components::*;
use super::constants::*;
use crate::game::input::gamepad::constants::RADIAL_SLOT_COUNT;
use crate::game::input::gamepad::resources::{ActiveInputDevice, RadialHoveredSlot};
use crate::game::input::messages::ActionBarKeyPressed;
use crate::ui::components::ButtonColors;

/// Linear target position (screen-space `left`, `bottom`) of slot `i` — the
/// row in the bottom-left corner that matches the mouse/keyboard layout.
pub(super) fn linear_pos(i: u8) -> Vec2 {
    let left =
        ACTION_BAR_LEFT_MARGIN + i as f32 * (SLOT_BUTTON_STYLE.width + SLOT_GAP);
    Vec2::new(left, ACTION_BAR_BOTTOM_MARGIN)
}

/// Radial target position of slot `i`, placed at the corresponding 72° wedge
/// on a ring centered at `(RADIAL_CENTER_LEFT, RADIAL_CENTER_BOTTOM)`. Slot 0
/// sits at the top (12 o'clock); subsequent slots go clockwise. Positions
/// account for the shrunken radial button size so each slot's center lands
/// exactly on the ring.
pub(super) fn radial_pos(i: u8) -> Vec2 {
    let angle = i as f32 * TAU / RADIAL_SLOT_COUNT as f32;
    let dx = angle.sin() * RADIAL_RING_RADIUS;
    let dy = angle.cos() * RADIAL_RING_RADIUS;
    let w = SLOT_BUTTON_STYLE.width * RADIAL_SLOT_SCALE;
    let h = SLOT_BUTTON_STYLE.height * RADIAL_SLOT_SCALE;
    Vec2::new(
        RADIAL_CENTER_LEFT + dx - w / 2.0,
        RADIAL_CENTER_BOTTOM + dy - h / 2.0,
    )
}

/// Smoothstep easing for a nicer morph — the buttons accelerate out of the
/// row, cruise, and settle softly into the ring (and vice versa).
pub(super) fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Ticks the layout progress toward its target (driven by
/// `ActiveInputDevice`) and repositions every slot with the resulting lerp.
/// Also hides the debug INF button once the morph has mostly finished, since
/// it has no radial equivalent.
#[allow(clippy::too_many_arguments)]
pub(super) fn animate_action_bar_layout(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    mut progress: ResMut<ActionBarLayoutProgress>,
    mut last_applied: Local<Option<f32>>,
    mut slots: Query<(Entity, &ActionBarSlot, &mut Node), Without<DebugManaButton>>,
    mut icons: Query<
        (&mut Node, &mut ImageNode),
        (With<ActionBarSlotIcon>, Without<ActionBarSlot>),
    >,
    mut name_texts: Query<
        (&mut TextFont, &mut Node),
        (
            With<ActionBarSlotText>,
            Without<ActionBarSlot>,
            Without<ActionBarSlotIcon>,
            Without<ActionBarHotkeyText>,
        ),
    >,
    mut hotkey_texts: Query<
        (&mut TextFont, &mut Node),
        (
            With<ActionBarHotkeyText>,
            Without<ActionBarSlotText>,
            Without<ActionBarSlotIcon>,
            Without<ActionBarSlot>,
        ),
    >,
    mut inf: Query<&mut Visibility, With<DebugManaButton>>,
    children_query: Query<&Children>,
    mut fronts: Query<
        &mut Node,
        (
            With<crate::ui::components::ButtonFront>,
            Without<ActionBarSlot>,
            Without<ActionBarSlotIcon>,
            Without<ActionBarSlotText>,
            Without<ActionBarHotkeyText>,
            Without<DebugManaButton>,
        ),
    >,
) {
    let target = if active.is_gamepad() { 1.0 } else { 0.0 };
    let step = time.delta_secs() / RADIAL_TRANSITION_SECS;
    if (progress.0 - target).abs() <= step {
        progress.0 = target;
    } else {
        progress.0 += (target - progress.0).signum() * step;
    }
    // Skip applying the layout if nothing has changed since last tick —
    // keeps the system from marking every Node as Changed every frame and
    // triggering a full UI re-layout + visible flicker. We still always
    // apply on first run and on every in-flight frame of the morph.
    if last_applied.map(|v| (v - progress.0).abs() < f32::EPSILON).unwrap_or(false) {
        return;
    }
    *last_applied = Some(progress.0);
    let t = ease(progress.0);
    let scale = 1.0 + (RADIAL_SLOT_SCALE - 1.0) * t;

    let slot_w = SLOT_BUTTON_STYLE.width * scale;
    let slot_h = SLOT_BUTTON_STYLE.height * scale;
    let border_px = SLOT_BUTTON_STYLE.border_width * scale;
    let padding_px = 2.0 * scale;
    let mut slot_entities: Vec<Entity> = Vec::new();
    for (entity, slot, mut node) in &mut slots {
        slot_entities.push(entity);
        let from = linear_pos(slot.slot);
        let to = radial_pos(slot.slot);
        let pos = from.lerp(to, t);
        node.left = Val::Px(pos.x);
        node.bottom = Val::Px(pos.y);
        node.width = Val::Px(slot_w);
        node.height = Val::Px(slot_h);
        // Pin min to 0 so width/height stays authoritative. Content fits
        // because hidden text nodes use Display::None (see
        // `update_action_bar_slots`) — they don't reserve layout space.
        node.min_width = Val::Px(0.0);
        node.min_height = Val::Px(0.0);
        node.border = UiRect::all(Val::Px(border_px));
        node.padding = UiRect::all(Val::Px(padding_px));
    }

    // Action-bar slots have a 3D button structure spawned by
    // `apply_3d_button_structure`, which locks the inner `ButtonFront`
    // height to whatever the outer node's height was at spawn time. As the
    // outer morphs between linear (50px) and radial (~32px), the front face
    // needs to track so the edge and front layers stay the same size. Walk
    // each slot's immediate children and resize any `ButtonFront` node.
    for slot_entity in slot_entities {
        let Ok(children) = children_query.get(slot_entity) else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut front_node) = fronts.get_mut(child) {
                front_node.width = Val::Px(slot_w);
                front_node.height = Val::Px(slot_h);
            }
        }
    }

    let icon_px = SPELL_ICON_SIZE * scale;
    for (mut node, _image) in &mut icons {
        // `update_action_bar_slots` toggles icons off (width/height=0) for
        // empty slots or gunslinger mode; leave those alone.
        if matches!(node.width, Val::Px(w) if w <= 0.5) {
            continue;
        }
        node.width = Val::Px(icon_px);
        node.height = Val::Px(icon_px);
    }

    // In radial mode the icon alone identifies the spell; the keyboard
    // hotkey glyph isn't meaningful on a controller and the spell name is
    // redundant with the icon. Collapsing both also prevents their intrinsic
    // size from overflowing the shrunken button.
    let text_hidden = progress.0 > 0.5;
    for (mut font, mut node) in &mut name_texts {
        font.font_size = SPELL_NAME_FONT_SIZE * scale;
        node.display = if text_hidden {
            Display::None
        } else {
            Display::Flex
        };
    }
    for (mut font, mut node) in &mut hotkey_texts {
        font.font_size = HOTKEY_FONT_SIZE * scale;
        node.display = if text_hidden {
            Display::None
        } else {
            Display::Flex
        };
    }
    for (_entity, _slot, mut node) in slots.iter_mut() {
        node.justify_content = if text_hidden {
            JustifyContent::Center
        } else {
            JustifyContent::SpaceBetween
        };
    }

    for mut vis in &mut inf {
        *vis = if progress.0 > 0.05 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// Applies a gold outline to the slot the right stick is currently pointing
/// at. Writes to the visible 3D layers (`ButtonFront` BorderColor + Outline
/// on `ButtonEdge`) — the outer Button's own Border is hidden behind the
/// edge/front children and isn't visible from the camera.
///
/// The Bevy plugin already gates this with `resource_changed::<RadialHoveredSlot>`,
/// but the run_if check happens before system params are evaluated and
/// systems still run on the very first frame; the `is_changed` early-out
/// here is belt-and-suspenders against any future caller that runs us
/// outside the `resource_changed` gate.
pub(super) fn highlight_radial_hovered_slot(
    hovered: Res<RadialHoveredSlot>,
    slots: Query<(Entity, &ActionBarSlot, &ButtonColors)>,
    children_query: Query<&Children>,
    mut front_query: Query<
        &mut BorderColor,
        (
            With<crate::ui::components::ButtonFront>,
            Without<crate::ui::components::ButtonEdge>,
        ),
    >,
    mut edge_query: Query<&mut Outline, With<crate::ui::components::ButtonEdge>>,
) {
    for (entity, slot, colors) in &slots {
        let is_hovered = hovered.0 == Some(slot.slot);
        let Ok(children) = children_query.get(entity) else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut bc) = front_query.get_mut(child) {
                *bc = if is_hovered {
                    BorderColor::all(RADIAL_HOVER_COLOR)
                } else {
                    BorderColor::all(colors.border)
                };
            }
            if let Ok(mut outline) = edge_query.get_mut(child) {
                outline.color = if is_hovered {
                    RADIAL_HOVER_COLOR
                } else {
                    crate::ui::constants::BUTTON_REST_OUTLINE
                };
            }
        }
    }
}

/// When `ActionBarKeyPressed` fires (keyboard hotkey OR right-stick + RT),
/// tag the corresponding slot with a short commit flash so the player gets a
/// clear visual confirmation that the spell was primed.
pub(super) fn flash_committed_slot(
    mut commands: Commands,
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    slots: Query<(Entity, &ActionBarSlot)>,
) {
    for event in action_bar_key.read() {
        for (entity, slot) in &slots {
            if slot.slot == event.slot {
                commands.entity(entity).insert(RadialCommitFlash {
                    remaining: RADIAL_COMMIT_FLASH_SECS,
                });
            }
        }
    }
}

/// Ticks down the commit flash and drives the slot's standard 3D press
/// animation (ButtonAnimState + front-border + outline). This is the same
/// visual used by keyboard hotkey presses, so a radial commit reads
/// identically to pressing `1`/`2`/`3`/`4`/`5` on the keyboard.
pub(super) fn tick_commit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut slots: Query<(
        Entity,
        &mut RadialCommitFlash,
        &ButtonColors,
        &Children,
        Option<&mut crate::ui::components::ButtonAnimState>,
    )>,
    mut front_query: Query<
        &mut BorderColor,
        (
            With<crate::ui::components::ButtonFront>,
            Without<crate::ui::components::ButtonEdge>,
        ),
    >,
    mut edge_query: Query<&mut Outline, With<crate::ui::components::ButtonEdge>>,
) {
    use crate::ui::constants::{
        BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_PRESSED_OUTLINE,
        BUTTON_REST_OUTLINE,
    };
    use crate::ui::styles::border_bright;

    for (entity, mut flash, colors, children, anim) in &mut slots {
        let was_fresh = flash.remaining == RADIAL_COMMIT_FLASH_SECS;
        flash.remaining -= time.delta_secs();

        if flash.remaining <= 0.0 {
            if let Some(mut a) = anim {
                a.target = BUTTON_3D_OFFSET_REST;
            }
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(colors.border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_REST_OUTLINE;
                }
            }
            commands.entity(entity).remove::<RadialCommitFlash>();
        } else if was_fresh {
            // Only apply pressed styling once, on the first tick.
            if let Some(mut a) = anim {
                a.target = BUTTON_3D_OFFSET_PRESSED;
            }
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(border_bright(colors.border));
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_PRESSED_OUTLINE;
                }
            }
        }
    }
}
