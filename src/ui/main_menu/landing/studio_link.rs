//! Bottom-right Blackhearth studio logo that links to the studio website.

use bevy::prelude::*;

use crate::ui::button_systems::{PointerState, PointerVisual, PointerVisualChanges};
use crate::ui::components::UiButton;
use crate::ui::link_button::{LinkButton, link_glow_for};

use super::components::OnLandingScreen;

const STUDIO_URL: &str = "https://blackhearthgames.com/";
const STUDIO_LOGO_PATH: &str = "images/logos/blackhearth_logo.png";
const STUDIO_ICON_WIDTH: f32 = 101.0;
const STUDIO_ICON_HEIGHT: f32 = 45.0;

const TINT_REST: Color = Color::srgb(1.0, 1.0, 1.0);
const TINT_HOVER: Color = Color::srgb(1.4, 1.4, 1.4);
const TINT_PRESSED: Color = Color::srgb(1.9, 1.9, 1.9);

#[derive(Component)]
pub(super) struct StudioLinkButton;

pub(super) fn setup_studio_link(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        UiButton,
        ImageNode::new(asset_server.load(STUDIO_LOGO_PATH)).with_color(TINT_REST),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(STUDIO_ICON_WIDTH),
            height: Val::Px(STUDIO_ICON_HEIGHT),
            ..default()
        },
        BoxShadow(vec![]),
        StudioLinkButton,
        LinkButton(STUDIO_URL),
        OnLandingScreen,
    ));
}

pub(super) fn update_studio_link_visuals(
    mut changes: PointerVisualChanges,
    mut q: Query<(PointerState, &mut ImageNode, &mut BoxShadow), With<StudioLinkButton>>,
) {
    for entity in changes.entities() {
        let Ok((state, mut image, mut shadow)) = q.get_mut(entity) else {
            continue;
        };
        let visual = PointerVisual::of(state);
        image.color = match visual {
            PointerVisual::Press => TINT_PRESSED,
            PointerVisual::Hover => TINT_HOVER,
            PointerVisual::Rest => TINT_REST,
        };
        shadow.0 = link_glow_for(visual);
    }
}
