use bevy::prelude::*;

use super::resources::SliderType;

/// Message sent when a slider should be adjusted by a delta amount.
///
/// This is sent from UI input handlers and processed by the allocation
/// update system to modify the ArcanoRouterState resource.
#[derive(Message)]
pub struct SliderAdjustMessage {
    pub slider: SliderType,
    pub delta: f32,
}
