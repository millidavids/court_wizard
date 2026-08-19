//! Reusable controller-glyph button prompts, shared by every HUD that hints a
//! special-ability button (runes, weather, sliders, reload, …).
//!
//! A prompt is two sibling UI entities — a [`ButtonPrompt`] on a `Text` node and
//! a [`ButtonPromptImage`] on an `ImageNode` linked back to it. The single
//! [`adapt_button_prompts`] system keeps them in sync with the active input
//! device: the official Steam glyph image on a controller when one is loaded,
//! else the Kenney font glyph, else the bound keyboard key.

use bevy::prelude::*;

use super::resources::{CurrentControllerGlyphStyle, GamepadGlyphFonts, SteamGlyphs, glyph_char};
use crate::config::ControllerGlyphStyle;
use crate::config::input_bindings::{
    BindingAction, BindingContext, InputBindings, key_display_name,
};
use crate::game::input::action_state::GamepadAction;
use crate::game::input::gamepad::resources::ActiveInputDevice;

/// The shared input state for a frame's worth of prompt updates.
pub(crate) struct GlyphContext<'a> {
    pub gamepad: bool,
    pub style: ControllerGlyphStyle,
    pub fonts: Option<&'a GamepadGlyphFonts>,
    pub steam: &'a SteamGlyphs,
}

impl<'a> GlyphContext<'a> {
    /// Builds the context from the resources every adapt system reads.
    pub(crate) fn from_res(
        active: &ActiveInputDevice,
        style: &CurrentControllerGlyphStyle,
        fonts: Option<&'a GamepadGlyphFonts>,
        steam: &'a SteamGlyphs,
    ) -> Self {
        Self {
            gamepad: active.is_gamepad(),
            style: style.0,
            fonts,
            steam,
        }
    }
}

/// Where a prompt's keyboard label comes from.
#[derive(Clone, Copy)]
pub(crate) enum PromptKey {
    /// A rebindable key, resolved live from [`InputBindings`] so the label always
    /// tracks the player's current binding.
    Binding(BindingContext, BindingAction),
    /// A fixed label that never changes.
    Static(&'static str),
}

/// The text half of a button prompt: shows the bound keyboard key on
/// mouse/keyboard, or the controller glyph for `action` on a pad. Pair it with a
/// sibling [`ButtonPromptImage`] (linked to this entity) for the Steam image.
#[derive(Component, Clone, Copy)]
pub(crate) struct ButtonPrompt {
    pub action: GamepadAction,
    pub key: PromptKey,
    /// On-screen size of the controller glyph (image + Kenney font).
    pub glyph_px: f32,
    /// Font size of the keyboard key label.
    pub keyboard_px: f32,
}

/// The image half of a button prompt, linked back to its [`ButtonPrompt`] entity.
#[derive(Component, Clone, Copy)]
pub(crate) struct ButtonPromptImage {
    pub text: Entity,
}

/// Keeps every [`ButtonPrompt`] in sync with the active input device. One generic
/// system for all archetypes — each HUD only spawns the component pair.
pub(crate) fn adapt_button_prompts(
    active: Res<ActiveInputDevice>,
    style: Res<CurrentControllerGlyphStyle>,
    steam: Res<SteamGlyphs>,
    fonts: Option<Res<GamepadGlyphFonts>>,
    bindings: Res<InputBindings>,
    mut texts: Query<
        (&ButtonPrompt, &mut Text, &mut TextFont, &mut Node),
        Without<ButtonPromptImage>,
    >,
    mut images: Query<(&ButtonPromptImage, &mut ImageNode, &mut Node), Without<ButtonPrompt>>,
) {
    let ctx = GlyphContext::from_res(&active, &style, fonts.as_deref(), &steam);
    for (link, image, image_node) in &mut images {
        let Ok((prompt, text, font, text_node)) = texts.get_mut(link.text) else {
            continue;
        };
        let keyboard_label = match prompt.key {
            PromptKey::Static(s) => s,
            PromptKey::Binding(context, action) => key_display_name(bindings.get(context, action)),
        };
        apply_button_glyph(
            &ctx,
            prompt.action,
            keyboard_label,
            prompt.glyph_px,
            prompt.keyboard_px,
            text,
            font,
            text_node,
            image,
            image_node,
        );
    }
}

/// Renders one button prompt across a `Text` node and a sibling `ImageNode`.
/// Exactly one of the two is shown: the Steam image on a controller with a loaded
/// glyph, else the Kenney font glyph, else `keyboard_label` in the default font.
///
/// Takes the `Mut<T>` query items by value (not `&mut T`) so change-detection
/// only fires when a value actually changes — passing `&mut T` would deref-mut
/// the `Mut` at the call site and tick every node every frame.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_button_glyph(
    ctx: &GlyphContext,
    action: GamepadAction,
    keyboard_label: &str,
    glyph_px: f32,
    keyboard_px: f32,
    mut text: Mut<Text>,
    mut text_font: Mut<TextFont>,
    mut text_node: Mut<Node>,
    mut image: Mut<ImageNode>,
    mut image_node: Mut<Node>,
) {
    // Directional D-pad abilities: a bold arrow reads far better than a d-pad
    // glyph. Steam's and Kenney's d-pad glyphs are both near-identical crosses
    // with one subtly-filled arm — indistinguishable at button size. An arrow
    // doesn't depend on Steam, the controller type, or the Kenney font.
    if ctx.gamepad
        && let Some(arrow) = action.direction_arrow()
    {
        if **text != *arrow {
            **text = arrow.to_string();
        }
        set_font(&mut text_font, default_font_source(), glyph_px);
        set_display(&mut image_node, Display::None);
        set_display(&mut text_node, Display::Flex);
        return;
    }

    // Steam image when on a controller and a glyph is loaded for this action.
    if let Some(handle) = ctx.gamepad.then(|| ctx.steam.get(action)).flatten() {
        if image.image != handle {
            image.image = handle;
        }
        set_display(&mut image_node, Display::Flex);
        set_display(&mut text_node, Display::None);
        return;
    }

    // Otherwise the text carries the prompt; the image is hidden.
    set_display(&mut image_node, Display::None);
    set_display(&mut text_node, Display::Flex);

    if ctx.gamepad
        && let Some(fonts) = ctx.fonts
        && let Some(ch) = glyph_char(action.default_button(), ctx.style)
    {
        let mut buf = [0u8; 4];
        let glyph = ch.encode_utf8(&mut buf);
        if **text != *glyph {
            **text = glyph.to_string();
        }
        set_font(&mut text_font, fonts.font_for(ctx.style).into(), glyph_px);
    } else {
        if **text != *keyboard_label {
            **text = keyboard_label.to_string();
        }
        set_font(&mut text_font, default_font_source(), keyboard_px);
    }
}

/// The game's default font — PressStart2P, swapped in at `AssetId::default()`
/// by `ui::components::set_default_font`.
fn default_font_source() -> FontSource {
    FontSource::Handle(Handle::default())
}

/// Writes font + size only when they differ, so `TextFont` isn't marked changed
/// (and the text re-laid out) on every frame a prompt is on screen.
fn set_font(text_font: &mut Mut<TextFont>, font: FontSource, size_px: f32) {
    let size = FontSize::Px(size_px);
    if text_font.font != font || text_font.font_size != size {
        text_font.font = font;
        text_font.font_size = size;
    }
}

fn set_display(node: &mut Mut<Node>, display: Display) {
    if node.display != display {
        node.display = display;
    }
}
