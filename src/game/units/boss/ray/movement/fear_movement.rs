use bevy::prelude::*;

use crate::game::components::{Acceleration, Velocity};
use crate::game::units::components::{
    BerserkerRageModifier, FearModifier, MindControlled, MovementSpeed,
};

pub fn cleanse_fear_with_rage(
    mut commands: Commands,
    rage_query: Query<Entity, (With<FearModifier>, With<BerserkerRageModifier>)>,
    mc_query: Query<Entity, (With<FearModifier>, With<MindControlled>)>,
) {
    for entity in &rage_query {
        commands.entity(entity).remove::<FearModifier>();
    }
    for entity in &mc_query {
        commands.entity(entity).remove::<FearModifier>();
    }
}

pub fn update_fear_movement(
    mut query: Query<(
        &Transform,
        &FearModifier,
        &mut Velocity,
        &mut Acceleration,
        &MovementSpeed,
    )>,
) {
    for (transform, fear, mut velocity, mut acceleration, movement_speed) in query.iter_mut() {
        let flee_dir = transform.translation - fear.flee_from;
        let horizontal = Vec3::new(flee_dir.x, 0.0, flee_dir.z);
        let length = horizontal.length();
        if length > 0.001 {
            let normalized = horizontal / length;
            let speed = movement_speed.0 * 1.5;
            velocity.x = normalized.x * speed;
            velocity.z = normalized.z * speed;
            acceleration.reset();
        }
    }
}
