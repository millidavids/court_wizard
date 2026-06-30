//! Warglock reload hint: the bound reload key (or its controller glyph) followed
//! by the word "Reload". The key/glyph is driven by the shared
//! `adapt_button_prompts` system.

use bevy::prelude::*;

use crate::config::input_bindings::{
    BindingAction, BindingContext, InputBindings, key_display_name,
};
use crate::game::input::action_state::GamepadAction;
use crate::ui::gamepad_glyphs::{ButtonPrompt, ButtonPromptImage, PromptKey};

/// Font size of the reload key letter.
const RELOAD_KEY_FONT_SIZE: f32 = 12.0;
/// Font size of the "Reload" word.
const RELOAD_LABEL_FONT_SIZE: f32 = 11.0;
/// On-screen size of the reload controller glyph.
const RELOAD_GLYPH_SIZE: f32 = 16.0;

/// Spawns the reload hint row.
pub(crate) fn spawn_reload_prompt(parent: &mut ChildSpawnerCommands, bindings: &InputBindings) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|row| {
            let prompt = row
                .spawn((
                    Text::new(key_display_name(
                        bindings.get(BindingContext::Warglock, BindingAction::Reload),
                    )),
                    TextFont::from_font_size(RELOAD_KEY_FONT_SIZE),
                    TextColor(Color::srgba(0.85, 0.85, 0.85, 0.9)),
                    ButtonPrompt {
                        action: GamepadAction::AbilityUp,
                        key: PromptKey::Binding(BindingContext::Warglock, BindingAction::Reload),
                        glyph_px: RELOAD_GLYPH_SIZE,
                        keyboard_px: RELOAD_KEY_FONT_SIZE,
                    },
                ))
                .id();
            row.spawn((
                ImageNode::new(Handle::default()),
                Node {
                    width: Val::Px(RELOAD_GLYPH_SIZE),
                    height: Val::Px(RELOAD_GLYPH_SIZE),
                    display: Display::None,
                    ..default()
                },
                ButtonPromptImage { text: prompt },
            ));
            row.spawn((
                Text::new("Reload"),
                TextFont::from_font_size(RELOAD_LABEL_FONT_SIZE),
                TextColor(Color::srgba(0.7, 0.7, 0.7, 0.9)),
            ));
        });
}
