//! Shared UI components used across all menus and screens.

use std::collections::HashMap;

use bevy::asset::AssetId;
use bevy::prelude::*;

use crate::game::units::UnitType;
use crate::game::units::components::CompendiumSpriteSpec;
use crate::game::units::wizard::components::Spell;

/// Replaces Bevy's built-in default font (FiraMono) with PressStart2P.
///
/// After this runs, every `TextFont::default()` and `TextFont::from_font_size()`
/// automatically uses PressStart2P without needing an explicit font handle.
///
/// Must stay in `Startup`. Bevy registers each `Font` asset with Parley the
/// first time it sees that asset id, and never again — so replacing the asset at
/// `AssetId::default()` only takes effect if it happens before the first
/// `PostUpdate`. Move this into a state transition and every `TextFont` silently
/// falls back to Bevy's embedded FiraMono.
pub fn set_default_font(mut fonts: ResMut<Assets<Font>>) {
    let font_data = include_bytes!("../../assets/fonts/PressStart2P-Regular.ttf");
    fonts
        .insert(AssetId::default(), Font::from_bytes(font_data.to_vec()))
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

/// Pre-loaded gun icon image handles for the Warglock action bar.
#[derive(Resource)]
pub struct GunIconAssets {
    icons: HashMap<crate::game::units::wizard::archetypes::gunslinger::GunType, Handle<Image>>,
}

impl GunIconAssets {
    pub fn get(
        &self,
        gun: &crate::game::units::wizard::archetypes::gunslinger::GunType,
    ) -> Option<&Handle<Image>> {
        self.icons.get(gun)
    }
}

pub fn load_gun_icon_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    use crate::game::units::wizard::archetypes::gunslinger::GunType;
    let mut icons = HashMap::new();
    for gun in GunType::all() {
        let path = match gun {
            GunType::MachineGun => "images/icons/assault_rifle_icon.png",
            GunType::Magnum => "images/icons/magnum_icon.png",
            GunType::Shotgun => "images/icons/shotgun_icon.png",
            GunType::RocketLauncher => "images/icons/rocket_launcher_icon.png",
            GunType::Flamethrower => "images/icons/flame_thrower_icon.png",
        };
        icons.insert(gun, asset_server.load(path));
    }
    commands.insert_resource(GunIconAssets { icons });
}

/// Pre-loaded unit portrait handles for the compendium detail panel.
///
/// Holds an `Image` handle per unit, plus a `TextureAtlasLayout` handle for
/// units whose portrait is a sprite-sheet frame (everything except the boss
/// static portraits).
#[derive(Resource)]
pub struct UnitCompendiumSpriteAssets {
    pub images: HashMap<UnitType, Handle<Image>>,
    pub layouts: HashMap<UnitType, Handle<TextureAtlasLayout>>,
}

pub fn load_unit_compendium_sprite_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut images = HashMap::new();
    let mut layouts = HashMap::new();
    for unit in UnitType::all() {
        match unit.compendium_sprite() {
            CompendiumSpriteSpec::Static { path, .. } => {
                images.insert(*unit, asset_server.load(path));
            }
            CompendiumSpriteSpec::Atlas {
                path,
                sheet_size,
                frame_px,
                ..
            } => {
                images.insert(*unit, asset_server.load(path));
                let layout = TextureAtlasLayout::from_grid(
                    UVec2::splat(frame_px),
                    sheet_size.0 / frame_px,
                    sheet_size.1 / frame_px,
                    None,
                    None,
                );
                layouts.insert(*unit, atlas_layouts.add(layout));
            }
        }
    }
    commands.insert_resource(UnitCompendiumSpriteAssets { images, layouts });
}

/// Tracks the 3D push animation state for a button's front face.
///
/// The front face slides up/down relative to the edge layer to create
/// a physical "pushable" illusion. `current` lerps toward `target` each frame.
#[derive(Component)]
pub struct ButtonAnimState {
    /// Current Y offset of the front face (pixels, negative = raised).
    pub current: f32,
    /// Target Y offset based on interaction state.
    pub target: f32,
}

/// Marker for the front face child of a 3D button.
#[derive(Component)]
pub struct ButtonFront;

/// Marker for the edge (depth) child of a 3D button.
#[derive(Component)]
pub struct ButtonEdge;

/// Marker for a button that should stay in a permanent "pressed" visual state.
/// Used for active tabs, selected options, and other toggle-like buttons.
#[derive(Component)]
pub struct ButtonActive;

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
    /// Whether to render a drop shadow behind button text.
    pub text_shadow: bool,
}
