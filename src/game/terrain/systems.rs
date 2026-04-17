//! Cross-cutting terrain reaction systems. Listen to `TerrainDamageMessage` broadcast by
//! damage-dealing spells and apply reactive effects to ponds and boulders.

use bevy::prelude::*;

use super::boulder::components::{Boulder, BoulderHeat};
use super::boulder::constants::{
    BOULDER_EXPLOSION_DAMAGE_MULT, BOULDER_EXPLOSION_RADIUS_MULT, BOULDER_HEAT_DECAY_DELAY,
    BOULDER_HEAT_THRESHOLD,
};
use super::messages::TerrainDamageMessage;
use super::pond::components::{Pond, PondEvaporation, PondFogCloud, PondFrozen, PondShocked};
use super::pond::constants::{
    FOG_PER_DAMAGE, POND_FREEZE_DECAY_DELAY, POND_FREEZE_PER_DAMAGE, POND_SHOCK_DURATION,
};
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::{
    constants as fireball_constants,
};
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};

/// Fans out `TerrainDamageMessage` to ponds, upserting reactive-state components.
pub fn apply_terrain_damage_to_ponds(
    mut commands: Commands,
    mut msgs: MessageReader<TerrainDamageMessage>,
    mut ponds: Query<(
        Entity,
        &Pond,
        Option<&mut PondEvaporation>,
        Option<&mut PondFrozen>,
        Option<&mut PondShocked>,
    )>,
) {
    for msg in msgs.read() {
        for (entity, pond, evap, frozen, shocked) in &mut ponds {
            let dx = msg.position.x - pond.center.x;
            let dz = msg.position.z - pond.center.z;
            let dist_sq = dx * dx + dz * dz;
            let reach = msg.radius + pond.radius;
            if dist_sq > reach * reach {
                continue;
            }

            match msg.damage_type {
                DamageType::Fire => {
                    if let Some(mut evap) = evap {
                        evap.fog_intensity += msg.damage * FOG_PER_DAMAGE;
                        evap.time_since_contribution = 0.0;
                    } else {
                        commands.entity(entity).insert(PondEvaporation {
                            fog_intensity: msg.damage * FOG_PER_DAMAGE,
                            time_since_contribution: 0.0,
                        });
                        commands.spawn((
                            PondFogCloud::default(),
                            ChildOf(entity),
                            OnGameplayScreen,
                        ));
                    }
                }
                DamageType::Frost => {
                    if let Some(mut frozen) = frozen {
                        frozen.freeze_level =
                            (frozen.freeze_level + msg.damage * POND_FREEZE_PER_DAMAGE).min(1.0);
                        frozen.decay_delay = POND_FREEZE_DECAY_DELAY;
                    } else {
                        commands.entity(entity).insert(PondFrozen {
                            freeze_level: (msg.damage * POND_FREEZE_PER_DAMAGE).min(1.0),
                            decay_delay: POND_FREEZE_DECAY_DELAY,
                            pathfinding_frozen: false,
                        });
                    }
                }
                DamageType::Electric => {
                    if let Some(mut shocked) = shocked {
                        shocked.time_remaining = POND_SHOCK_DURATION;
                    } else {
                        commands.entity(entity).insert(PondShocked {
                            time_remaining: POND_SHOCK_DURATION,
                            arc_cooldown: 0.0,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

/// Fans out fire `TerrainDamageMessage`s to boulders, accumulating heat. When heat crosses
/// the explosion threshold, spawns a 2× fireball explosion at the boulder position, clears
/// pathfinding, and despawns the boulder immediately (no sink animation — it was blown up).
/// Runs independently of `apply_spell_damage_to_rocks` which handles HP damage from non-fire
/// sources; a boulder can still explode from heat even if its HP reached 0 the same frame.
#[allow(clippy::too_many_arguments)]
pub fn apply_terrain_damage_to_boulders(
    mut commands: Commands,
    mut msgs: MessageReader<TerrainDamageMessage>,
    mut boulders: Query<(Entity, &Boulder, Option<&mut BoulderHeat>)>,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for msg in msgs.read() {
        if msg.damage_type != DamageType::Fire {
            continue;
        }
        for (entity, boulder, heat) in &mut boulders {
            if boulder.sinking {
                continue;
            }
            let dx = msg.position.x - boulder.center.x;
            let dz = msg.position.z - boulder.center.z;
            let dist_sq = dx * dx + dz * dz;
            let reach = msg.radius + boulder.radius;
            if dist_sq > reach * reach {
                continue;
            }

            let reached_threshold = if let Some(mut heat) = heat {
                heat.heat += msg.damage;
                heat.decay_delay = BOULDER_HEAT_DECAY_DELAY;
                heat.heat >= BOULDER_HEAT_THRESHOLD
            } else {
                let new_heat = msg.damage;
                commands.entity(entity).insert(BoulderHeat {
                    heat: new_heat,
                    decay_delay: BOULDER_HEAT_DECAY_DELAY,
                });
                new_heat >= BOULDER_HEAT_THRESHOLD
            };

            if reached_threshold {
                spawn_boulder_explosion(
                    &mut commands,
                    &visual_assets,
                    &mut sphere_materials,
                    boulder.center,
                );
                let obs_bounds = boulder.obstacle_bounds();
                obstacle_events.write(ObstacleChanged {
                    bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                    obstacle_type: ObstacleType::Removed,
                    shape: Some(ObstacleShape::circle(
                        Vec2::new(boulder.center.x, boulder.center.z),
                        boulder.radius,
                    )),
                    rebuild: false,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawns a fireball-style explosion at the given position with 2× radius and 2× damage.
/// Reuses the full Fireball explosion entity so visuals, damage ticks, and fade-out are
/// identical to a normal fireball — just bigger.
fn spawn_boulder_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    position: Vec3,
) {
    let explosion_radius = fireball_constants::EXPLOSION_RADIUS * BOULDER_EXPLOSION_RADIUS_MULT;
    let damage_per_tick = fireball_constants::DAMAGE_PER_TICK * BOULDER_EXPLOSION_DAMAGE_MULT;

    let mut explosion = FireballExplosion::new(
        position,
        explosion_radius,
        damage_per_tick,
        DamageType::Fire,
        1.0,
    );
    explosion.duration = fireball_constants::EXPLOSION_DURATION;
    explosion.source_spell = Spell::Fireball;

    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    commands.spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
        explosion,
        OnGameplayScreen,
    ));
}
