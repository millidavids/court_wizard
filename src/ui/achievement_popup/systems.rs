use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::messages::{
    AchievementUnlockedMessage, IngredientCollectedMessage, SpellResearchedMessage,
};
use crate::game::units::wizard::components::Spell;

use super::components::{AchievementPopup, AchievementPopupTimer, PopupEntry, PopupQueue};
use super::constants::*;

/// Queues achievements, ingredient collections, and spell research as they happen.
pub(super) fn queue_popups(
    mut achievement_events: MessageReader<AchievementUnlockedMessage>,
    mut ingredient_events: MessageReader<IngredientCollectedMessage>,
    mut spell_events: MessageReader<SpellResearchedMessage>,
    mut queue: ResMut<PopupQueue>,
) {
    for event in achievement_events.read() {
        queue.push(PopupEntry::Achievement(event.id));
    }
    for event in ingredient_events.read() {
        queue.push(PopupEntry::IngredientCollected(event.ingredient));
    }
    for event in spell_events.read() {
        queue.push(PopupEntry::SpellResearched(event.spell));
    }
}

/// Spawns the next popup from the queue if no popup is currently displayed.
pub(super) fn spawn_next_popup(
    mut commands: Commands,
    mut queue: ResMut<PopupQueue>,
    active_popups: Query<&AchievementPopup>,
) {
    if active_popups.is_empty()
        && !queue.is_empty()
        && let Some(entry) = queue.pop()
    {
        match entry {
            PopupEntry::Achievement(id) => spawn_achievement_popup(&mut commands, id),
            PopupEntry::IngredientCollected(ingredient) => {
                spawn_ingredient_popup(&mut commands, ingredient)
            }
            PopupEntry::SpellResearched(spell) => {
                spawn_spell_researched_popup(&mut commands, spell)
            }
        }
    }
}

fn spawn_achievement_popup(commands: &mut Commands, id: AchievementId) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Percent(50.0),
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
            Pickable::IGNORE,
            GlobalZIndex(999),
            AchievementPopup,
            AchievementPopupTimer::new(DISPLAY_DURATION, FADE_DURATION),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Achievement Unlocked"),
                TextFont::from_font_size(12.0),
                TextColor(HEADER_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(id.display_name()),
                TextFont::from_font_size(18.0),
                TextColor(TITLE_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(id.description()),
                TextFont::from_font_size(13.0),
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));

            if let Some(reward) = id.unlock_reward() {
                parent.spawn((
                    Text::new(reward),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgb(0.4, 0.9, 0.4)),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn spawn_ingredient_popup(commands: &mut Commands, ingredient: Ingredient) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Percent(50.0),
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
            BorderColor::all(INGREDIENT_BORDER_COLOR),
            BackgroundColor(INGREDIENT_BACKGROUND_COLOR),
            Pickable::IGNORE,
            GlobalZIndex(999),
            AchievementPopup,
            AchievementPopupTimer::new(DISPLAY_DURATION, FADE_DURATION),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Ingredient Discovered!"),
                TextFont::from_font_size(12.0),
                TextColor(INGREDIENT_HEADER_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(ingredient.name()),
                TextFont::from_font_size(18.0),
                TextColor(INGREDIENT_TITLE_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(ingredient.description()),
                TextFont::from_font_size(13.0),
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_spell_researched_popup(commands: &mut Commands, spell: Spell) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                left: Val::Percent(50.0),
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
            BorderColor::all(SPELL_BORDER_COLOR),
            BackgroundColor(SPELL_BACKGROUND_COLOR),
            Pickable::IGNORE,
            GlobalZIndex(999),
            AchievementPopup,
            AchievementPopupTimer::new(DISPLAY_DURATION, FADE_DURATION),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Spell Researched!"),
                TextFont::from_font_size(12.0),
                TextColor(SPELL_HEADER_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(spell.display_name()),
                TextFont::from_font_size(18.0),
                TextColor(SPELL_TITLE_COLOR),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(spell.description()),
                TextFont::from_font_size(13.0),
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));
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
        let mut bg_color = bg.0.to_srgba();
        bg_color.alpha *= opacity;
        bg.0 = bg_color.into();

        // Fade border
        let mut border_srgba = border.top.to_srgba();
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
