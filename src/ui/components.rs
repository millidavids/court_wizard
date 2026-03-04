//! Shared UI components used across all menus and screens.

use std::collections::HashMap;

use bevy::asset::AssetId;
use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Replaces Bevy's built-in default font (FiraMono) with PressStart2P.
///
/// After this runs, every `TextFont::default()` and `TextFont::from_font_size()`
/// automatically uses PressStart2P without needing an explicit font handle.
pub fn set_default_font(mut fonts: ResMut<Assets<Font>>) {
    let font_data = include_bytes!("../../assets/fonts/PressStart2P-Regular.ttf");
    let font =
        Font::try_from_bytes(font_data.to_vec()).expect("Failed to load PressStart2P-Regular.ttf");
    fonts
        .insert(AssetId::default(), font)
        .expect("Failed to insert default font");
}

/// Stores the original colors for a button, used to compute hover/pressed states.
///
/// Attach this to any `Button` entity to enable shared hover/pressed visual feedback
/// via the `button_interaction` system.
#[derive(Component)]
pub struct ButtonColors {
    /// The button's background color in its default state.
    pub background: Color,
    /// The button's border color in its default state.
    pub border: Color,
}

/// Pre-loaded spell icon image handles.
///
/// Loaded at startup so spell icons display instantly in the spell book and action bar.
#[derive(Resource)]
pub struct SpellIconAssets {
    icons: HashMap<Spell, Handle<Image>>,
}

impl SpellIconAssets {
    /// Returns the pre-loaded icon handle for a spell, if it has one.
    pub fn get(&self, spell: &Spell) -> Option<&Handle<Image>> {
        self.icons.get(spell)
    }
}

/// Loads all spell icon images at startup.
pub fn load_spell_icon_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut icons = HashMap::new();
    for spell in Spell::all() {
        if let Some(path) = spell.icon_path() {
            icons.insert(*spell, asset_server.load(path));
        }
    }
    commands.insert_resource(SpellIconAssets { icons });
}

/// Shared marker component for back buttons across menu screens.
///
/// Used by `handle_back_to_landing` in `ui::systems` to navigate back to the landing page.
#[derive(Component)]
pub struct BackButton;

/// Configuration for button dimensions and styling.
///
/// Pass this to `spawn_button` to control button size, font, and colors.
/// Each screen can define its own `ButtonStyle` constant.
pub struct ButtonStyle {
    /// Button width in pixels.
    pub width: f32,
    /// Button height in pixels.
    pub height: f32,
    /// Border width in pixels.
    pub border_width: f32,
    /// Font size for button text.
    pub font_size: f32,
    /// Background color.
    pub background: Color,
    /// Border color.
    pub border: Color,
    /// Text color.
    pub text_color: Color,
}
