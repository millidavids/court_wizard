use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

const LINE_HEIGHT: f32 = 20.0;

/// Injects scroll events into the UI hierarchy.
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(ScrollEvent { entity, delta });
            }
        }
    }
}

/// UI scrolling event.
#[derive(Event)]
pub struct ScrollEvent {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

/// Handles scroll events on UI nodes with overflow by traversing up the hierarchy.
pub fn on_scroll_handler(
    trigger: On<ScrollEvent>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode, Option<&ChildOf>)>,
) {
    let scroll_event = trigger.event();
    let mut current_entity = scroll_event.entity;
    let delta = scroll_event.delta;

    // Traverse up the hierarchy until we find a scrollable container
    loop {
        let Ok((mut scroll_position, node, computed, parent)) = query.get_mut(current_entity)
        else {
            break;
        };

        // Check if this node is scrollable in the Y direction
        if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
            let max_offset =
                (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

            // Is this node already scrolled all the way in the direction of the scroll?
            let at_limit = if delta.y > 0.0 {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.0
            };

            if !at_limit {
                scroll_position.y += delta.y;
                scroll_position.y = scroll_position.y.clamp(0.0, max_offset.y);
                // Successfully scrolled, stop traversing
                break;
            }
        }

        // Move up to parent if it exists
        if let Some(parent) = parent {
            current_entity = parent.0;
        } else {
            // No more parents, stop
            break;
        }
    }
}
