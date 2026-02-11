use bevy::prelude::*;

use crate::game::units::wizard::archetypes::arcanorouter::resources::SliderType;

/// Root container for the Arcanorouter display UI
#[derive(Component)]
pub(super) struct ArcanoRouterDisplay;

/// Marker for a slider bar background (the full bar area)
#[derive(Component)]
pub(super) struct SliderBar;

/// Marker for a slider fill (the colored bar showing current allocation)
#[derive(Component)]
pub(super) struct SliderFill {
    pub slider_type: SliderType,
}
