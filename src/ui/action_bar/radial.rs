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
fn linear_pos(i: u8) -> Vec2 {
    let left =
        ACTION_BAR_LEFT_MARGIN + i as f32 * (SLOT_BUTTON_STYLE.width + SLOT_GAP);
    Vec2::new(left, ACTION_BAR_BOTTOM_MARGIN)
}

/// Radial target position of slot `i`, placed at the corresponding 72° wedge
/// on a ring centered at `(RADIAL_CENTER_LEFT, RADIAL_CENTER_BOTTOM)`. Slot 0
/// sits at the top (12 o'clock); subsequent slots go clockwise. Positions
/// account for the shrunken radial button size so each slot's center lands
/// exactly on the ring.
fn radial_pos(i: u8) -> Vec2 {
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
fn ease(t: f32) -> f32 {
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
    mut slots: Query<(&ActionBarSlot, &mut Node), Without<DebugManaButton>>,
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
) {
    let target = if active.is_gamepad() { 1.0 } else { 0.0 };
    let step = time.delta_secs() / RADIAL_TRANSITION_SECS;
    if (progress.0 - target).abs() <= step {
        progress.0 = target;
    } else {
        progress.0 += (target - progress.0).signum() * step;
    }
    let t = ease(progress.0);
    let scale = 1.0 + (RADIAL_SLOT_SCALE - 1.0) * t;

    let slot_w = SLOT_BUTTON_STYLE.width * scale;
    let slot_h = SLOT_BUTTON_STYLE.height * scale;
    let border_px = SLOT_BUTTON_STYLE.border_width * scale;
    let padding_px = 2.0 * scale;
    for (slot, mut node) in &mut slots {
        let from = linear_pos(slot.slot);
        let to = radial_pos(slot.slot);
        let pos = from.lerp(to, t);
        node.left = Val::Px(pos.x);
        node.bottom = Val::Px(pos.y);
        node.width = Val::Px(slot_w);
        node.height = Val::Px(slot_h);
        // Default `min_*: Auto` lets intrinsic content force the button to
        // grow past the set width/height. Pinning min to 0 lets width/height
        // be authoritative without clipping the content.
        node.min_width = Val::Px(0.0);
        node.min_height = Val::Px(0.0);
        node.border = UiRect::all(Val::Px(border_px));
        node.padding = UiRect::all(Val::Px(padding_px));
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
    for (_slot, mut node) in slots.iter_mut() {
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

/// Applies a yellow outline to the slot the right stick is currently
/// pointing at. Only has a visible effect while the radial layout is active
/// since `RadialHoveredSlot` stays `None` under mouse input.
pub(super) fn highlight_radial_hovered_slot(
    hovered: Res<RadialHoveredSlot>,
    mut slots: Query<(&ActionBarSlot, &mut BorderColor, &mut Node, &ButtonColors)>,
) {
    for (slot, mut border_color, mut node, colors) in &mut slots {
        if hovered.0 == Some(slot.slot) {
            *border_color = BorderColor::all(RADIAL_HOVER_COLOR);
            node.border = UiRect::all(Val::Px(RADIAL_HOVER_BORDER));
        } else {
            *border_color = BorderColor::all(colors.border);
            node.border = UiRect::all(Val::Px(SLOT_BUTTON_STYLE.border_width));
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

/// Ticks down the commit flash, painting the slot yellow while active and
/// restoring its default background when the timer expires.
pub(super) fn tick_commit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut RadialCommitFlash,
        &mut BackgroundColor,
        &ButtonColors,
    )>,
) {
    for (entity, mut flash, mut bg, colors) in &mut query {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            *bg = BackgroundColor(colors.background);
            commands.entity(entity).remove::<RadialCommitFlash>();
        } else {
            *bg = BackgroundColor(RADIAL_COMMIT_FLASH_COLOR);
        }
    }
}
