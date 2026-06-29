use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::input::action_state::GamepadAction;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::units::wizard::archetypes::roulette::constants::SPIN_DURATION;
use crate::game::units::wizard::archetypes::roulette::resources::{RoulettePhase, RouletteState};
use crate::game::units::wizard::components::Spell;
use crate::ui::gamepad_glyphs::{
    CurrentControllerGlyphStyle, GamepadGlyphFonts, SteamGlyphs, glyph_char,
};

/// Returns the display name for a spell with newlines replaced by spaces.
fn spell_display_name(spell: &Spell) -> String {
    spell.name().replace('\n', " ")
}

/// Spawns the roulette wheel as a UI image node with UiTransform for rotation.
pub(crate) fn spawn_roulette_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the roulette wheel image
    let wheel_texture: Handle<Image> = asset_server.load("images/roulette.png");

    // Root container - absolute positioned at bottom center
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_MARGIN),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            OnGameplayScreen,
            ArchetypeUI,
            RouletteDisplayRoot,
        ))
        .with_children(|parent| {
            // Selected spell text (above wheel)
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(SELECTED_SPELL_FONT_SIZE),
                TextColor(SELECTED_SPELL_COLOR),
                RouletteSelectedText,
            ));

            // The spinning wheel image with UiTransform for rotation
            parent.spawn((
                ImageNode::new(wheel_texture),
                Node {
                    width: Val::Px(WHEEL_RADIUS * 2.0),
                    height: Val::Px(WHEEL_RADIUS * 2.0),
                    ..default()
                },
                UiTransform::default(),
                RouletteWheelMesh,
            ));

            // Triangle indicator pointing up
            parent.spawn((
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    border: UiRect {
                        left: Val::Px(10.0),
                        right: Val::Px(10.0),
                        top: Val::Px(15.0),
                        bottom: Val::Px(0.0),
                    },
                    margin: UiRect::top(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor {
                    left: Color::NONE,
                    right: Color::NONE,
                    top: POINTER_COLOR,
                    bottom: Color::NONE,
                },
                RoulettePointer,
            ));

            // Prompt text (below wheel)
            parent.spawn((
                Text::new("Press SPACE to spin"),
                TextFont::from_font_size(PROMPT_FONT_SIZE),
                TextColor(PROMPT_COLOR),
                RoulettePromptText,
            ));
            // Hidden image, shown in place of the text glyph when an official
            // Steam Input glyph is available for the spin action.
            parent.spawn((
                ImageNode::new(Handle::default()),
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    display: Display::None,
                    ..default()
                },
                RoulettePromptGlyphImage,
            ));
        });
}

/// Updates text based on roulette state.
pub(super) fn update_roulette_display(
    roulette_state: Res<RouletteState>,
    mut selected_text_query: Query<
        &mut Text,
        (With<RouletteSelectedText>, Without<RoulettePromptText>),
    >,
    mut prompt_query: Query<
        (&mut Text, &mut TextColor),
        (With<RoulettePromptText>, Without<RouletteSelectedText>),
    >,
    mut commands: Commands,
    selected_text_entity_query: Query<
        (Entity, Option<&SelectedSpellFadeTimer>),
        With<RouletteSelectedText>,
    >,
) {
    if !roulette_state.is_changed() {
        return;
    }

    match &roulette_state.phase {
        RoulettePhase::Idle => {
            if let Ok((_, fade_timer)) = selected_text_entity_query.single()
                && fade_timer.is_none()
                && let Ok(mut text) = selected_text_query.single_mut()
            {
                **text = "".to_string();
            }

            if let Ok((mut text, mut color)) = prompt_query.single_mut() {
                **text = "Press SPACE to spin".to_string();
                color.0 = PROMPT_COLOR;
            }
        }
        RoulettePhase::Spinning { .. } => {
            if let Ok((entity, fade_timer)) = selected_text_entity_query.single()
                && fade_timer.is_some()
            {
                commands.entity(entity).remove::<SelectedSpellFadeTimer>();
            }
            if let Ok(mut text) = selected_text_query.single_mut() {
                **text = "".to_string();
            }

            if let Ok((mut text, mut color)) = prompt_query.single_mut() {
                **text = "Spinning...".to_string();
                color.0 = SELECTED_SPELL_COLOR;
            }
        }
        RoulettePhase::Selected { spell } => {
            if let Ok(mut text) = selected_text_query.single_mut() {
                **text = spell_display_name(spell);
            }
            if let Ok((entity, fade_timer)) = selected_text_entity_query.single()
                && fade_timer.is_none()
            {
                commands.entity(entity).insert(SelectedSpellFadeTimer {
                    elapsed: 0.0,
                    duration: SELECTED_FADE_DURATION,
                });
            }

            if let Ok((mut text, mut color)) = prompt_query.single_mut() {
                **text = "Cast your spell!".to_string();
                color.0 = SELECTED_SPELL_COLOR;
            }
        }
    }
}

/// Animates the wheel rotation during spinning using UiTransform.
pub(super) fn animate_wheel_spin(
    time: Res<Time>,
    roulette_state: Res<RouletteState>,
    mut wheel_query: Query<&mut UiTransform, With<RouletteWheelMesh>>,
) {
    if let Ok(mut ui_transform) = wheel_query.single_mut() {
        match &roulette_state.phase {
            RoulettePhase::Spinning { elapsed, .. } => {
                // Calculate rotation speed with easing (fast at start, slow at end)
                let progress = (*elapsed / SPIN_DURATION).min(1.0);
                let speed = 20.0 * (1.0 - progress * progress); // Quadratic easing

                // Accumulate rotation (clockwise in radians)
                let delta_angle = speed * time.delta_secs();
                let current_angle = ui_transform.rotation.as_radians();
                ui_transform.rotation = Rot2::radians(current_angle + delta_angle);
            }
            RoulettePhase::Selected { .. } => {
                // Keep the final rotation
            }
            RoulettePhase::Idle => {
                // Reset to no rotation
                ui_transform.rotation = Rot2::IDENTITY;
            }
        }
    }
}

/// Fades the selected spell name text over time.
pub(super) fn update_selected_spell_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut fade_query: Query<
        (Entity, &mut SelectedSpellFadeTimer, &mut TextColor),
        With<RouletteSelectedText>,
    >,
    mut text_query: Query<&mut Text, With<RouletteSelectedText>>,
) {
    for (entity, mut timer, mut color) in &mut fade_query {
        timer.elapsed += time.delta_secs();

        let alpha = (1.0 - (timer.elapsed / timer.duration)).max(0.0);
        color.0 = SELECTED_SPELL_COLOR.with_alpha(alpha);

        if timer.elapsed >= timer.duration {
            commands.entity(entity).remove::<SelectedSpellFadeTimer>();
            color.0 = SELECTED_SPELL_COLOR;

            if let Ok(mut text) = text_query.single_mut() {
                **text = "".to_string();
            }
        }
    }
}

/// Renders the spin prompt. While **idle**: the bound D-pad-Up glyph (Steam art
/// when available, else the Kenney font) on a controller, or "Press SPACE to
/// spin" on mouse/keyboard. During non-idle phases (`update_roulette_display`
/// owns the "Spinning…"/"Cast…" status text) the glyph image is hidden and the
/// text shown in the default font. Runs every frame but bails cheaply when
/// nothing changed.
#[allow(clippy::type_complexity)]
pub(super) fn adapt_prompt_to_input_device(
    roulette_state: Res<RouletteState>,
    active: Res<ActiveInputDevice>,
    style: Res<CurrentControllerGlyphStyle>,
    steam: Res<SteamGlyphs>,
    fonts: Option<Res<GamepadGlyphFonts>>,
    mut prompt_query: Query<
        (&mut Text, &mut TextFont, &mut Node),
        (With<RoulettePromptText>, Without<RoulettePromptGlyphImage>),
    >,
    mut image_query: Query<
        (&mut ImageNode, &mut Node),
        (With<RoulettePromptGlyphImage>, Without<RoulettePromptText>),
    >,
) {
    let idle = matches!(roulette_state.phase, RoulettePhase::Idle);
    let gamepad = active.is_gamepad();
    // Spin = D-pad Up (`AbilityUp`); the Steam image only stands in for the idle prompt.
    let steam_glyph = (idle && gamepad)
        .then(|| steam.get(GamepadAction::AbilityUp))
        .flatten();

    // The image is visible iff the idle prompt is showing a Steam glyph; hidden
    // in every other case (mouse/keyboard, Kenney fallback, or non-idle phases).
    if let Ok((mut image_node, mut img_style)) = image_query.single_mut() {
        match &steam_glyph {
            Some(handle) => {
                if image_node.image != *handle {
                    image_node.image = handle.clone();
                }
                if img_style.display != Display::Flex {
                    img_style.display = Display::Flex;
                }
            }
            None => {
                if img_style.display != Display::None {
                    img_style.display = Display::None;
                }
            }
        }
    }

    let Ok((mut text, mut font, mut text_node)) = prompt_query.single_mut() else {
        return;
    };

    // Text is hidden exactly when the Steam image is showing.
    let want_display = if steam_glyph.is_some() {
        Display::None
    } else {
        Display::Flex
    };
    if text_node.display != want_display {
        text_node.display = want_display;
    }
    if steam_glyph.is_some() {
        return;
    }

    // Non-idle: `update_roulette_display` owns the status text; just ensure the
    // font is the default (not a leftover glyph font) and leave the content.
    if !idle {
        if font.font != Handle::<Font>::default() {
            font.font = Handle::<Font>::default();
            font.font_size = PROMPT_FONT_SIZE;
        }
        return;
    }

    if gamepad && let Some(fonts) = fonts {
        if let Some(glyph) = glyph_char(GamepadButton::DPadUp, style.0) {
            let glyph_str = glyph.to_string();
            if **text != glyph_str {
                **text = glyph_str;
            }
            let want_font = fonts.font_for(style.0);
            // Match the 40px Steam image so the two paths don't visually jump.
            let want_size = 40.0;
            if font.font != want_font || font.font_size != want_size {
                font.font = want_font;
                font.font_size = want_size;
            }
        }
    } else {
        let default_text = "Press SPACE to spin";
        if **text != default_text {
            **text = default_text.to_string();
        }
        if font.font != Handle::<Font>::default() {
            font.font = Handle::<Font>::default();
            font.font_size = PROMPT_FONT_SIZE;
        }
    }
}
