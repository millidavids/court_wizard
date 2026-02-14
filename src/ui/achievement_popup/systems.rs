use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::messages::AchievementUnlockedMessage;

use super::components::{AchievementPopup, AchievementPopupTimer, AchievementQueue};
use super::constants::*;

/// Queues achievements as they unlock.
pub(super) fn queue_achievements(
    mut achievement_events: MessageReader<AchievementUnlockedMessage>,
    mut queue: ResMut<AchievementQueue>,
) {
    for event in achievement_events.read() {
        queue.push(event.id);
    }
}

/// Spawns the next achievement popup from the queue if no popup is currently displayed.
pub(super) fn spawn_next_popup(
    mut commands: Commands,
    mut queue: ResMut<AchievementQueue>,
    active_popups: Query<&AchievementPopup>,
) {
    // Only spawn a new popup if there isn't one already showing
    if active_popups.is_empty() && !queue.is_empty() {
        if let Some(id) = queue.pop() {
            spawn_popup(&mut commands, id);
        }
    }
}

fn spawn_popup(commands: &mut Commands, id: AchievementId) {
    commands
        .spawn((
            // Root: absolute-positioned container at top-center
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Percent(50.0),
                // Shift left by half width to center
                margin: UiRect {
                    left: Val::Px(-150.0),
                    ..default()
                },
                width: Val::Px(300.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            BackgroundColor(BACKGROUND_COLOR),
            // Make the popup non-interactive so clicks pass through
            Pickable::IGNORE,
            GlobalZIndex(999),
            AchievementPopup,
            AchievementPopupTimer::new(DISPLAY_DURATION, FADE_DURATION),
        ))
        .with_children(|parent| {
            // "Achievement Unlocked" header
            parent.spawn((
                Text::new("Achievement Unlocked"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(HEADER_COLOR),
                Pickable::IGNORE,
            ));

            // Achievement name
            parent.spawn((
                Text::new(id.display_name()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TITLE_COLOR),
                Pickable::IGNORE,
            ));

            // Achievement description
            parent.spawn((
                Text::new(id.description()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));

            // Unlock reward (if any)
            if let Some(reward) = id.unlock_reward() {
                parent.spawn((
                    Text::new(reward),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.9, 0.4)), // Bright green
                    Pickable::IGNORE,
                ));
            }
        });
}

/// Ticks popup timers, applies fade, and despawns expired popups.
pub(super) fn update_achievement_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<
        (
            Entity,
            &mut AchievementPopupTimer,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<AchievementPopup>,
    >,
    children_query: Query<&Children>,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (entity, mut timer, mut bg, mut border) in &mut popups {
        timer.elapsed += time.delta_secs();

        if timer.is_expired() {
            commands.entity(entity).despawn();
            continue;
        }

        let opacity = timer.opacity();

        // Fade background
        let mut bg_color = BACKGROUND_COLOR.to_srgba();
        bg_color.alpha *= opacity;
        bg.0 = bg_color.into();

        // Fade border
        let mut border_srgba = BORDER_COLOR.to_srgba();
        border_srgba.alpha = opacity;
        *border = BorderColor::all(border_srgba);

        // Fade all child text elements
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut text_color) = text_color_query.get_mut(child) {
                    let mut c = text_color.0.to_srgba();
                    c.alpha = opacity;
                    text_color.0 = c.into();
                }
            }
        }
    }
}
