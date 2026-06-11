use bevy::prelude::*;

use super::super::components::{Boulder, BoulderProjectile, BoulderShadow};
use super::super::constants::*;
use super::super::messages::*;
use super::super::resources::BoulderAssets;
use crate::config::GameConfig;
use crate::game::components::{Billboard, ObstacleHealth, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::ShadowAssets;
use crate::game::units::components::Teleportable;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};

pub fn spawn_rock_projectile(
    mut commands: Commands,
    mut events: MessageReader<BoulderThrownMessage>,
    rock_assets: Res<BoulderAssets>,
) {
    for event in events.read() {
        let start_y = 20.0; // Launch from unit height
        let start = Vec3::new(event.origin.x, start_y, event.origin.z);
        let target = Vec3::new(event.target.x, 0.0, event.target.z);
        let idx = (event.sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

        let projectile = BoulderProjectile {
            start,
            target,
            duration: ROCK_PROJECTILE_DURATION,
            elapsed: 0.0,
            arc_height: ROCK_PROJECTILE_ARC_HEIGHT,
            sprite_index: event.sprite_index,
        };

        let pos = projectile.current_position();

        commands.spawn((
            Mesh3d(rock_assets.mesh.clone()),
            MeshMaterial3d(rock_assets.materials[idx].clone()),
            Transform::from_translation(pos),
            projectile,
            // Tag for MP visual sync — host's projectile entity ships to the
            // guest, which spawns a ghost at the synced position each frame
            // so the guest sees the full arc.
            crate::game::multiplayer::components::NetworkedSpellEffect {
                kind: crate::networking::snapshot::SpellEffectKind::BoulderProjectileEffect,
            },
            OnGameplayScreen,
        ));
    }
}

/// Animates boulder projectiles along their parabolic arc and spawns landed boulders.
#[allow(clippy::too_many_arguments)]
pub fn animate_rock_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    rock_assets: Res<BoulderAssets>,
    shadow_assets: Res<ShadowAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut projectiles: Query<(Entity, &mut BoulderProjectile, &mut Transform)>,
    existing_rocks: Query<&Boulder>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    for (entity, mut projectile, mut transform) in &mut projectiles {
        projectile.elapsed += delta;
        let pos = projectile.current_position();
        transform.translation = pos;

        transform.rotate_z(BOULDER_SPIN_SPEED * delta);

        if projectile.is_landed() {
            // Despawn the projectile
            commands.entity(entity).despawn();

            // Determine landing position, resolving overlaps with existing boulders
            let mut land_pos = Vec3::new(projectile.target.x, 0.0, projectile.target.z);
            land_pos = resolve_overlap(land_pos, &existing_rocks);

            let rock_y = BOULDER_SPRITE_HEIGHT / 2.0 - BOULDER_GROUND_CLIP;
            let idx = (projectile.sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

            // Spawn the permanent boulder obstacle
            let rock = Boulder {
                center: Vec3::new(land_pos.x, 0.0, land_pos.z),
                radius: ROCK_RADIUS,
                height: ROCK_HEIGHT,
                sinking: false,
                time_alive: 0.0,
                sink_deadline: f32::MAX,
                sprite_index: projectile.sprite_index,
            };

            let obs_bounds = rock.obstacle_bounds();
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Blocked,
                shape: Some(ObstacleShape::circle(
                    Vec2::new(rock.center.x, rock.center.z),
                    rock.radius,
                )),
                rebuild: false,
            });

            // Play boulder impact sound
            audio::play_impact_sfx(
                &mut commands,
                &sfx.boulder_impact,
                Vec3::new(land_pos.x, 0.0, land_pos.z),
                &game_config,
                &sfx,
            );

            // Landed boulder spawns upright (no rotation carried over from spin)
            let rock_entity = commands
                .spawn((
                    Mesh3d(rock_assets.mesh.clone()),
                    MeshMaterial3d(rock_assets.materials[idx].clone()),
                    Transform::from_xyz(land_pos.x, rock_y, land_pos.z),
                    rock,
                    ObstacleHealth::new(ROCK_HEALTH),
                    Billboard,
                    Teleportable,
                    // Tag for MP visual sync — the persistent boulder
                    // obstacle ships to the guest as a separate effect kind
                    // so the guest can pick the correct sprite via the
                    // `sprite_index` packed into `extra[0]`.
                    crate::game::multiplayer::components::NetworkedSpellEffect {
                        kind: crate::networking::snapshot::SpellEffectKind::BoulderObstacle,
                    },
                    OnGameplayScreen,
                ))
                .id();

            // Spawn a shadow under the boulder
            commands.spawn((
                Mesh3d(shadow_assets.mesh.clone()),
                MeshMaterial3d(shadow_assets.material.clone()),
                Transform::from_xyz(land_pos.x, ROCK_SHADOW_Y, land_pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(ROCK_SHADOW_SCALE)),
                BoulderShadow { owner: rock_entity },
                OnGameplayScreen,
            ));
        }
    }
}

/// Resolves overlap between a new boulder landing position and existing boulders.
/// Pushes the new boulder outward so it sits adjacent to any overlapping boulder.
fn resolve_overlap(mut pos: Vec3, existing_rocks: &Query<&Boulder>) -> Vec3 {
    // Iterate a few times to resolve cascading overlaps
    for _ in 0..8 {
        let mut adjusted = false;
        for rock in existing_rocks.iter() {
            if rock.sinking {
                continue;
            }
            let dx = pos.x - rock.center.x;
            let dz = pos.z - rock.center.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < ROCK_MIN_SEPARATION {
                if dist < 0.001 {
                    // Exactly overlapping, push in arbitrary direction
                    pos.x += ROCK_MIN_SEPARATION;
                } else {
                    let push = (ROCK_MIN_SEPARATION - dist) / dist;
                    pos.x += dx * push;
                    pos.z += dz * push;
                }
                adjusted = true;
            }
        }
        if !adjusted {
            break;
        }
    }
    pos
}
