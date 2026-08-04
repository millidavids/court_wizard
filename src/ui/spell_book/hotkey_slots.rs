//! The "Assign Hotkey" row: five slot boxes showing what each slot holds.
//!
//! Each box renders the icon of the spell currently bound to that slot, with
//! the slot number as a small corner badge — so the panel communicates the
//! whole loadout, not just whether the previewed spell happens to be bound.

use bevy::prelude::*;

use super::components::{HotkeySlotButton, SelectedSpellPreview};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::units::wizard::components::Spell;
use crate::networking::session::MultiplayerSession;
use crate::ui::action_bar::systems::slot_icon;
use crate::ui::components::{ButtonColors, SpellIconAssets};

/// Number of assignable action bar slots.
pub(super) const SLOT_COUNT: u8 = 5;

/// Marks the icon image inside a hotkey slot box.
///
/// The boxes are restructured into 3D buttons (`apply_3d_button_structure`
/// reparents their children under a `ButtonFront`), so the refresh finds its
/// target by this marker rather than by walking the button's direct children.
#[derive(Component)]
pub(super) struct HotkeySlotIcon(pub u8);

/// Background, border, and text colors for a hotkey box.
///
/// "Active" means *the previewed spell is bound to this slot* — independent of
/// which icon the box displays.
fn hotkey_colors(is_active: bool) -> (Color, Color, Color) {
    if is_active {
        (HOTKEY_ACTIVE_BG, HOTKEY_ACTIVE_BORDER, HOTKEY_ACTIVE_TEXT)
    } else {
        (
            HOTKEY_INACTIVE_BG,
            HOTKEY_INACTIVE_BORDER,
            HOTKEY_INACTIVE_TEXT,
        )
    }
}

/// Spawns the "Assign Hotkey" label and the five slot boxes.
pub(super) fn spawn_hotkey_row(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    config: &GameConfig,
    mp_session: Option<&MultiplayerSession>,
    icon_assets: &SpellIconAssets,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|hotkey_section| {
            hotkey_section.spawn((
                Text::new("Assign Hotkey"),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(LABEL_COLOR),
            ));

            hotkey_section
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(HOTKEY_BOX_GAP),
                    ..default()
                })
                .with_children(|row| {
                    for slot in 0..SLOT_COUNT {
                        spawn_hotkey_box(row, slot, spell, config, mp_session, icon_assets);
                    }
                });
        });
}

/// Spawns one hotkey box: the bound spell's icon plus a corner slot number.
fn spawn_hotkey_box(
    row: &mut ChildSpawnerCommands,
    slot: u8,
    spell: Spell,
    config: &GameConfig,
    mp_session: Option<&MultiplayerSession>,
    icon_assets: &SpellIconAssets,
) {
    let is_active = config.action_bar_slots[slot as usize] == Some(spell);
    let (bg, border, text_color) = hotkey_colors(is_active);
    let icon = slot_icon(config, slot as usize, mp_session, icon_assets);
    let icon_visibility = if icon.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
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
            ImageNode::new(icon.unwrap_or_default()),
            Node {
                width: Val::Px(HOTKEY_ICON_SIZE),
                height: Val::Px(HOTKEY_ICON_SIZE),
                ..default()
            },
            icon_visibility,
            HotkeySlotIcon(slot),
        ));

        // Corner badge. Absolute, so it overlays the icon rather than
        // displacing it; resolves against the 3D front face once the button
        // is restructured, which keeps it inside the box.
        btn.spawn((
            Text::new(format!("{}", slot + 1)),
            TextFont::from_font_size(HOTKEY_FONT_SIZE),
            TextColor(text_color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(2.0),
                left: Val::Px(3.0),
                ..default()
            },
        ));
    });
}

/// Refreshes every hotkey box: its icon (loadout changed) and its active/
/// inactive treatment (previewed spell changed).
///
/// Owns `ButtonActive` for the boxes as well as their palette. Both are needed:
/// `sync_front_face_colors` is filtered `Without<ButtonActive>`, so a box left
/// wrongly marked active would keep a stale 3D front face no matter what its
/// `ButtonColors` say.
#[allow(clippy::too_many_arguments)]
pub(super) fn refresh_hotkey_slots(
    mut commands: Commands,
    selected: Res<SelectedSpellPreview>,
    config: Res<GameConfig>,
    mp_session: Option<Res<MultiplayerSession>>,
    icon_assets: Res<SpellIconAssets>,
    mut boxes: Query<(
        Entity,
        &HotkeySlotButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ButtonColors,
        Has<crate::ui::components::ButtonActive>,
    )>,
    mut icons: Query<(&HotkeySlotIcon, &mut ImageNode, &mut Visibility)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut TextColor>,
) {
    if !selected.is_changed() && !config.is_changed() {
        return;
    }

    for (marker, mut image, mut visibility) in icons.iter_mut() {
        match slot_icon(
            &config,
            marker.0 as usize,
            mp_session.as_deref(),
            &icon_assets,
        ) {
            Some(handle) => {
                image.image = handle;
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }

    // Collect first: the digit badge's `TextColor` is reached through the
    // children walk, which can't borrow while `boxes` is iterated.
    let mut text_updates: Vec<(Entity, Color)> = Vec::new();
    for (entity, slot_btn, mut bg, mut border, mut colors, was_active) in boxes.iter_mut() {
        let is_active = config.action_bar_slots[slot_btn.0 as usize] == Some(selected.0);
        let (new_bg, new_border, new_text) = hotkey_colors(is_active);
        bg.0 = new_bg;
        *border = BorderColor::all(new_border);
        colors.background = new_bg;
        colors.border = new_border;
        text_updates.push((entity, new_text));

        // Toggle only on a real transition — `enforce_active_button_state` keys
        // off `Added<ButtonActive>`, so re-inserting every frame would re-fire
        // the press animation continuously.
        if is_active && !was_active {
            commands
                .entity(entity)
                .insert(crate::ui::components::ButtonActive);
        } else if !is_active && was_active {
            commands
                .entity(entity)
                .remove::<crate::ui::components::ButtonActive>();
        }
    }

    // The boxes are restructured into 3D buttons, so the digit badge sits a
    // level deeper than it was authored — hence the grandchild walk.
    for (entity, new_text) in &text_updates {
        let Ok(children) = children_query.get(*entity) else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                tc.0 = *new_text;
            }
            if let Ok(grandchildren) = children_query.get(child) {
                for gc in grandchildren.iter() {
                    if let Ok(mut tc) = text_query.get_mut(gc) {
                        tc.0 = *new_text;
                    }
                }
            }
        }
    }
}
