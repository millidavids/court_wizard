use bevy::prelude::*;

use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::messages::{
    AchievementUnlockedMessage, ComboDiscoveredMessage, IngredientCollectedMessage,
    SpellResearchedMessage,
};
use crate::game::units::wizard::components::Spell;

use super::components::{AchievementPopup, AchievementPopupTimer, PopupEntry, PopupQueue};
use super::constants::*;

/// Queues achievements, ingredient collections, and spell research as they happen.
pub(super) fn queue_popups(
    mut achievement_events: MessageReader<AchievementUnlockedMessage>,
    mut ingredient_events: MessageReader<IngredientCollectedMessage>,
    mut spell_events: MessageReader<SpellResearchedMessage>,
    mut combo_events: MessageReader<ComboDiscoveredMessage>,
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
    for event in combo_events.read() {
        queue.push(PopupEntry::ComboDiscovered {
            name: event.name,
            description: event.description,
        });
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
            PopupEntry::ComboDiscovered { name, description } => {
                spawn_combo_popup(&mut commands, name, description)
            }
            PopupEntry::Toast { message } => spawn_toast_popup(&mut commands, message),
        }
    }
}

/// Shared popup spawning with configurable colors and text content.
#[allow(clippy::too_many_arguments)]
fn spawn_popup(
    commands: &mut Commands,
    background: Color,
    border: Color,
    header: &str,
    header_color: Color,
    title: impl Into<String>,
    title_color: Color,
    description: impl Into<String>,
    extra_child: Option<(String, f32, Color)>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                right: Val::Px(20.0),
                width: Val::Px(300.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(border),
            BackgroundColor(background),
            Pickable::IGNORE,
            GlobalZIndex(999),
            AchievementPopup,
            AchievementPopupTimer::new(DISPLAY_DURATION, FADE_DURATION),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(header),
                TextFont::from_font_size(12.0),
                TextColor(header_color),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(title),
                TextFont::from_font_size(18.0),
                TextColor(title_color),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(description),
                TextFont::from_font_size(13.0),
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));

            if let Some((text, font_size, color)) = extra_child {
                parent.spawn((
                    Text::new(text),
                    TextFont::from_font_size(font_size),
                    TextColor(color),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn spawn_achievement_popup(commands: &mut Commands, id: AchievementId) {
    let extra = id
        .unlock_reward()
        .map(|reward| (reward.to_string(), 14.0, Color::srgb(0.4, 0.9, 0.4)));
    spawn_popup(
        commands,
        BACKGROUND_COLOR,
        BORDER_COLOR,
        "Achievement Unlocked",
        HEADER_COLOR,
        id.display_name(),
        TITLE_COLOR,
        id.description(),
        extra,
    );
}

fn spawn_ingredient_popup(commands: &mut Commands, ingredient: Ingredient) {
    spawn_popup(
        commands,
        INGREDIENT_BACKGROUND_COLOR,
        INGREDIENT_BORDER_COLOR,
        "Ingredient Discovered!",
        INGREDIENT_HEADER_COLOR,
        ingredient.name(),
        INGREDIENT_TITLE_COLOR,
        ingredient.description(),
        None,
    );
}

fn spawn_spell_researched_popup(commands: &mut Commands, spell: Spell) {
    spawn_popup(
        commands,
        SPELL_BACKGROUND_COLOR,
        SPELL_BORDER_COLOR,
        "Spell Researched!",
        SPELL_HEADER_COLOR,
        spell.display_name(),
        SPELL_TITLE_COLOR,
        spell.description(),
        None,
    );
}

fn spawn_toast_popup(commands: &mut Commands, message: &str) {
    spawn_popup(
        commands,
        BACKGROUND_COLOR,
        BORDER_COLOR,
        "",
        Color::NONE,
        message,
        DESCRIPTION_COLOR,
        "",
        None,
    );
}

fn spawn_combo_popup(commands: &mut Commands, name: &str, description: &str) {
    spawn_popup(
        commands,
        COMBO_BACKGROUND_COLOR,
        COMBO_BORDER_COLOR,
        "Combo Discovered!",
        COMBO_HEADER_COLOR,
        name,
        COMBO_TITLE_COLOR,
        description,
        None,
    );
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
            commands.entity(entity).try_despawn();
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

        // Fade all descendant text elements (children + grandchildren for shadow wrappers)
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(mut text_color) = text_color_query.get_mut(child) {
                    let mut c = text_color.0.to_srgba();
                    c.alpha = opacity;
                    text_color.0 = c.into();
                }
                if let Ok(grandchildren) = children_query.get(child) {
                    for gc in grandchildren.iter() {
                        if let Ok(mut text_color) = text_color_query.get_mut(gc) {
                            let mut c = text_color.0.to_srgba();
                            c.alpha = opacity;
                            text_color.0 = c.into();
                        }
                    }
                }
            }
        }
    }
}
