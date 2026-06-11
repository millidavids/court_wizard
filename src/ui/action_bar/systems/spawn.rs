use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::radial::{ease, linear_pos, radial_pos};
use crate::config::input_bindings::{InputBindings, key_display_name};
use crate::config::{GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::ui::components::{ButtonColors, GunIconAssets, SpellIconAssets};
use crate::ui::systems::scale_font_by_text_width;

#[cfg(debug_assertions)]
const DEBUG_BUTTON_SIZE: f32 = 30.0;
#[cfg(debug_assertions)]
const DEBUG_BUTTON_GAP: f32 = 8.0;
#[cfg(debug_assertions)]
pub(crate) const DEBUG_BUTTON_BG_OFF: Color = Color::srgba(0.2, 0.1, 0.1, 0.8);
#[cfg(debug_assertions)]
pub(crate) const DEBUG_BUTTON_BG_ON: Color = Color::srgba(0.1, 0.5, 0.1, 0.9);
#[cfg(debug_assertions)]
const DEBUG_BUTTON_BORDER: Color = Color::srgba(0.6, 0.3, 0.3, 1.0);

/// Resets the radial-vs-linear morph progress on gameplay start. Snaps
/// directly to the radial endpoint when a gamepad is the active input device
/// so the action bar renders in its final radial layout from the very first
/// frame — otherwise the controller in-game tutorial (which references the
/// radial controls) would talk about a layout that hadn't morphed yet.
pub(crate) fn reset_layout_progress(
    mut progress: ResMut<ActionBarLayoutProgress>,
    active: Res<ActiveInputDevice>,
) {
    progress.0 = if active.is_gamepad() { 1.0 } else { 0.0 };
}

/// Calculates the appropriate font size for action bar spell names based on max line width.
pub(crate) fn calculate_action_bar_font_size(name: &str) -> f32 {
    let max_line_width = name.lines().map(|line| line.len()).max().unwrap_or(0) as f32;
    scale_font_by_text_width(max_line_width, 6.0, 11.0, 0.65, SPELL_NAME_FONT_SIZE)
}

/// Returns the spell that should appear in this action bar slot, filtering
/// out any spells that are not allowed in the current session. Used by all
/// the slot read sites so the user's saved loadout is preserved (we never
/// mutate `config.action_bar_slots`) but the UI / input layer treats
/// MP-disallowed spells as empty slots.
///
/// Currently only filters `Telekinesis` in MP (drops are SP-only); see
/// `Spell::is_mp_allowed`.
pub(crate) fn effective_slot(
    config: &GameConfig,
    slot_idx: usize,
    mp_session: Option<&crate::networking::session::MultiplayerSession>,
) -> Option<crate::game::units::wizard::components::Spell> {
    let spell = *config.action_bar_slots.get(slot_idx)?;
    let spell = spell?;
    if mp_session.is_some() && !spell.is_mp_allowed() {
        return None;
    }
    Some(spell)
}

/// Spawns the action bar UI at the bottom-left of the screen.
/// Clears action bar spells that are blocked by the current wizard type.
/// Runs once when entering gameplay, before the action bar is spawned.
pub(crate) fn clear_blocked_action_bar_spells(mut config: ResMut<GameConfig>) {
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

pub(crate) fn spawn_action_bar(
    mut commands: Commands,
    config: Res<GameConfig>,
    bindings: Res<InputBindings>,
    icon_assets: Res<SpellIconAssets>,
    gun_icon_assets: Res<GunIconAssets>,
    layout_progress: Res<ActionBarLayoutProgress>,
    mp_session: Option<Res<crate::networking::session::MultiplayerSession>>,
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
                        let spell = effective_slot(&config, slot as usize, mp_session.as_deref());
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
