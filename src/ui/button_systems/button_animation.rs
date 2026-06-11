use bevy::prelude::*;

use super::super::components::{ButtonAnimState, ButtonFront};
use super::super::constants::{BUTTON_3D_ANIM_SPEED, BUTTON_3D_OFFSET_PRESSED};

/// Smoothly animates the 3D button front face toward its target offset.
/// Uses real (wall-clock) time so animations play even when game time is paused/scaled.
/// Updates both the Node.top on the front face and ButtonAnimState.current in one pass.
pub fn animate_button_3d(
    time: Res<Time<Real>>,
    mut buttons: Query<(&mut ButtonAnimState, &Children)>,
    mut front_query: Query<&mut Node, With<ButtonFront>>,
) {
    let dt = time.delta_secs();
    for (mut anim, children) in &mut buttons {
        if (anim.current - anim.target).abs() < 0.01 {
            anim.current = anim.target;
            continue;
        }

        let speed = if anim.target == BUTTON_3D_OFFSET_PRESSED {
            BUTTON_3D_ANIM_SPEED * 3.0
        } else {
            BUTTON_3D_ANIM_SPEED
        };
        let t = (speed * dt).min(1.0);
        anim.current += (anim.target - anim.current) * t;

        for child in children.iter() {
            if let Ok(mut node) = front_query.get_mut(child) {
                node.top = Val::Px(anim.current);
            }
        }
    }
}
