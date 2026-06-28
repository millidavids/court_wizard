use bevy::prelude::*;
use bevy::ui::ui_transform::UiGlobalTransform;

use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::gamepad::resources::{ActiveInputDevice, GamepadAimSettings};
use crate::game::input::gamepad::systems::shape_stick;
use crate::game::input::messages::MouseClicked;

use super::super::super::components::*;
use super::super::super::constants::*;
use super::super::panels::*;

/// Cursor speed at full right-stick deflection, in logical pixels per second.
const STUDY_CURSOR_SPEED: f32 = 900.0;

/// Reticle side length in logical pixels when idle.
const STUDY_CURSOR_SIZE: f32 = 24.0;

/// Reticle side length in logical pixels when a node is within click range.
const STUDY_CURSOR_HOVER_SIZE: f32 = 36.0;

/// Distance (physical pixels) from cursor center to a node's center within
/// which a press of A activates that node.
const STUDY_CURSOR_HOVER_RADIUS: f32 = 44.0;

const STUDY_CURSOR_IDLE_BORDER: Color = Color::hsla(270.0, 0.70, 0.75, 0.85);
const STUDY_CURSOR_HOVER_BORDER: Color = Color::hsla(48.0, 0.95, 0.65, 1.0);
const STUDY_CURSOR_IDLE_BG: Color = Color::hsla(270.0, 0.40, 0.40, 0.18);
const STUDY_CURSOR_HOVER_BG: Color = Color::hsla(48.0, 0.85, 0.55, 0.28);

/// Cursor state for the Study tab. Position is in logical pixels measured
/// from the `SpellGraphArea` top-left. `hovered` tracks the current graph
/// node within click radius (spell or insight).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct StudyCursorMode {
    pub position: Vec2,
    pub hovered: Option<Entity>,
}

#[derive(Component)]
pub(crate) struct StudyCursorReticle;

/// Spawns the cursor reticle inside the graph area and initialises cursor
/// resources the first frame a `SpellGraphArea` appears. Removed when the
/// graph area is despawned (tab change) via `cleanup_study_cursor`.
pub(crate) fn spawn_study_cursor_on_area_added(
    mut commands: Commands,
    new_area: Query<Entity, Added<SpellGraphArea>>,
    graph_area_node: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    for area in &new_area {
        commands.entity(area).with_children(|a| {
            a.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(STUDY_CURSOR_SIZE),
                    height: Val::Px(STUDY_CURSOR_SIZE),
                    left: Val::Px(-STUDY_CURSOR_SIZE * 0.5),
                    top: Val::Px(-STUDY_CURSOR_SIZE * 0.5),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(STUDY_CURSOR_IDLE_BG),
                BorderColor::all(STUDY_CURSOR_IDLE_BORDER),
                ZIndex(1000),
                StudyCursorReticle,
                // Let mouse clicks pass through the reticle to the spell
                // nodes beneath it.
                Pickable::IGNORE,
            ));
        });
        // Initialise cursor at panel center if we have the size; otherwise
        // the first update_study_cursor tick will fix it on clamp.
        let center = graph_area_node
            .get(area)
            .ok()
            .map(|n| n.size() * n.inverse_scale_factor() * 0.5)
            .unwrap_or(Vec2::ZERO);
        commands.insert_resource(StudyCursorMode {
            position: center,
            hovered: None,
        });
        commands.init_resource::<crate::ui::focus::FocusNavInhibit>();
    }
}

/// Reads the right stick and moves `StudyCursorMode.position` within the
/// logical bounds of the graph area. Clamps so the reticle can't leave the
/// panel.
pub(crate) fn update_study_cursor(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    state: Res<GamepadActionState>,
    aim: Res<GamepadAimSettings>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
    mut cursor: ResMut<StudyCursorMode>,
) {
    if !active.is_gamepad() {
        return;
    }
    let Ok(area_node) = graph_area.single() else {
        return;
    };

    let shaped = shape_stick(state.left_stick, &aim);
    let delta = shaped * STUDY_CURSOR_SPEED * time.delta_secs();
    if delta == Vec2::ZERO && !cursor.is_added() {
        return;
    }
    cursor.position += delta;

    let inv_sf = area_node.inverse_scale_factor();
    let size_logical = area_node.size() * inv_sf;
    cursor.position.x = cursor.position.x.clamp(0.0, size_logical.x);
    cursor.position.y = cursor.position.y.clamp(0.0, size_logical.y);
}

/// Writes `StudyCursorMode.position` into the reticle's `Node.left`/`top`
/// whenever the cursor moves. Skips when nothing changed to avoid dirtying
/// the UI layout every frame while the stick is idle.
pub(crate) fn sync_study_cursor_visual(
    cursor: Res<StudyCursorMode>,
    mut reticle: Query<&mut Node, With<StudyCursorReticle>>,
) {
    if !cursor.is_changed() {
        return;
    }
    let Ok(mut node) = reticle.single_mut() else {
        return;
    };
    let half = match node.width {
        Val::Px(w) => w * 0.5,
        _ => STUDY_CURSOR_SIZE * 0.5,
    };
    node.left = Val::Px(cursor.position.x - half);
    node.top = Val::Px(cursor.position.y - half);
}

/// Finds the nearest spell / insight node within `STUDY_CURSOR_HOVER_RADIUS`
/// of the reticle and caches it in `StudyCursorMode.hovered`.
#[allow(clippy::type_complexity)]
pub(crate) fn detect_study_cursor_hover(
    reticle: Query<&UiGlobalTransform, With<StudyCursorReticle>>,
    nodes: Query<
        (Entity, &UiGlobalTransform, Option<&InheritedVisibility>),
        Or<(With<SpellGraphNode>, With<InsightBonusNode>)>,
    >,
    mut cursor: ResMut<StudyCursorMode>,
) {
    let Ok(reticle_xform) = reticle.single() else {
        return;
    };
    let reticle_pos = reticle_xform.translation;

    let mut best: Option<(Entity, f32)> = None;
    for (entity, xform, vis) in &nodes {
        if !vis.map(|v| v.get()).unwrap_or(true) {
            continue;
        }
        let dist = (xform.translation - reticle_pos).length();
        if dist <= STUDY_CURSOR_HOVER_RADIUS
            && best.as_ref().map(|(_, d)| dist < *d).unwrap_or(true)
        {
            best = Some((entity, dist));
        }
    }
    let new_hover = best.map(|(e, _)| e);
    if cursor.hovered != new_hover {
        cursor.hovered = new_hover;
    }
}

/// Animates the reticle's size and colors between idle and hover states, and
/// hides the reticle whenever cursor mode is off (detail panel open). The
/// "cursor mode active" signal is the presence of `FocusNavInhibit`.
pub(crate) fn update_reticle_appearance(
    cursor: Res<StudyCursorMode>,
    inhibit: Option<Res<crate::ui::focus::FocusNavInhibit>>,
    active_input: Res<ActiveInputDevice>,
    mut reticle: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
        With<StudyCursorReticle>,
    >,
) {
    let Ok((mut node, mut bg, mut border)) = reticle.single_mut() else {
        return;
    };
    // Reticle is a controller affordance only — when the player is on mouse
    // + keyboard the OS cursor is the input, so the reticle would just be
    // visual clutter.
    if inhibit.is_none() || !active_input.is_gamepad() {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;
    let (size, bg_color, border_color) = if cursor.hovered.is_some() {
        (
            STUDY_CURSOR_HOVER_SIZE,
            STUDY_CURSOR_HOVER_BG,
            STUDY_CURSOR_HOVER_BORDER,
        )
    } else {
        (
            STUDY_CURSOR_SIZE,
            STUDY_CURSOR_IDLE_BG,
            STUDY_CURSOR_IDLE_BORDER,
        )
    };
    node.width = Val::Px(size);
    node.height = Val::Px(size);
    bg.set_if_neq(BackgroundColor(bg_color));
    border.set_if_neq(BorderColor::all(border_color));
}

/// Emits a `MouseClicked` on the hovered node when the A / South button is
/// just-pressed. The existing `handle_graph_node_clicks` system consumes it.
pub(crate) fn study_cursor_confirm(
    cursor: Res<StudyCursorMode>,
    state: Res<GamepadActionState>,
    mut clicks: MessageWriter<MouseClicked>,
) {
    let Some(target) = cursor.hovered else {
        return;
    };
    if state.just_pressed(GamepadAction::StudyConfirm) {
        clicks.write(MouseClicked { button: target });
    }
}

/// Pans the graph when the cursor is pushed against the edge of the panel.
/// Pan speed scales with how close to the edge the cursor is (0 at
/// `EDGE_SCROLL_THRESHOLD` from the edge, full speed at the edge itself).
#[allow(clippy::too_many_arguments)]
pub(crate) fn study_cursor_edge_scroll(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    state: Res<GamepadActionState>,
    aim: Res<GamepadAimSettings>,
    cursor: Res<StudyCursorMode>,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    const EDGE_SCROLL_THRESHOLD: f32 = 60.0; // logical px from edge
    const EDGE_SCROLL_SPEED: f32 = 700.0; // logical px/sec at the edge

    if !active.is_gamepad() {
        return;
    }
    let Ok(area_node) = graph_area.single() else {
        return;
    };

    // Stick input: only scroll while the player is actively pushing toward
    // the edge — letting go of the stick while the cursor rests at the edge
    // should stop the pan.
    let stick = shape_stick(state.left_stick, &aim);
    if stick == Vec2::ZERO {
        return;
    }

    let size = area_node.size() * area_node.inverse_scale_factor();
    let pos = cursor.position;

    // Per-axis edge fraction: 0 when inside, rising to 1 at the edge. Sign
    // matches the pan direction the graph should move (`offset` +x reveals
    // content on the left).
    let edge_frac = |p: f32, max: f32| -> f32 {
        if p < EDGE_SCROLL_THRESHOLD {
            1.0 - (p / EDGE_SCROLL_THRESHOLD).clamp(0.0, 1.0)
        } else if p > max - EDGE_SCROLL_THRESHOLD {
            -(1.0 - ((max - p) / EDGE_SCROLL_THRESHOLD).clamp(0.0, 1.0))
        } else {
            0.0
        }
    };
    // Pan only along axes where both the edge AND the stick are pushing in
    // the same direction. `edge_frac` is positive when near the left/top
    // (pan content right/down) and the stick is negative in those directions
    // (pushing left/up) — so they have OPPOSITE signs when aligned.
    let raw_pan = Vec2::new(edge_frac(pos.x, size.x), edge_frac(pos.y, size.y));
    let pan = Vec2::new(
        if raw_pan.x * stick.x < 0.0 {
            raw_pan.x
        } else {
            0.0
        },
        if raw_pan.y * stick.y < 0.0 {
            raw_pan.y
        } else {
            0.0
        },
    );
    if pan == Vec2::ZERO {
        return;
    }
    view.offset += pan * EDGE_SCROLL_SPEED * time.delta_secs();
    if let Some(b) = bounds.as_ref() {
        clamp_view_offset(&mut view, b);
    }
}

/// Zooms the graph around the cursor when either trigger is held. RT zooms
/// in (scale up), LT zooms out. Reads the triggers as analog axes so partial
/// squeezes scale the zoom rate.
#[allow(clippy::too_many_arguments)]
pub(crate) fn study_cursor_trigger_zoom(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    state: Res<GamepadActionState>,
    cursor: Res<StudyCursorMode>,
    mut commands: Commands,
    mut view: ResMut<GraphViewState>,
    bounds: Option<Res<GraphBounds>>,
    graph_area: Query<&ComputedNode, With<SpellGraphArea>>,
) {
    const TRIGGER_DEADZONE: f32 = 0.08;
    const TRIGGER_ZOOM_RATE: f32 = 2.4; // exp rate/sec at full trigger

    if !active.is_gamepad() {
        return;
    }
    let rt = state.right_trigger.max(0.0);
    let lt = state.left_trigger.max(0.0);
    let rt = if rt >= TRIGGER_DEADZONE { rt } else { 0.0 };
    let lt = if lt >= TRIGGER_DEADZONE { lt } else { 0.0 };
    let signed = rt - lt; // +1 zoom in, -1 zoom out
    if signed.abs() < f32::EPSILON {
        return;
    }

    let Ok(area_node) = graph_area.single() else {
        return;
    };
    let old_scale = view.scale;
    let factor = (signed * TRIGGER_ZOOM_RATE * time.delta_secs()).exp();
    let new_scale = (old_scale * factor).clamp(GRAPH_ZOOM_MIN, GRAPH_ZOOM_MAX);
    if (new_scale - old_scale).abs() < f32::EPSILON {
        return;
    }

    // Keep the graph point under the cursor fixed during the zoom.
    let area_size = area_node.size() * area_node.inverse_scale_factor();
    let cursor_from_center = cursor.position - area_size * 0.5;
    let graph_point = (cursor_from_center - view.offset) / old_scale;
    view.offset = cursor_from_center - graph_point * new_scale;
    view.scale = new_scale;

    commands.remove_resource::<GraphViewAnimation>();
    if let Some(b) = bounds.as_ref() {
        clamp_view_offset(&mut view, b);
    }
}

/// Removes cursor resources when the `SpellGraphArea` is despawned (tab
/// change) so focus navigation resumes for the next tab.
pub(crate) fn cleanup_study_cursor_on_area_removed(
    mut commands: Commands,
    mut removed: RemovedComponents<SpellGraphArea>,
) {
    if removed.read().next().is_some() {
        commands.remove_resource::<StudyCursorMode>();
        commands.remove_resource::<crate::ui::focus::FocusNavInhibit>();
    }
}
