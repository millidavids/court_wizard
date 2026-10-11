//! Pointer-driven slider tracks. Pressing a track (or its handle) starts a drag
//! and `DragTrack::fraction` follows the pointer across the track, clamped to
//! 0..1 so dragging past either end pins the value.

use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui::ui_transform::UiGlobalTransform;

use crate::ui::components::NON_BLOCKING;

/// A slider track. Holds `Pressed` while being dragged, until
/// `clear_stale_presses` drops it on release.
#[derive(Component, Default)]
#[require(Hovered, Pickable = NON_BLOCKING)]
pub(crate) struct DragTrack {
    /// Pointer position across the track, 0 (left) to 1 (right).
    pub(crate) fraction: f32,
}

impl DragTrack {
    /// The fraction, if the pointer moved it since the caller last ran.
    /// Spawning counts as a change to Bevy, but must not move the value.
    pub(crate) fn moved(track: &Ref<Self>) -> Option<f32> {
        (track.is_changed() && !track.is_added()).then_some(track.fraction)
    }
}

/// Where `position` (window-logical) falls across the track, mirroring the UI
/// picking backend's camera scale and viewport offset.
fn fraction_at(
    position: Vec2,
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    camera: &Camera,
) -> Option<f32> {
    let mut point = position * camera.target_scaling_factor().unwrap_or(1.0);
    if let Some(viewport) = camera.physical_viewport_rect() {
        point -= viewport.min.as_vec2();
    }
    node.normalize_point(*transform, point)
        .map(|local| (local.x + 0.5).clamp(0.0, 1.0))
}

type TrackQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut DragTrack,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static ComputedUiTargetCamera,
    ),
>;

fn follow(
    entity: Entity,
    position: Vec2,
    is_press: bool,
    tracks: &mut TrackQuery,
    cameras: &Query<&Camera>,
) {
    let Ok((mut track, node, transform, target)) = tracks.get_mut(entity) else {
        return;
    };
    let Some(camera) = target.get().and_then(|e| cameras.get(e).ok()) else {
        return;
    };
    // Consumers treat any change as user input, so drags only write on a real
    // move. A press always writes: the value may have changed since the track
    // last saw the pointer (+/- buttons, controller).
    if let Some(fraction) = fraction_at(position, node, transform, camera)
        && (is_press || track.fraction != fraction)
    {
        track.fraction = fraction;
    }
}

pub(crate) fn begin_track_drag(
    mut press: On<PointerPress>,
    mut tracks: TrackQuery,
    cameras: Query<&Camera>,
    mut commands: Commands,
) {
    if press.button != PointerButton::Primary || !tracks.contains(press.entity) {
        return;
    }
    press.propagate(false);
    commands.entity(press.entity).insert(Pressed);
    follow(
        press.entity,
        press.pointer.position,
        true,
        &mut tracks,
        &cameras,
    );
}

pub(crate) fn follow_track_drag(
    mut drag: On<PointerDrag>,
    mut tracks: TrackQuery,
    cameras: Query<&Camera>,
) {
    if drag.button != PointerButton::Primary || !tracks.contains(drag.entity) {
        return;
    }
    drag.propagate(false);
    follow(
        drag.entity,
        drag.pointer.position,
        false,
        &mut tracks,
        &cameras,
    );
}
