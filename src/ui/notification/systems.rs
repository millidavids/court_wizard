use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::achievements::messages::WizardTypeUnlockedMessage;
use crate::game::cauldron::brews::Ingredient;
use crate::game::messages::{
    ComboDiscoveredMessage, IngredientCollectedMessage, SpellResearchedMessage,
    TalentTierUnlockedMessage,
};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::talents::constants::tier_name;

use super::components::{Notification, NotificationEntry, NotificationQueue, NotificationTimer};
use super::constants::*;

/// Queues notifications for wizard unlocks, ingredients, spell research, talent tiers, and combos.
pub(super) fn queue_notifications(
    mut wizard_events: MessageReader<WizardTypeUnlockedMessage>,
    mut ingredient_events: MessageReader<IngredientCollectedMessage>,
    mut spell_events: MessageReader<SpellResearchedMessage>,
    mut talent_events: MessageReader<TalentTierUnlockedMessage>,
    mut combo_events: MessageReader<ComboDiscoveredMessage>,
    mut queue: ResMut<NotificationQueue>,
) {
    for event in wizard_events.read() {
        if event.newly_unlocked {
            queue.push(NotificationEntry::WizardUnlocked(event.wizard_type));
        }
    }
    for event in ingredient_events.read() {
        queue.push(NotificationEntry::IngredientCollected(event.ingredient));
    }
    for event in spell_events.read() {
        queue.push(NotificationEntry::SpellResearched(event.spell));
    }
    for event in talent_events.read() {
        queue.push(NotificationEntry::TalentTierUnlocked {
            spell: event.spell,
            tier: event.tier,
        });
    }
    for event in combo_events.read() {
        queue.push(NotificationEntry::ComboDiscovered {
            name: event.name,
            description: event.description,
        });
    }
}

/// Spawns the next notification from the queue if no notification is currently displayed.
pub(super) fn spawn_next_notification(
    mut commands: Commands,
    mut queue: ResMut<NotificationQueue>,
    active: Query<&Notification>,
) {
    if active.is_empty()
        && !queue.is_empty()
        && let Some(entry) = queue.pop()
    {
        match entry {
            NotificationEntry::WizardUnlocked(wizard_type) => {
                spawn_wizard_notification(&mut commands, wizard_type)
            }
            NotificationEntry::IngredientCollected(ingredient) => {
                spawn_ingredient_notification(&mut commands, ingredient)
            }
            NotificationEntry::SpellResearched(spell) => {
                spawn_spell_researched_notification(&mut commands, spell)
            }
            NotificationEntry::TalentTierUnlocked { spell, tier } => {
                spawn_talent_tier_notification(&mut commands, spell, tier)
            }
            NotificationEntry::ComboDiscovered { name, description } => {
                spawn_combo_notification(&mut commands, name, description)
            }
            NotificationEntry::Toast { message } => {
                spawn_toast_notification(&mut commands, message)
            }
        }
    }
}

/// Shared notification spawning with configurable colors and text content.
#[allow(clippy::too_many_arguments)]
fn spawn_notification(
    commands: &mut Commands,
    background: Color,
    border: Color,
    header: impl Into<String>,
    header_color: Color,
    title: impl Into<String>,
    title_color: Color,
    description: impl Into<String>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                right: Val::Px(20.0),
                width: Val::Px(240.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(3.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(border),
            BackgroundColor(background),
            Pickable::IGNORE,
            GlobalZIndex(999),
            Notification,
            NotificationTimer::new(DISPLAY_DURATION, FADE_DURATION, background.to_srgba().alpha),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(header),
                TextFont::from_font_size(10.0),
                TextColor(header_color),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(title),
                TextFont::from_font_size(14.0),
                TextColor(title_color),
                Pickable::IGNORE,
            ));

            parent.spawn((
                Text::new(description),
                TextFont::from_font_size(11.0),
                TextColor(DESCRIPTION_COLOR),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_wizard_notification(commands: &mut Commands, wizard_type: WizardType) {
    spawn_notification(
        commands,
        WIZARD_BACKGROUND_COLOR,
        WIZARD_BORDER_COLOR,
        "New Wizard Unlocked",
        WIZARD_HEADER_COLOR,
        wizard_type.display_name(),
        WIZARD_TITLE_COLOR,
        wizard_type.description(),
    );
}

fn spawn_ingredient_notification(commands: &mut Commands, ingredient: Ingredient) {
    spawn_notification(
        commands,
        INGREDIENT_BACKGROUND_COLOR,
        INGREDIENT_BORDER_COLOR,
        "Ingredient Discovered",
        INGREDIENT_HEADER_COLOR,
        ingredient.name(),
        INGREDIENT_TITLE_COLOR,
        ingredient.description(),
    );
}

fn spawn_spell_researched_notification(commands: &mut Commands, spell: Spell) {
    spawn_notification(
        commands,
        SPELL_BACKGROUND_COLOR,
        SPELL_BORDER_COLOR,
        "Spell Researched",
        SPELL_HEADER_COLOR,
        spell.display_name(),
        SPELL_TITLE_COLOR,
        spell.description(),
    );
}

fn spawn_talent_tier_notification(commands: &mut Commands, spell: Spell, tier: u8) {
    spawn_notification(
        commands,
        TALENT_BACKGROUND_COLOR,
        TALENT_BORDER_COLOR,
        format!("{} Tier Unlocked", tier_name(tier)),
        TALENT_HEADER_COLOR,
        spell.display_name(),
        TALENT_TITLE_COLOR,
        "Choose a new talent at the Wizard Tower.",
    );
}

fn spawn_combo_notification(commands: &mut Commands, name: &str, description: &str) {
    spawn_notification(
        commands,
        COMBO_BACKGROUND_COLOR,
        COMBO_BORDER_COLOR,
        "Combo Discovered",
        COMBO_HEADER_COLOR,
        name.to_string(),
        COMBO_TITLE_COLOR,
        description.to_string(),
    );
}

fn spawn_toast_notification(commands: &mut Commands, message: &str) {
    spawn_notification(
        commands,
        BACKGROUND_COLOR,
        BORDER_COLOR,
        "",
        Color::NONE,
        message.to_string(),
        DESCRIPTION_COLOR,
        "",
    );
}

/// Ticks notification timers, applies fade, and despawns expired notifications.
///
/// Uses `Time<Real>` so the popup stays legible for a fixed wall-clock duration.
/// On virtual time the between-wave staging speedup (`STAGING_SPEEDUP`, up to 10x
/// with the Game Speed setting) compressed the whole lifetime into a fraction of a
/// second — which is exactly when ingredient drops reach the wizard.
pub(super) fn update_notifications(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut notifications: Query<
        (
            Entity,
            &mut NotificationTimer,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Notification>,
    >,
    children_query: Query<&Children>,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (entity, mut timer, mut bg, mut border) in &mut notifications {
        timer.elapsed += time.delta_secs();

        if timer.is_expired() {
            commands.entity(entity).try_despawn();
            continue;
        }

        let opacity = timer.opacity();

        let mut bg_color = bg.0.to_srgba();
        bg_color.alpha = timer.base_bg_alpha * opacity;
        bg.0 = bg_color.into();

        let mut border_srgba = border.top.to_srgba();
        border_srgba.alpha = opacity;
        *border = BorderColor::all(border_srgba);

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

/// Clears in-flight notifications when a match ends.
///
/// Notifications display strictly one at a time on real time, so a burst of
/// discoveries late in a battle can still be draining when the score screen
/// opens. These entities carry no screen marker and the queue is a global
/// resource, so without this they'd render on top of the score screen and the
/// wizard tower.
pub(super) fn clear_notifications(
    mut commands: Commands,
    mut queue: ResMut<NotificationQueue>,
    notifications: Query<Entity, With<Notification>>,
) {
    queue.queue.clear();
    for entity in &notifications {
        commands.entity(entity).try_despawn();
    }
}
