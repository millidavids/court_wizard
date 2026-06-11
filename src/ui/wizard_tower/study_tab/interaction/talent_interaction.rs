use bevy::prelude::*;

use super::super::super::components::*;
use super::super::super::constants::*;

/// Handles clicks on talent cards to select/deselect talents. Updates
/// `ButtonActive` markers on the affected tier's cards in place rather than
/// rebuilding the detail panel — rebuilding would despawn the focused card
/// and snap focus back to the first card in tier 1.
pub(crate) fn handle_talent_card_clicks(
    mut commands: Commands,
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    card_query: Query<&TalentCard>,
    all_cards_query: Query<(Entity, &TalentCard)>,
) {
    use crate::config::save_data::{get_spell_talent_progress, set_spell_talent_selection};
    use crate::game::units::wizard::talents::constants as talent_consts;

    for event in button_clicked.read() {
        let Ok(card) = card_query.get(event.button) else {
            continue;
        };

        let talent_progress = get_spell_talent_progress(card.spell);
        let thresholds = talent_consts::tier_thresholds(card.spell);

        if talent_progress < thresholds[card.tier as usize] {
            continue;
        }

        let current = crate::config::save_data::get_spell_talent_selections(card.spell);
        let new_choice = if current[card.tier as usize] == Some(card.choice) {
            None
        } else {
            Some(card.choice)
        };

        set_spell_talent_selection(card.spell, card.tier as usize, new_choice);

        // Toggle ButtonActive on each sibling card in the same spell+tier so
        // exactly the selected card is marked active.
        for (entity, other) in &all_cards_query {
            if other.spell != card.spell || other.tier != card.tier {
                continue;
            }
            if new_choice == Some(other.choice) {
                commands
                    .entity(entity)
                    .insert(crate::ui::components::ButtonActive);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui::components::ButtonActive>();
            }
        }
    }
}

/// Updates the talent description text when hovering over talent cards.
pub(crate) fn update_talent_hover_description(
    interaction_query: Query<(&Interaction, &TalentCard), Changed<Interaction>>,
    mut desc_query: Query<(&mut Text, &mut TextFont, &mut TextColor), With<TalentDescriptionText>>,
) {
    use crate::config::save_data::get_spell_talent_progress;
    use crate::game::units::wizard::talents::{constants as talent_consts, definitions};

    for (interaction, card) in &interaction_query {
        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        let talent_progress = get_spell_talent_progress(card.spell);
        let thresholds = talent_consts::tier_thresholds(card.spell);
        let tier_unlocked = talent_progress >= thresholds[card.tier as usize];
        let defs = definitions::talent_definitions(card.spell);
        let def = &defs[card.tier as usize][card.choice as usize];

        for (mut text, mut font, mut color) in &mut desc_query {
            if tier_unlocked {
                let desc = format!("{}: {}", def.name, def.description);
                let font_size = if desc.len() > DESC_SHRINK_THRESHOLD {
                    TALENT_DESC_FONT_SMALL
                } else {
                    TALENT_DESC_FONT
                };
                *text = Text::new(desc);
                *font = TextFont::from_font_size(font_size);
                *color = TextColor(TEXT_COLOR);
            } else {
                let font_size = if def.locked_text.len() > DESC_SHRINK_THRESHOLD {
                    TALENT_DESC_FONT_SMALL
                } else {
                    TALENT_DESC_FONT
                };
                *text = Text::new(def.locked_text);
                *font = TextFont::from_font_size(font_size);
                *color = TextColor(LOCKED_TEXT_COLOR);
            }
        }
    }
}

/// Clears the talent description text when not hovering any talent card.
pub(crate) fn clear_talent_hover_description(
    interaction_query: Query<&Interaction, (With<TalentCard>, Changed<Interaction>)>,
    mut desc_query: Query<&mut Text, With<TalentDescriptionText>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::None {
            // Only clear if no other card is hovered
            let any_hovered = interaction_query
                .iter()
                .any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);
            if !any_hovered {
                for mut text in &mut desc_query {
                    *text = Text::new("");
                }
            }
        }
    }
}
