use super::super::components::{PandemicProcessed, PlagueWindCloud, PlagueWindTalentParams};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::spells::utils::UniqueHitTracker;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

/// Computes talent parameters from the player's active talent selections.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_plague_cloud(
    commands: &mut Commands,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    duration: f32,
    speed: f32,
    direction: Vec3,
    talent_params: PlagueWindTalentParams,
) {
    // Notify pathfinding
    let origin_2d = Vec2::new(pos.x, pos.z);
    let buffered = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
        obstacle_type: ObstacleType::Hazard(10.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered)),
        rebuild: false,
    });

    commands.spawn((
        Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
        PlagueWindCloud::new(
            pos,
            radius,
            damage,
            constants::TICK_INTERVAL,
            duration,
            speed,
            direction,
            talent_params,
        ),
        UniqueHitTracker::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::PlagueWindCloud,
        },
        OnGameplayScreen,
    ));
}

/// Pandemic: when an enemy dies inside a cloud, spawn a smaller child cloud at their position.
/// Only triggers once per death (uses PandemicProcessed marker) and only from non-child clouds.
pub fn spawn_pandemic_clouds(
    mut commands: Commands,
    clouds: Query<&PlagueWindCloud>,
    dead_units: Query<(Entity, &Transform, &Health), (Without<Corpse>, Without<PandemicProcessed>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, transform, health) in &dead_units {
        if !health.is_dead() {
            continue;
        }

        let unit_pos = transform.translation;

        for cloud in &clouds {
            if !cloud.talent_params.pandemic {
                continue;
            }

            if horizontal_distance(cloud.origin, unit_pos) <= cloud.radius {
                // Spawn stationary child cloud at death position
                let child_radius = cloud.radius * constants::PANDEMIC_CHILD_RADIUS_MULT;

                // Child inherits parent talents but cannot spawn further children
                let mut child_params = cloud.talent_params;
                child_params.pandemic = false;

                spawn_plague_cloud(
                    &mut commands,
                    &mut obstacle_events,
                    unit_pos,
                    child_radius,
                    cloud.damage_per_tick,
                    constants::PANDEMIC_CHILD_DURATION,
                    0.0, // Stationary
                    Vec3::ZERO,
                    child_params,
                );

                // Mark this death as processed so we don't spawn again next frame
                commands.entity(entity).insert(PandemicProcessed);

                // Only spawn one child per death
                break;
            }
        }
    }
}
