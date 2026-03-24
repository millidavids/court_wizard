use bevy::prelude::*;

/// Preloaded image handles for the parallax background layers.
#[derive(Resource)]
pub(super) struct MenuBackgroundAssets {
    pub skybox: Handle<Image>,
    pub background: Handle<Image>,
    pub foreground: Handle<Image>,
}
