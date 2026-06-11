use bevy::prelude::*;

use super::super::components::{Boulder, BoulderHeat, BoulderShadow, ClonedMaterial};
use super::super::constants::*;
use crate::game::components::ObstacleHealth;

/// Tints boulder sprite from white (full HP) toward a dark color (zero HP).
/// Clones the shared material on first damage so tinting one boulder doesn't affect others.
pub fn update_rock_damage_tint(
    mut commands: Commands,
    mut rocks: Query<
        (
            Entity,
            &ObstacleHealth,
            &mut MeshMaterial3d<StandardMaterial>,
            Has<ClonedMaterial>,
        ),
        With<Boulder>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base = Color::WHITE.to_srgba();
    let damaged = ROCK_DAMAGED_COLOR.to_srgba();

    for (entity, health, mut material_handle, already_cloned) in &mut rocks {
        if health.current >= health.max {
            continue;
        }

        if !already_cloned {
            let Some(shared_mat) = materials.get(&material_handle.0) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
            commands.entity(entity).insert(ClonedMaterial);
        }

        let Some(mat) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        let fraction = health.fraction();
        let r = damaged.red + (base.red - damaged.red) * fraction;
        let g = damaged.green + (base.green - damaged.green) * fraction;
        let b = damaged.blue + (base.blue - damaged.blue) * fraction;
        mat.base_color = Color::srgba(r, g, b, 1.0);
    }
}

pub fn cleanup_rock_shadows(
    mut commands: Commands,
    shadows: Query<(Entity, &BoulderShadow)>,
    rocks: Query<Entity, With<Boulder>>,
) {
    for (shadow_entity, shadow) in &shadows {
        if rocks.get(shadow.owner).is_err() {
            commands.entity(shadow_entity).try_despawn();
        }
    }
}

/// Bleeds off accumulated fire heat after `BOULDER_HEAT_DECAY_DELAY` seconds of no fire contribution.
/// Removes the `BoulderHeat` component when heat reaches zero.
pub fn tick_boulder_heat(
    mut commands: Commands,
    time: Res<Time>,
    mut boulders: Query<(Entity, &mut BoulderHeat)>,
) {
    let delta = time.delta_secs();
    for (entity, mut heat) in &mut boulders {
        if heat.decay_delay > 0.0 {
            heat.decay_delay = (heat.decay_delay - delta).max(0.0);
            continue;
        }
        heat.heat = (heat.heat - BOULDER_HEAT_DECAY_RATE * delta).max(0.0);
        if heat.heat <= 0.0 {
            commands.entity(entity).remove::<BoulderHeat>();
        }
    }
}
