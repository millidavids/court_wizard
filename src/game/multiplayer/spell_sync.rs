//! Bidirectional spell visual synchronization.
//!
//! Both host and guest collect their local spell visuals into a
//! `SpellVisualSnapshot` and send it over the unreliable channel.
//! The receiving client renders ghost entities from the snapshot.
//!
//! This runs symmetrically on both clients — each sees the other's spells.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::multiplayer::components::{
    GhostBeam, GhostMagicMissile, GhostSpellArc, GhostSpellProjectile, NetworkedSpellEffect,
    OnMultiplayerGameScreen, SpellEffectEntityMap,
};
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::chain_lightning::components::ChainLightningArc;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::entangle::components::EntangleGroundEffect;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::{Fireball, FireballExplosion};
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeZone;
use crate::game::units::wizard::spells::lightning_rod::components::{
    LightningRod, LightningRodArc, LightningStrike,
};
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire, MeteorProjectile,
};
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::squall::components::{IceExplosion, IceProjectile};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    BeamSnapshot, MagicMissileSnapshot, SpellArcSnapshot, SpellEffectKind, SpellEffectSnapshot,
    SpellProjectileSnapshot, SpellVisualSnapshot, UNRELIABLE_SPELL_SNAPSHOT,
};

/// Resource holding the latest received remote spell visual snapshot.
#[derive(Resource, Default)]
pub struct LatestSpellSnapshot(pub Option<SpellVisualSnapshot>);

/// Collects persistent spell effect entities into the spell visual snapshot.
///
/// Queries all entities with `NetworkedSpellEffect` and builds the spell_effects
/// vector. Uses `NetworkEntityId` if available, otherwise `Entity::index()`.
#[allow(clippy::type_complexity)]
pub fn collect_spell_effect_snapshots(
    mut spell_data: ResMut<SpellSnapshotData>,
    effects: Query<(
        Entity,
        Option<&NetworkEntityId>,
        &NetworkedSpellEffect,
        &Transform,
    )>,
    zone_data: (
        Query<&SpikeGrowthZone>,
        Query<&HealingPlumeZone>,
        Query<&EntangleGroundEffect>,
        Query<&FogCloudZone>,
        Query<&GreaseZone>,
        Query<&PlagueWindCloud>,
        Query<&MeteorGroundFire>,
    ),
    object_data: (
        Query<&BlackHole>,
        Query<&ArcaneCrystal>,
        Query<&LightningRod>,
    ),
    wall_data: (Query<&WallOfStone>, Query<&WallOfFireEffect>),
    explosion_data: (
        Query<&FireballExplosion>,
        Query<&MeteorExplosion>,
        Query<&IceExplosion>,
    ),
) {
    spell_data.spell_effects.clear();

    for (entity, net_id, effect, transform) in &effects {
        let t = transform.translation;
        let rot_y = transform.rotation.to_euler(EulerRot::YXZ).0;

        // Use NetworkEntityId if assigned, otherwise use Entity index
        let id = net_id.map_or(entity.index(), |n| n.0);

        let extra = match effect.kind {
            SpellEffectKind::SpikeGrowthZone => {
                if let Ok(z) = zone_data.0.get(entity) {
                    [z.radius, z.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::HealingPlumeZone => {
                if let Ok(z) = zone_data.1.get(entity) {
                    [z.radius, z.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::EntangleGround => {
                if let Ok(z) = zone_data.2.get(entity) {
                    [0.0, z.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::FogCloudZone => {
                if let Ok(z) = zone_data.3.get(entity) {
                    [z.radius, z.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::GreaseZone => {
                if let Ok(z) = zone_data.4.get(entity) {
                    [z.radius, z.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::GreaseFire => [transform.scale.x, 0.0, 0.0, 0.0],
            SpellEffectKind::PlagueWindCloud => {
                if let Ok(c) = zone_data.5.get(entity) {
                    [
                        c.radius,
                        c.duration,
                        c.speed,
                        c.direction.x.atan2(c.direction.z),
                    ]
                } else {
                    continue;
                }
            }
            SpellEffectKind::MeteorGroundFire => {
                if let Ok(f) = zone_data.6.get(entity) {
                    [f.radius, f.duration, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::BlackHole => {
                if let Ok(bh) = object_data.0.get(entity) {
                    [bh.max_radius, bh.empowerment, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::ArcaneCrystal => {
                if let Ok(ac) = object_data.1.get(entity) {
                    [ac.range, ac.duration, ac.empowerment, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::LightningRod => {
                if let Ok(lr) = object_data.2.get(entity) {
                    [lr.duration, lr.empowerment, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::WallOfStone => {
                if let Ok(w) = wall_data.0.get(entity) {
                    [w.half_length, w.half_width, w.height, w.duration]
                } else {
                    continue;
                }
            }
            SpellEffectKind::WallOfFire => {
                if let Ok(w) = wall_data.1.get(entity) {
                    [w.half_width, w.duration, transform.scale.x, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::FireballExplosion => {
                if let Ok(e) = explosion_data.0.get(entity) {
                    [e.max_radius, e.empowerment, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::MeteorExplosion => {
                if let Ok(e) = explosion_data.1.get(entity) {
                    [e.max_radius, 0.0, 0.0, 0.0]
                } else {
                    continue;
                }
            }
            SpellEffectKind::IceExplosion => {
                if let Ok(e) = explosion_data.2.get(entity) {
                    [e.max_radius, e.empowerment, 0.0, 0.0]
                } else {
                    continue;
                }
            }
        };

        spell_data.spell_effects.push(SpellEffectSnapshot {
            net_id: id,
            kind: effect.kind as u8,
            x: t.x,
            y: t.y,
            z: t.z,
            rotation_y: rot_y,
            extra,
        });
    }
}

/// Collects ephemeral spell projectiles and arcs into the snapshot data.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn collect_spell_projectile_snapshots(
    mut spell_data: ResMut<SpellSnapshotData>,
    fireballs: Query<&Transform, With<Fireball>>,
    ice_projectiles: Query<&Transform, With<IceProjectile>>,
    meteor_projectiles: Query<&Transform, With<MeteorProjectile>>,
    chain_arcs: Query<&ChainLightningArc>,
    lightning_strikes: Query<(&LightningStrike, &Transform)>,
    lightning_rod_arcs: Query<(&LightningRodArc, &Transform)>,
    fod_beams: Query<&FingerOfDeathBeam>,
    disintegrate_beams: Query<&DisintegrateBeam>,
) {
    spell_data.spell_projectiles.clear();
    spell_data.spell_arcs.clear();

    for t in &fireballs {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 0,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    for t in &ice_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 1,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    for t in &meteor_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 2,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    for arc in &chain_arcs {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 0,
            ox: arc.start.x,
            oy: arc.start.y,
            oz: arc.start.z,
            tx: arc.end.x,
            ty: arc.end.y,
            tz: arc.end.z,
        });
    }

    for (strike, transform) in &lightning_strikes {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 1,
            ox: transform.translation.x,
            oy: transform.translation.y,
            oz: transform.translation.z,
            tx: strike.target_pos.x,
            ty: strike.target_pos.y,
            tz: strike.target_pos.z,
        });
    }

    // Crystal beams (kind=2) and crystal arcs (kind=3) are now represented as
    // DisintegrateBeam and ChainLightningArc entities respectively, captured
    // by the existing disintegrate_beams and chain_arcs queries above.

    for beam in &fod_beams {
        let end = beam.origin + beam.direction * beam.length;
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 4,
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }

    for (_arc, transform) in &lightning_rod_arcs {
        let pos = transform.translation;
        let scale = transform.scale;
        let up = transform.rotation * Vec3::Y;
        let half_len = scale.y * 0.5;
        let start = pos - up * half_len;
        let end = pos + up * half_len;
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 5,
            ox: start.x,
            oy: start.y,
            oz: start.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }

    for beam in &disintegrate_beams {
        let end = beam.origin + beam.direction * beam.current_length();
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 6,
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }
}

/// Collects magic missiles and beams, then serializes and sends the complete
/// spell visual snapshot over the unreliable channel.
pub fn send_spell_visual_snapshot(
    mut connection: ResMut<NetworkConnection>,
    mut spell_data: ResMut<SpellSnapshotData>,
    missiles: Query<&Transform, With<MagicMissile>>,
    beams: Query<&DisintegrateBeam>,
) {
    let snapshot = SpellVisualSnapshot {
        spell_effects: std::mem::take(&mut spell_data.spell_effects),
        spell_projectiles: std::mem::take(&mut spell_data.spell_projectiles),
        spell_arcs: std::mem::take(&mut spell_data.spell_arcs),
        magic_missiles: missiles
            .iter()
            .map(|t| MagicMissileSnapshot {
                x: t.translation.x,
                y: t.translation.y,
                z: t.translation.z,
            })
            .collect(),
        beams: beams
            .iter()
            .map(|beam| BeamSnapshot {
                ox: beam.origin.x,
                oy: beam.origin.y,
                oz: beam.origin.z,
                dx: beam.direction.x,
                dy: beam.direction.y,
                dz: beam.direction.z,
                length: beam.current_length(),
            })
            .collect(),
    };

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(UNRELIABLE_SPELL_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}

/// Receives spell visual snapshots from the remote peer and stores the latest.
///
/// Filters incoming unreliable data by type prefix byte, taking only spell
/// snapshots and re-queuing any non-spell data (game snapshots) for other systems.
pub fn receive_spell_visual_snapshot(
    mut connection: ResMut<NetworkConnection>,
    mut latest: ResMut<LatestSpellSnapshot>,
) {
    latest.0 = None;

    // Separate spell snapshots from other unreliable data
    let all_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_spell_data: Option<&[u8]> = None;

    for data in &all_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_SPELL_SNAPSHOT => {
                latest_spell_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    // Re-queue non-spell data for other systems (game snapshots)
    connection.incoming_unreliable = other_data;

    // Deserialize the latest spell snapshot
    if let Some(spell_bytes) = latest_spell_data {
        match bincode::deserialize::<SpellVisualSnapshot>(spell_bytes) {
            Ok(snapshot) => {
                latest.0 = Some(snapshot);
            }
            Err(_) => {
                warn!(
                    "Failed to deserialize spell visual snapshot ({} bytes)",
                    spell_bytes.len()
                );
            }
        }
    }
}

/// Renders remote spell visuals from the latest spell visual snapshot.
///
/// - **Persistent effects**: Spawned once, tracked by `SpellEffectEntityMap`, force-despawned when gone.
/// - **Ephemeral**: Despawned and re-spawned each frame (projectiles, arcs, missiles, beams).
#[allow(clippy::too_many_arguments)]
pub fn apply_remote_spell_snapshot(
    mut commands: Commands,
    latest: Res<LatestSpellSnapshot>,
    mut effect_map: ResMut<SpellEffectEntityMap>,
    assets: Option<Res<SpellVisualAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effect_transforms: Query<&mut Transform>,
    ghost_projectiles: Query<Entity, With<GhostSpellProjectile>>,
    ghost_arcs: Query<Entity, With<GhostSpellArc>>,
    ghost_missiles: Query<Entity, With<GhostMagicMissile>>,
    ghost_beams: Query<Entity, With<GhostBeam>>,
) {
    let Some(snapshot) = &latest.0 else { return };
    let Some(assets) = assets else { return };

    // ── Tier 2: Persistent Spell Effects ──────────────────────────────────

    let mut seen_effect_ids = HashSet::with_capacity(snapshot.spell_effects.len());

    for effect in &snapshot.spell_effects {
        seen_effect_ids.insert(effect.net_id);

        if let Some(&local_entity) = effect_map.remote_to_local.get(&effect.net_id) {
            if effect.kind == SpellEffectKind::GreaseFire as u8
                && let Ok(mut transform) = effect_transforms.get_mut(local_entity)
            {
                transform.scale = Vec3::splat(effect.extra[0].max(0.01));
            }
            continue;
        }

        if let Some(entity) =
            super::guest_systems::spawn_spell_effect(&mut commands, effect, &assets, &mut materials)
        {
            effect_map.insert(effect.net_id, entity);
        }
    }

    let stale_ids: Vec<u32> = effect_map
        .remote_to_local
        .keys()
        .copied()
        .filter(|id| !seen_effect_ids.contains(id))
        .collect();

    for stale_id in stale_ids {
        if let Some(entity) = effect_map.remove_by_remote(stale_id)
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }

    // ── Tier 1: Ephemeral Spell Projectiles ──────────────────────────────

    for entity in &ghost_projectiles {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for proj in &snapshot.spell_projectiles {
        let (mesh, material) = match proj.kind {
            0 => (
                assets.unit_sphere.clone(),
                assets.fireball_projectile.clone(),
            ),
            1 => (assets.unit_sphere.clone(), assets.ice_projectile.clone()),
            2 => (assets.unit_sphere.clone(), assets.meteor_projectile.clone()),
            _ => continue,
        };

        let scale = match proj.kind {
            0 => 12.0,
            1 => 8.0,
            2 => 10.0,
            _ => 8.0,
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(proj.x, proj.y, proj.z))
                .with_scale(Vec3::splat(scale)),
            GhostSpellProjectile,
            OnMultiplayerGameScreen,
        ));
    }

    // ── Tier 1: Ephemeral Spell Arcs ─────────────────────────────────────

    for entity in &ghost_arcs {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for arc in &snapshot.spell_arcs {
        let material = match arc.kind {
            0 => assets.chain_lightning_arc.clone(),
            1 => assets.lightning_strike.clone(),
            2 => assets.crystal_beam.clone(),
            3 => assets.crystal_arc.clone(),
            4 => assets.finger_of_death_beam.clone(),
            5 => assets.lightning_rod_arc.clone(),
            6 => assets.disintegrate_beam.clone(),
            _ => continue,
        };

        let origin = Vec3::new(arc.ox, arc.oy, arc.oz);
        let target = Vec3::new(arc.tx, arc.ty, arc.tz);
        let diff = target - origin;
        let length = diff.length();
        if length < 0.1 {
            continue;
        }

        let direction = diff / length;
        let midpoint = origin + diff * 0.5;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        let width = match arc.kind {
            0 | 3 | 5 => 6.0,
            1 => 8.0,
            2 | 4 => 20.0,
            6 => 16.0,
            _ => 6.0,
        };

        // Beam-type arcs (finger of death=4, disintegrate=6) use cylinder mesh
        // so they're visible from all camera angles. Other arcs use flat rectangles.
        let is_beam = arc.kind == 4 || arc.kind == 6;
        let mesh = if is_beam {
            assets.unit_cylinder.clone()
        } else {
            assets.unit_rect.clone()
        };
        let scale = if is_beam {
            Vec3::new(width, length, width)
        } else {
            Vec3::new(width, length, 1.0)
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(scale),
            GhostSpellArc,
            OnMultiplayerGameScreen,
        ));
    }

    // ── Ephemeral Magic Missiles ─────────────────────────────────────────

    for entity in &ghost_missiles {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for missile in &snapshot.magic_missiles {
        commands.spawn((
            Mesh3d(assets.magic_missile_mesh.clone()),
            MeshMaterial3d(assets.magic_missile.clone()),
            Transform::from_translation(Vec3::new(missile.x, missile.y, missile.z)),
            GhostMagicMissile,
            OnMultiplayerGameScreen,
        ));
    }

    // ── Ephemeral Beams ──────────────────────────────────────────────

    for entity in &ghost_beams {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for beam in &snapshot.beams {
        let origin = Vec3::new(beam.ox, beam.oy, beam.oz);
        let direction = Vec3::new(beam.dx, beam.dy, beam.dz);
        let length = beam.length;

        let midpoint = origin + direction * (length / 2.0);
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        commands.spawn((
            Mesh3d(assets.unit_cylinder.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(30.0, length, 30.0)),
            GhostBeam,
            OnMultiplayerGameScreen,
        ));
    }
}

use crate::networking::snapshot::SpellSnapshotData;
