//! Guest-only multiplayer systems.
//!
//! The guest receives state snapshots from the host and renders ghost entities
//! that mirror the host's game state. Ghost entities are lightweight visual
//! representations with no simulation components.
//!
//! Spell effects use a two-tier approach:
//! - **Tier 2 (persistent)**: Spawned once with real spell components so existing
//!   visual/lifecycle systems run on the guest. Tracked via `SpellEffectEntityMap`.
//! - **Tier 1 (ephemeral)**: Despawned and re-spawned each frame (projectiles, arcs).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::resources::KingAssets;
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::entangle::components::EntangleGroundEffect;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeZone;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire,
};
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::squall::components::IceExplosion;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    GameSnapshot, SpellEffectKind, SpellEffectSnapshot, UnitFlags, u8_to_team,
};
use crate::state::MultiplayerGameState;

use super::components::{
    GhostArrow, GhostBeam, GhostEntity, GhostMagicMissile, GhostSpellArc, GhostSpellAssets,
    GhostSpellProjectile, OnMultiplayerGameScreen, SpellEffectAssets, SpellEffectEntityMap,
};

/// Holds the latest deserialized snapshot for sharing between systems.
#[derive(Resource, Default)]
pub struct LatestSnapshot(pub Option<GameSnapshot>);

/// Receives the latest state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state. Also stores the snapshot in
/// `LatestSnapshot` for `apply_spell_snapshot` to consume.
#[allow(clippy::too_many_arguments)]
pub fn apply_state_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    mut latest: ResMut<LatestSnapshot>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<KingAssets>,
    spell_assets: Option<Res<GhostSpellAssets>>,
    mut ghost_query: Query<(&mut Transform, &mut MeshMaterial3d<StandardMaterial>), With<GhostEntity>>,
    ghost_arrows: Query<Entity, With<GhostArrow>>,
    ghost_missiles: Query<Entity, With<GhostMagicMissile>>,
    ghost_beams: Query<Entity, With<GhostBeam>>,
) {
    latest.0 = None;

    // Take only the latest snapshot (discard stale ones)
    let raw_snapshots: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    if raw_snapshots.is_empty() {
        return;
    }

    let Some(latest_data) = raw_snapshots.last() else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<GameSnapshot>(latest_data) else {
        warn!(
            "Failed to deserialize snapshot ({} bytes)",
            latest_data.len()
        );
        return;
    };

    // Track which IDs are present in this snapshot
    let mut seen_ids = HashSet::with_capacity(snapshot.units.len());

    for unit in &snapshot.units {
        seen_ids.insert(unit.id);

        let is_corpse = unit.flags & UnitFlags::CORPSE != 0;
        let is_king = unit.flags & UnitFlags::KING != 0;
        let is_archer = unit.flags & UnitFlags::ARCHER != 0;
        let is_guard = unit.flags & UnitFlags::KINGS_GUARD != 0;
        let team = u8_to_team(unit.team);

        // Pick the correct preloaded material based on unit type, team, and alive/dead
        let material_handle = pick_material(
            &infantry_assets,
            &archer_assets,
            &king_assets,
            team,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
        );

        // Pick the correct preloaded mesh based on unit type
        let mesh_handle = if is_king {
            king_assets.mesh.clone()
        } else if is_archer {
            archer_assets.mesh.clone()
        } else {
            // Infantry and King's Guard both use infantry mesh
            infantry_assets.mesh.clone()
        };

        let pos = Vec3::new(unit.x, unit.y, unit.z);

        if let Some(&local_entity) = entity_map.remote_to_local.get(&unit.id) {
            // Update existing ghost entity
            if let Ok((mut transform, material_ref)) = ghost_query.get_mut(local_entity) {
                transform.translation = pos;

                // Update material if it changed (e.g., unit became corpse)
                if material_ref.0 != material_handle {
                    commands.entity(local_entity).insert(MeshMaterial3d(material_handle));
                }
            }
        } else {
            // Spawn new ghost entity using preloaded Circle meshes
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(pos),
                    Billboard,
                    GhostEntity,
                    OnMultiplayerGameScreen,
                ))
                .id();

            entity_map.insert(unit.id, entity);
        }
    }

    // Despawn ghost entities whose IDs are no longer in the snapshot
    let stale_ids: Vec<u32> = entity_map
        .remote_to_local
        .keys()
        .copied()
        .filter(|id| !seen_ids.contains(id))
        .collect();

    for stale_id in stale_ids {
        if let Some(entity) = entity_map.remove_by_remote(stale_id)
            && let Ok(mut entity_commands) = commands.get_entity(entity)
        {
            entity_commands.despawn();
        }
    }

    // Replace all ghost arrows with fresh positions from the snapshot.
    // Arrows are ephemeral projectiles — no stable identity tracking needed.
    for entity in &ghost_arrows {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    for arrow in &snapshot.arrows {
        commands.spawn((
            Mesh3d(archer_assets.arrow_mesh.clone()),
            MeshMaterial3d(archer_assets.arrow_material.clone()),
            Transform::from_translation(Vec3::new(arrow.x, arrow.y, arrow.z)),
            Billboard,
            GhostArrow,
            OnMultiplayerGameScreen,
        ));
    }

    // Replace all ghost magic missiles with fresh positions from the snapshot.
    for entity in &ghost_missiles {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    // Replace all ghost beams with fresh data from the snapshot.
    for entity in &ghost_beams {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    if let Some(spell_assets) = spell_assets {
        for missile in &snapshot.magic_missiles {
            commands.spawn((
                Mesh3d(spell_assets.missile_mesh.clone()),
                MeshMaterial3d(spell_assets.missile_material.clone()),
                Transform::from_translation(Vec3::new(missile.x, missile.y, missile.z)),
                Billboard,
                GhostMagicMissile,
                OnMultiplayerGameScreen,
            ));
        }

        for beam in &snapshot.beams {
            let origin = Vec3::new(beam.ox, beam.oy, beam.oz);
            let direction = Vec3::new(beam.dx, beam.dy, beam.dz);
            let length = beam.length;

            // Position at beam midpoint, rotate to align with direction, scale to length
            let midpoint = origin + direction * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

            commands.spawn((
                Mesh3d(spell_assets.beam_mesh.clone()),
                MeshMaterial3d(spell_assets.beam_material.clone()),
                Transform::from_translation(midpoint)
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(30.0, length, 1.0)),
                GhostBeam,
                OnMultiplayerGameScreen,
            ));
        }
    }

    // Store snapshot for apply_spell_snapshot
    latest.0 = Some(snapshot);
}

/// Picks the correct preloaded material handle for a ghost entity.
///
/// Corpse materials already have low alpha baked into the preloaded assets.
#[allow(clippy::too_many_arguments)]
fn pick_material(
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    team: crate::game::units::components::Team,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
) -> Handle<StandardMaterial> {
    use crate::game::units::components::Team;

    if is_corpse {
        if is_king {
            king_assets.corpse_material.clone()
        } else if is_archer {
            match team {
                Team::Defenders => archer_assets.defender_corpse_material.clone(),
                Team::Attackers => archer_assets.attacker_corpse_material.clone(),
                Team::Undead => archer_assets.undead_corpse_material.clone(),
            }
        } else {
            // Infantry and King's Guard corpses use the same material
            match team {
                Team::Defenders => infantry_assets.defender_corpse_material.clone(),
                Team::Attackers => infantry_assets.attacker_corpse_material.clone(),
                Team::Undead => infantry_assets.undead_corpse_material.clone(),
            }
        }
    } else if is_king {
        king_assets.material.clone()
    } else if is_archer {
        match team {
            Team::Defenders => archer_assets.defender_material.clone(),
            Team::Attackers => archer_assets.attacker_material.clone(),
            Team::Undead => archer_assets.undead_material.clone(),
        }
    } else if is_guard {
        infantry_assets.kings_guard_material.clone()
    } else {
        // Infantry
        match team {
            Team::Defenders => infantry_assets.defender_material.clone(),
            Team::Attackers => infantry_assets.attacker_material.clone(),
            Team::Undead => infantry_assets.undead_material.clone(),
        }
    }
}

/// Applies spell effects from the latest snapshot.
///
/// - **Tier 2 (persistent)**: Spawns real spell entities with actual components on first
///   appearance (identified by `net_id`). Existing visual/lifecycle systems handle animation.
///   Force-despawns entities whose IDs disappear from the snapshot.
/// - **Tier 1 (ephemeral)**: Despawns all ghost spell projectiles/arcs and re-spawns fresh.
#[allow(clippy::too_many_arguments)]
pub fn apply_spell_snapshot(
    mut commands: Commands,
    latest: Res<LatestSnapshot>,
    mut effect_map: ResMut<SpellEffectEntityMap>,
    assets: Option<Res<SpellEffectAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut effect_transforms: Query<&mut Transform>,
    ghost_projectiles: Query<Entity, With<GhostSpellProjectile>>,
    ghost_arcs: Query<Entity, With<GhostSpellArc>>,
) {
    let Some(snapshot) = &latest.0 else { return };
    let Some(assets) = assets else { return };

    // ── Tier 2: Persistent Spell Effects ──────────────────────────────────

    let mut seen_effect_ids = HashSet::with_capacity(snapshot.spell_effects.len());

    for effect in &snapshot.spell_effects {
        seen_effect_ids.insert(effect.net_id);

        if let Some(&local_entity) = effect_map.remote_to_local.get(&effect.net_id) {
            // GreaseFire: update scale each frame (fire spread animation)
            if effect.kind == SpellEffectKind::GreaseFire as u8 {
                if let Ok(mut transform) = effect_transforms.get_mut(local_entity) {
                    transform.scale = Vec3::splat(effect.extra[0].max(0.01));
                }
            }
            continue;
        }

        if let Some(entity) = spawn_spell_effect(
            &mut commands,
            effect,
            &assets,
            &mut materials,
            &mut meshes,
        ) {
            effect_map.insert(effect.net_id, entity);
        }
    }

    // Force-despawn persistent effects no longer in the snapshot
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
            0 => (assets.sphere_mesh.clone(), assets.fireball_projectile_material.clone()),
            1 => (assets.sphere_mesh.clone(), assets.ice_projectile_material.clone()),
            2 => (assets.sphere_mesh.clone(), assets.meteor_projectile_material.clone()),
            _ => continue,
        };

        let scale = match proj.kind {
            0 => 12.0,  // fireball
            1 => 8.0,   // ice
            2 => 10.0,  // meteor
            _ => 8.0,
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(proj.x, proj.y, proj.z))
                .with_scale(Vec3::splat(scale)),
            Billboard,
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
            0 => assets.chain_lightning_material.clone(),
            1 => assets.lightning_strike_material.clone(),
            2 => assets.crystal_beam_material.clone(),
            3 => assets.crystal_arc_material.clone(),
            4 => assets.finger_of_death_material.clone(),
            5 => assets.lightning_rod_arc_material.clone(),
            6 => assets.disintegrate_beam_material.clone(),
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

        // Arc width varies by type
        let width = match arc.kind {
            0 | 3 | 5 => 6.0,   // chain lightning, crystal arc, rod arc
            1 => 8.0,            // lightning strike
            2 | 4 => 20.0,      // crystal beam, finger of death
            6 => 16.0,          // disintegrate beam
            _ => 6.0,
        };

        commands.spawn((
            Mesh3d(assets.rect_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(width, length, 1.0)),
            GhostSpellArc,
            OnMultiplayerGameScreen,
        ));
    }
}

/// Spawns a persistent spell effect entity on the guest with real components.
///
/// Returns the spawned entity, or `None` if the kind is unknown.
/// Zone fade systems modify materials directly, so zones get unique material clones.
/// Some effects (black hole, lightning rod) allocate meshes at spawn since they need
/// specific sizes; these are rare entities so the cost is negligible.
fn spawn_spell_effect(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    assets: &SpellEffectAssets,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) -> Option<Entity> {
    let kind = SpellEffectKind::try_from(effect.kind).ok()?;
    let pos = Vec3::new(effect.x, effect.y, effect.z);
    let extra = effect.extra;

    // Rotation for flat circles (face up on the ground plane)
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    match kind {
        // ── Zones (flat circles, real components, unique materials for fading) ──

        SpellEffectKind::SpikeGrowthZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.spike_growth_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                SpikeGrowthZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 1.0, 0.0, 0.0, duration,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::HealingPlumeZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.healing_plume_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                HealingPlumeZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 1.0, duration,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::EntangleGround => {
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.entangle_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(120.0)), // CIRCLE_RADIUS from entangle constants
                EntangleGroundEffect::new(duration),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::FogCloudZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.fog_cloud_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                FogCloudZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 0.0, 1.0, duration,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::GreaseZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.grease_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                GreaseZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 0.0, 1.0, duration, 0.0, 0.0, 0.0, 1.0,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::GreaseFire => {
            // Fire overlay is a second circle on top of the grease zone.
            // Scale is updated every frame from the snapshot (fire spread animation).
            let scale = extra[0].max(0.01);
            let material = materials.add(materials.get(&assets.grease_fire_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.1, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(scale)),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::PlagueWindCloud => {
            let radius = extra[0];
            let duration = extra[1];
            let speed = extra[2];
            let direction_angle = extra[3];
            let direction = Vec3::new(direction_angle.sin(), 0.0, direction_angle.cos());
            let material = materials.add(materials.get(&assets.plague_wind_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                PlagueWindCloud::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 1.0, duration, speed, direction,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::MeteorGroundFire => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.meteor_ground_fire_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                MeteorGroundFire::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius, 0.0, 1.0, duration,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        // ── Objects (3D meshes, shared materials) ──

        SpellEffectKind::BlackHole => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            // Black hole update system sets scale 0→1 on a pre-sized mesh,
            // so we create a Sphere of the correct size (rare entity, small cost).
            let bh_mesh = meshes.add(Sphere::new(max_radius));
            Some(commands.spawn((
                Mesh3d(bh_mesh),
                MeshMaterial3d(assets.black_hole_material.clone()),
                Transform::from_translation(pos)
                    .with_scale(Vec3::ZERO), // Grows from 0 via update_black_hole_visuals
                BlackHole::new(pos, max_radius, empowerment),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::ArcaneCrystal => {
            let range = extra[0];
            let duration = extra[1];
            let empowerment = extra[2];
            let height = 35.0 * empowerment; // CRYSTAL_HEIGHT * empowerment
            // Crystal update system sets non-uniform scale (0.7, 1.5, 0.7) on a
            // pre-sized mesh, so create a sphere of the correct size.
            let crystal_mesh = meshes.add(Sphere::new(height / 3.0));
            Some(commands.spawn((
                Mesh3d(crystal_mesh),
                MeshMaterial3d(assets.arcane_crystal_material.clone()),
                Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z))
                    .with_scale(Vec3::new(0.7, 1.5, 0.7)),
                crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal::new(
                    Vec3::new(pos.x, height / 2.0, pos.z),
                    range, duration, range * 0.15, empowerment,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::LightningRod => {
            let duration = extra[0];
            let empowerment = extra[1];
            // Lightning rod uses a cylinder mesh; create one at spawn.
            // This is a small allocation but rods are rare (1-2 at most).
            let tower_height = 60.0; // TOWER_HEIGHT
            let tower_radius = 8.0;  // TOWER_RADIUS
            Some(commands.spawn((
                Mesh3d(assets.cuboid_mesh.clone()),
                MeshMaterial3d(assets.lightning_rod_material.clone()),
                Transform::from_translation(Vec3::new(pos.x, tower_height / 2.0, pos.z))
                    .with_scale(Vec3::new(tower_radius * 2.0, tower_height, tower_radius * 2.0)),
                crate::game::units::wizard::spells::lightning_rod::components::LightningRod::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    duration, empowerment,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        // ── Walls ──

        SpellEffectKind::WallOfStone => {
            let half_length = extra[0];
            let half_width = extra[1];
            let height = extra[2];
            let duration = extra[3];
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            // Reconstruct forward/right from rotation
            let forward = rotation * Vec3::X;
            let right = Vec3::new(-forward.z, 0.0, forward.x);
            Some(commands.spawn((
                Mesh3d(assets.cuboid_mesh.clone()),
                MeshMaterial3d(assets.wall_of_stone_material.clone()),
                Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z))
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(half_length * 2.0, height, half_width * 2.0)),
                WallOfStone {
                    center: Vec3::new(pos.x, 0.0, pos.z),
                    half_length,
                    half_width,
                    forward,
                    right,
                    height,
                    time_alive: 0.0,
                    duration,
                    sinking: false,
                    empowerment: 1.0,
                },
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::WallOfFire => {
            let half_width = extra[0];
            let duration = extra[1];
            let wall_length = extra[2];
            let material = materials.add(materials.get(&assets.wall_of_fire_material)?.clone());
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            // Host mesh is Cuboid(1.0, 10.0, 60.0) with scale.x = length.
            // Guest uses unit cuboid with scale matching host visual dimensions.
            let wall_height = 10.0;
            Some(commands.spawn((
                Mesh3d(assets.cuboid_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, wall_height / 2.0, pos.z))
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(wall_length, wall_height, 60.0)),
                WallOfFireEffect::new(
                    Vec3::ZERO, Vec3::ZERO,
                    half_width,
                    0.0, crate::game::units::DamageType::Fire, 1.0, duration,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        // ── Explosions (unit meshes, scale-driven animation) ──

        SpellEffectKind::FireballExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            Some(commands.spawn((
                Mesh3d(assets.sphere_mesh.clone()),
                MeshMaterial3d(assets.fireball_explosion_material.clone()),
                Transform::from_translation(pos)
                    .with_scale(Vec3::splat(0.1)),
                FireballExplosion::new(
                    pos, max_radius, 0.0,
                    crate::game::units::DamageType::Fire,
                    empowerment,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::MeteorExplosion => {
            let max_radius = extra[0];
            let material = materials.add(materials.get(&assets.meteor_explosion_material)?.clone());
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(0.1)),
                MeteorExplosion::new(pos, max_radius, 0.0),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::IceExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            Some(commands.spawn((
                Mesh3d(assets.circle_mesh.clone()),
                MeshMaterial3d(assets.ice_explosion_material.clone()),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(0.1)),
                IceExplosion::new(pos, max_radius, 0.0, empowerment),
                OnMultiplayerGameScreen,
            )).id())
        }
    }
}

/// Listens for `GameOver` messages from the host and transitions to the score screen.
pub fn handle_game_over_message(
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::GameOver(result) => {
                *game_outcome = match result {
                    GameOverResult::HostWins => GameOutcome::DefeatKingDied,
                    GameOverResult::GuestWins => GameOutcome::Victory,
                };
                next_state.set(MultiplayerGameState::ScoreScreen);
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
