//! In-game wave display, flashes, buff tracker.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::cauldron::brews::BrewEffect;
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::components::OnGameplayScreen;
use crate::game::messages::WaveSpawnedMessage;
use crate::game::resources::WaveState;

/// Blocks spell input when the mouse is interacting with a UI button so
/// clicking a HUD button doesn't simultaneously fire a spell. With a
/// gamepad, spells are triggered by RT (not by a UI cursor), and the hidden
/// OS cursor may sit over a HUD button at any time — gating on mouse mode
/// prevents those incidental hovers from silently killing RT casts.
pub(super) fn update_wave_display(
    wave_state: Res<WaveState>,
    mut wave_display_query: Query<&mut Text, With<WaveDisplay>>,
) {
    if wave_state.is_changed()
        && let Ok(mut text) = wave_display_query.single_mut()
    {
        **text = format!(
            "Wave: {} / {}",
            wave_state.current_wave + 1,
            wave_state.total_waves
        );
    }
}

/// Spawns a "Wave X incoming!" flash when a new wave spawns.
pub(super) fn spawn_wave_incoming_flash(
    mut commands: Commands,
    mut wave_events: MessageReader<WaveSpawnedMessage>,
    existing_flash: Query<Entity, With<WaveIncomingFlash>>,
) {
    for event in wave_events.read() {
        spawn_flash_banner(
            &mut commands,
            &existing_flash,
            &format!("Wave {} incoming!", event.wave_number),
            WAVE_FLASH_COLOR,
        );
    }
}

/// Spawns a "The King calls for a retreat!" flash when retreat triggers.
pub(super) fn spawn_retreat_flash(
    mut commands: Commands,
    mut retreat_events: MessageReader<crate::game::messages::RetreatMessage>,
    existing_flash: Query<Entity, With<WaveIncomingFlash>>,
) {
    for _event in retreat_events.read() {
        spawn_flash_banner(
            &mut commands,
            &existing_flash,
            "The King calls for a retreat!",
            RETREAT_FLASH_COLOR,
        );
    }
}

/// Spawns a centered flash banner at the top of the screen.
///
/// Removes any existing flash before spawning the new one.
fn spawn_flash_banner(
    commands: &mut Commands,
    existing_flash: &Query<Entity, With<WaveIncomingFlash>>,
    text: &str,
    color: Color,
) {
    for entity in existing_flash {
        commands.entity(entity).try_despawn();
    }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(15.0),
            left: Val::Percent(5.0),
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(text),
        TextFont::from_font_size(WAVE_FLASH_FONT_SIZE),
        TextColor(color),
        WaveIncomingFlash {
            timer: WAVE_FLASH_DURATION,
        },
        Pickable::IGNORE,
        GlobalZIndex(998),
        crate::game::components::OnGameplayScreen,
    ));
}

/// Fades and despawns the wave incoming flash.
pub(super) fn update_wave_incoming_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut WaveIncomingFlash, &mut TextColor)>,
) {
    for (entity, mut flash, mut text_color) in &mut flash_query {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            // Fade out over the last second
            let opacity = (flash.timer / 1.0).min(1.0);
            let mut c = text_color.0.to_srgba();
            c.alpha = opacity;
            text_color.0 = c.into();
        }
    }
}

// ===== Buff Tracker Systems =====

/// Returns the primary abbreviation for a buff (from its first effect).
fn buff_abbreviation(effects: &[BrewEffect]) -> &'static str {
    effects.first().map(|e| e.abbreviation()).unwrap_or("??")
}

/// Returns the background color for a buff box based on the brew's averaged color.
fn buff_box_color(effects: &[BrewEffect]) -> Color {
    // Use a muted version of the first effect's type color
    match effects.first() {
        Some(BrewEffect::ManaRegenMultiplier(_) | BrewEffect::MaxManaMultiplier(_)) => {
            Color::srgba(0.2, 0.3, 0.6, 0.7)
        }
        Some(
            BrewEffect::SpellPowerMultiplier(_)
            | BrewEffect::SpellRangeMultiplier(_)
            | BrewEffect::CastSpeedMultiplier(_),
        ) => Color::srgba(0.4, 0.2, 0.5, 0.7),
        Some(
            BrewEffect::DefenderDamageBonus(_)
            | BrewEffect::AttackSpeedMultiplier(_)
            | BrewEffect::EffectivenessBonus(_),
        ) => Color::srgba(0.5, 0.3, 0.1, 0.7),
        Some(
            BrewEffect::DefenderHealPerSecond(_)
            | BrewEffect::DamageResistancePercent(_)
            | BrewEffect::DefenderShieldPerSecond(_),
        ) => Color::srgba(0.1, 0.4, 0.2, 0.7),
        Some(
            BrewEffect::DefenderSpeedBonus(_)
            | BrewEffect::AttackerSlowPercent(_)
            | BrewEffect::BuffDurationMultiplier(_),
        ) => Color::srgba(0.3, 0.3, 0.15, 0.7),
        None => Color::srgba(0.2, 0.2, 0.2, 0.7),
    }
}

/// Rebuilds buff tracker boxes when the CauldronBuffs resource changes.
///
/// Despawns all existing boxes and re-creates them from the current active buffs.
pub(super) fn update_buff_tracker(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    container_query: Query<Entity, With<BuffTrackerContainer>>,
    existing_boxes: Query<Entity, With<BuffTrackerBox>>,
    existing_tooltips: Query<Entity, With<BuffTooltip>>,
) {
    if !cauldron_buffs.is_changed() {
        return;
    }

    // Despawn existing buff boxes
    for entity in &existing_boxes {
        commands.entity(entity).try_despawn();
    }
    // Despawn any lingering tooltips
    for entity in &existing_tooltips {
        commands.entity(entity).try_despawn();
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    // Spawn new boxes for each active buff
    for (i, buff) in cauldron_buffs.active_buffs.iter().enumerate() {
        let abbr = buff_abbreviation(&buff.effects);
        let bg_color = buff_box_color(&buff.effects);

        commands.entity(container).with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(BUFF_BOX_SIZE),
                        height: Val::Px(BUFF_BOX_SIZE),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(BUFF_BOX_BORDER_WIDTH)),
                        row_gap: Val::Px(2.0),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(bg_color),
                    BorderColor::all(BUFF_BOX_BORDER_COLOR),
                    BuffTrackerBox(i),
                ))
                .with_children(|box_node| {
                    // Abbreviation label
                    box_node.spawn((
                        Text::new(abbr),
                        TextFont::from_font_size(BUFF_LABEL_FONT_SIZE),
                        TextColor(Color::WHITE),
                    ));
                    // Timer text
                    box_node.spawn((
                        Text::new(format!("{:.0}s", buff.time_remaining)),
                        TextFont::from_font_size(BUFF_TIMER_FONT_SIZE),
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                        BuffTimerText(i),
                    ));
                });
        });
    }
}

/// Updates buff timer text every frame.
pub(super) fn update_buff_timers(
    cauldron_buffs: Res<CauldronBuffs>,
    mut timer_query: Query<(&BuffTimerText, &mut Text)>,
) {
    for (timer, mut text) in &mut timer_query {
        if let Some(buff) = cauldron_buffs.active_buffs.get(timer.0) {
            **text = format!("{:.0}s", buff.time_remaining.ceil());
        }
    }
}

/// Shows a tooltip when hovering over a buff tracker box.
pub(super) fn show_buff_tooltip(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    box_query: Query<(&Interaction, &BuffTrackerBox), Changed<Interaction>>,
    existing_tooltips: Query<Entity, With<BuffTooltip>>,
) {
    for (interaction, buff_box) in &box_query {
        match interaction {
            Interaction::Hovered => {
                // Despawn any existing tooltip
                for entity in &existing_tooltips {
                    commands.entity(entity).try_despawn();
                }

                let Some(buff) = cauldron_buffs.active_buffs.get(buff_box.0) else {
                    continue;
                };

                // Build tooltip text
                let tooltip_lines: Vec<String> =
                    buff.effects.iter().map(|e| e.display_text()).collect();
                let tooltip_text = tooltip_lines.join("\n");

                // Spawn tooltip as absolute-positioned node
                commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(BUFF_BOX_SIZE + BUFF_BOX_GAP + 20.0 + 10.0),
                            left: Val::Px(20.0),
                            max_width: Val::Px(BUFF_TOOLTIP_MAX_WIDTH),
                            padding: UiRect::all(Val::Px(BUFF_TOOLTIP_PADDING)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(BUFF_TOOLTIP_BG),
                        BorderColor::all(BUFF_TOOLTIP_BORDER),
                        GlobalZIndex(999),
                        BuffTooltip,
                        Pickable::IGNORE,
                        OnGameplayScreen,
                    ))
                    .with_children(|tooltip| {
                        tooltip.spawn((
                            Text::new(tooltip_text),
                            TextFont::from_font_size(BUFF_TOOLTIP_FONT_SIZE),
                            TextColor(Color::WHITE),
                        ));
                    });
            }
            Interaction::None => {
                // Despawn tooltip when hover ends
                for entity in &existing_tooltips {
                    commands.entity(entity).try_despawn();
                }
            }
            Interaction::Pressed => {}
        }
    }
}
