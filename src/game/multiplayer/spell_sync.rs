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
use crate::game::units::wizard::spells::lightning_bolt::LightningBolt;
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
    AuraBubbleVariant, BeamSnapshot, CastEventKind, CastEventSnapshot, MagicMissileSnapshot,
    MoteMaterial, PoofVariant, SparkMaterial, SpellArcSnapshot, SpellEffectKind,
    SpellEffectSnapshot, SpellProjectileSnapshot, SpellSchoolWire, SpellSoundId,
    SpellVisualSnapshot, UNRELIABLE_SPELL_SNAPSHOT,
};

/// Resource holding the latest received remote spell visual snapshot.
#[derive(Resource, Default)]
pub struct LatestSpellSnapshot(pub Option<SpellVisualSnapshot>);

/// Outgoing one-shot cast VFX events for this tick.
///
/// Casting handlers push to this via the `_synced` VFX wrappers (or
/// `emit_cast_event` directly); `send_spell_visual_snapshot` drains it into
/// the outgoing `SpellVisualSnapshot.cast_events` vector once per send tick.
/// Receiver dispatches via `apply_cast_event`.
///
/// The `_synced` wrappers gate the push behind an `mp_active` flag so
/// single-player runs don't accumulate events that are never drained (the
/// drain system is `run_if(mp_running)`). Toggling happens via
/// `mark_pending_events_mp_active` / `_inactive` on MP state transitions.
#[derive(Resource, Default)]
pub struct PendingCastEvents {
    pub events: Vec<CastEventSnapshot>,
    pub mp_active: bool,
}

/// Marks `PendingCastEvents` as MP-active so `_synced` wrappers start pushing.
pub fn mark_pending_events_mp_active(mut pending: ResMut<PendingCastEvents>) {
    pending.mp_active = true;
}

/// Marks `PendingCastEvents` as MP-inactive and clears any straggler events.
pub fn mark_pending_events_mp_inactive(mut pending: ResMut<PendingCastEvents>) {
    pending.mp_active = false;
    pending.events.clear();
}

/// Collects persistent spell effect entities into the spell visual snapshot.
///
/// Queries all entities with `NetworkedSpellEffect` and builds the spell_effects
/// vector. Uses `NetworkEntityId` if available, otherwise `Entity::index()`.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
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
    extra_data: (
        Query<&crate::game::units::wizard::spells::dispel::components::DispelImpact>,
        Query<&crate::game::units::wizard::spells::squall::components::SquallStorm>,
    ),
    // Brute / Ogre boulder lifecycle queries — `sprite_index` is encoded
    // into `extra[0]` for the guest's asset picker.
    boulder_data: (
        Query<&crate::game::terrain::boulder::components::BoulderProjectile>,
        Query<&crate::game::terrain::boulder::components::Boulder>,
    ),
    // Spells whose talent state is stored as MARKER components on the spell
    // entity (rather than inside a `talent_params` field). Each Has<>
    // returns true if the marker is present on the queried entity.
    talent_markers: (
        // ArcaneCrystal markers: ResonanceCascade, PrismaticExplosion,
        // AutoCrystalTimer, CrystalNetwork
        Query<(
            Has<crate::game::units::wizard::spells::arcane_crystal::components::ResonanceCascade>,
            Has<crate::game::units::wizard::spells::arcane_crystal::components::PrismaticExplosion>,
            Has<crate::game::units::wizard::spells::arcane_crystal::components::AutoCrystalTimer>,
            Has<crate::game::units::wizard::spells::arcane_crystal::components::CrystalNetwork>,
        )>,
        // FogCloud markers: BlindingMistZone, ConcealingVeilZone,
        // DisorientingVaporsZone, PhantomFogZone, ChokingFogZone,
        // RollingFogZone
        Query<(
            Has<crate::game::units::wizard::spells::fog_cloud::components::BlindingMistZone>,
            Has<crate::game::units::wizard::spells::fog_cloud::components::ConcealingVeilZone>,
            Has<crate::game::units::wizard::spells::fog_cloud::components::DisorientingVaporsZone>,
            Has<crate::game::units::wizard::spells::fog_cloud::components::PhantomFogZone>,
            Has<crate::game::units::wizard::spells::fog_cloud::components::ChokingFogZone>,
            Has<crate::game::units::wizard::spells::fog_cloud::components::RollingFogZone>,
        )>,
    ),
) {
    spell_data.spell_effects.clear();

    for (entity, net_id, effect, transform) in &effects {
        let t = transform.translation;
        let rot_y = transform.rotation.to_euler(EulerRot::YXZ).0;

        // Use NetworkEntityId if assigned, otherwise use Entity index
        let id = net_id.map_or(entity.index_u32(), |n| n.0);

        // `(extra, flags)`: extra carries f32 init params, flags packs
        // talent booleans for the spells that have them. Spells that don't
        // ship talent state leave flags at 0.
        let (extra, flags): ([f32; 4], u32) = match effect.kind {
            SpellEffectKind::SpikeGrowthZone => {
                if let Ok(z) = zone_data.0.get(entity) {
                    ([z.base_radius, z.duration, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::HealingPlumeZone => {
                if let Ok(z) = zone_data.1.get(entity) {
                    ([z.radius, z.duration, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::EntangleGround => {
                if let Ok(z) = zone_data.2.get(entity) {
                    ([0.0, z.duration, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::FogCloudZone => {
                if let Ok(z) = zone_data.3.get(entity) {
                    // Pack FogCloud talent marker presence into bits 0..5.
                    let (blind, conceal, disorient, phantom, choking, rolling) =
                        talent_markers.1.get(entity).unwrap_or_default();
                    let mut flags: u32 = 0;
                    if blind {
                        flags |= 1 << 0;
                    }
                    if conceal {
                        flags |= 1 << 1;
                    }
                    if disorient {
                        flags |= 1 << 2;
                    }
                    if phantom {
                        flags |= 1 << 3;
                    }
                    if choking {
                        flags |= 1 << 4;
                    }
                    if rolling {
                        flags |= 1 << 5;
                    }
                    (
                        [
                            z.radius,
                            z.duration,
                            z.evasion_chance,
                            z.evasion_refresh_duration,
                        ],
                        flags,
                    )
                } else {
                    continue;
                }
            }
            SpellEffectKind::GreaseZone => {
                if let Ok(z) = zone_data.4.get(entity) {
                    ([z.radius, z.duration, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::GreaseFire => ([transform.scale.x, 0.0, 0.0, 0.0], 0),
            SpellEffectKind::PlagueWindCloud => {
                if let Ok(c) = zone_data.5.get(entity) {
                    // Pack PlagueWind talent booleans into bits 0..5.
                    let tp = &c.talent_params;
                    let mut flags: u32 = 0;
                    if tp.plague_carrier {
                        flags |= 1 << 0;
                    }
                    if tp.toxic_weakness {
                        flags |= 1 << 1;
                    }
                    if tp.choking_gas {
                        flags |= 1 << 2;
                    }
                    if tp.pandemic {
                        flags |= 1 << 3;
                    }
                    if tp.twin_plumes {
                        flags |= 1 << 4;
                    }
                    if tp.necrotic_rot {
                        flags |= 1 << 5;
                    }
                    (
                        [
                            c.radius,
                            c.duration,
                            c.speed,
                            c.direction.x.atan2(c.direction.z),
                        ],
                        flags,
                    )
                } else {
                    continue;
                }
            }
            SpellEffectKind::MeteorGroundFire => {
                if let Ok(f) = zone_data.6.get(entity) {
                    ([f.radius, f.duration, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::BlackHole => {
                if let Ok(bh) = object_data.0.get(entity) {
                    // Pack BlackHole talent booleans into bits 0..4.
                    let tp = &bh.talent_params;
                    let mut flags: u32 = 0;
                    if tp.event_horizon {
                        flags |= 1 << 0;
                    }
                    if tp.crushing_pressure {
                        flags |= 1 << 1;
                    }
                    if tp.void_siphon {
                        flags |= 1 << 2;
                    }
                    if tp.singularity {
                        flags |= 1 << 3;
                    }
                    if tp.dimensional_rift {
                        flags |= 1 << 4;
                    }
                    ([bh.max_radius, bh.empowerment, 0.0, 0.0], flags)
                } else {
                    continue;
                }
            }
            SpellEffectKind::ArcaneCrystal => {
                if let Ok(ac) = object_data.1.get(entity) {
                    // Pack ArcaneCrystal talent marker presence into bits 0..3.
                    let (has_res, has_prism, has_auto, has_network) =
                        talent_markers.0.get(entity).unwrap_or_default();
                    let mut flags: u32 = 0;
                    if has_res {
                        flags |= 1 << 0;
                    }
                    if has_prism {
                        flags |= 1 << 1;
                    }
                    if has_auto {
                        flags |= 1 << 2;
                    }
                    if has_network {
                        flags |= 1 << 3;
                    }
                    ([ac.range, ac.duration, ac.empowerment, 0.0], flags)
                } else {
                    continue;
                }
            }
            SpellEffectKind::LightningRod => {
                if let Ok(lr) = object_data.2.get(entity) {
                    // Pack LightningRod talent booleans into bits 0..5.
                    let tp = &lr.talent_params;
                    let mut flags: u32 = 0;
                    if tp.chain_reaction {
                        flags |= 1 << 0;
                    }
                    if tp.magnetic_field {
                        flags |= 1 << 1;
                    }
                    if tp.overcharge {
                        flags |= 1 << 2;
                    }
                    if tp.storm_spire {
                        flags |= 1 << 3;
                    }
                    if tp.tesla_coil {
                        flags |= 1 << 4;
                    }
                    if tp.lightning_nexus {
                        flags |= 1 << 5;
                    }
                    ([lr.duration, lr.empowerment, 0.0, 0.0], flags)
                } else {
                    continue;
                }
            }
            SpellEffectKind::WallOfStone => {
                if let Ok(w) = wall_data.0.get(entity) {
                    ([w.half_length, w.half_width, w.height, w.duration], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::WallOfFire => {
                if let Ok(w) = wall_data.1.get(entity) {
                    // Pack WallOfFire talent booleans into bits 0..5.
                    let tp = &w.talent_params;
                    let mut flags: u32 = 0;
                    if tp.searing_heat {
                        flags |= 1 << 0;
                    }
                    if tp.scorched_earth {
                        flags |= 1 << 1;
                    }
                    if tp.spreading_flames {
                        flags |= 1 << 2;
                    }
                    if tp.firestorm {
                        flags |= 1 << 3;
                    }
                    if tp.twin_walls {
                        flags |= 1 << 4;
                    }
                    if tp.consuming_inferno {
                        flags |= 1 << 5;
                    }
                    ([w.half_width, w.duration, transform.scale.x, 0.0], flags)
                } else {
                    continue;
                }
            }
            SpellEffectKind::FireballExplosion => {
                if let Ok(e) = explosion_data.0.get(entity) {
                    ([e.max_radius, e.empowerment, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::MeteorExplosion => {
                if let Ok(e) = explosion_data.1.get(entity) {
                    ([e.max_radius, 0.0, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::IceExplosion => {
                if let Ok(e) = explosion_data.2.get(entity) {
                    ([e.max_radius, e.empowerment, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::DispelImpact => {
                if let Ok(d) = extra_data.0.get(entity) {
                    ([d.duration, d.expand_speed, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::SquallStorm => {
                if let Ok(s) = extra_data.1.get(entity) {
                    // Pack SquallStorm talent booleans into bits 0..5.
                    let tp = &s.talent_params;
                    let mut flags: u32 = 0;
                    if tp.permafrost {
                        flags |= 1 << 0;
                    }
                    if tp.hailstones {
                        flags |= 1 << 1;
                    }
                    if tp.sleet_storm {
                        flags |= 1 << 2;
                    }
                    if tp.absolute_zero {
                        flags |= 1 << 3;
                    }
                    if tp.blizzard {
                        flags |= 1 << 4;
                    }
                    if tp.ice_age {
                        flags |= 1 << 5;
                    }
                    ([s.radius, 0.0, 0.0, 0.0], flags)
                } else {
                    continue;
                }
            }
            SpellEffectKind::ScorchedEarthFire | SpellEffectKind::NapalmTrail => {
                if let Ok(e) = explosion_data.0.get(entity) {
                    ([e.max_radius, e.empowerment, e.duration, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::BoulderProjectileEffect => {
                if let Ok(p) = boulder_data.0.get(entity) {
                    ([p.sprite_index as f32, 0.0, 0.0, 0.0], 0)
                } else {
                    continue;
                }
            }
            SpellEffectKind::BoulderObstacle => {
                if let Ok(b) = boulder_data.1.get(entity) {
                    ([b.sprite_index as f32, b.radius, b.height, 0.0], 0)
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
            flags,
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
    lightning_strikes: Query<(&LightningStrike, &LightningBolt)>,
    lightning_rod_arcs: Query<&LightningRodArc>,
    fod_beams: Query<&FingerOfDeathBeam>,
    dispel_projectiles: Query<
        &Transform,
        With<crate::game::units::wizard::spells::dispel::components::DispelProjectile>,
    >,
) {
    spell_data.spell_projectiles.clear();
    spell_data.spell_arcs.clear();

    // Read the visual radius the local caster used straight from
    // `Transform.scale` — both SP and ghost spawns set this via
    // `.with_scale(Vec3::splat(visual_radius))`, so it's a faithful
    // capture of talent + empowerment scaling without re-deriving.
    for t in &fireballs {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 0,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &ice_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 1,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &meteor_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 2,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &dispel_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 3,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
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

    for (strike, bolt) in &lightning_strikes {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 1,
            ox: bolt.end.x,
            oy: bolt.end.y,
            oz: bolt.end.z,
            tx: strike.target_pos.x,
            ty: strike.target_pos.y,
            tz: strike.target_pos.z,
        });
    }

    // Crystal beams are now DisintegrateBeam entities and crystal arcs are
    // ChainLightningArc entities. ChainLightningArc ships above as kind=0; every
    // DisintegrateBeam (real disintegrate + crystal beam) ships via the dedicated
    // `BeamSnapshot` path in `send_spell_visual_snapshot` (carrying width), so no
    // duplicate kind=6 arc is emitted here.

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

    for arc in &lightning_rod_arcs {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 5,
            ox: arc.start.x,
            oy: arc.start.y,
            oz: arc.start.z,
            tx: arc.end.x,
            ty: arc.end.y,
            tz: arc.end.z,
        });
    }
}

/// Collects magic missiles and beams, then serializes and sends the complete
/// spell visual snapshot over the unreliable channel.
pub fn send_spell_visual_snapshot(
    mut connection: ResMut<NetworkConnection>,
    mut spell_data: ResMut<SpellSnapshotData>,
    mut pending_cast_events: ResMut<PendingCastEvents>,
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
                width: beam.beam_width(),
            })
            .collect(),
        // Drain one-shot cast events accumulated by casting handlers this
        // tick. Casting handlers run before `send_spell_visual_snapshot` in
        // Bevy's default `Update` schedule, so events emitted on cast
        // completion (e.g. `spawn_school_flare_synced` on fireball release)
        // ship on the same frame.
        cast_events: std::mem::take(&mut pending_cast_events.events),
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

    // Deserialize the latest spell snapshot if a new one arrived this frame.
    // We do NOT reset `latest.0` on frames without new data — keeping the
    // previously-deserialized snapshot lets `apply_remote_spell_snapshot`
    // re-render ghost arcs every frame with fresh random jitter (matching
    // the local caster's per-frame `update_lightning_bolts` crackle).
    // Without persistence the bolts only redraw at the network snapshot
    // rate and visibly stutter between updates.
    //
    // One-shot `cast_events` (school flares, SFX, etc.) must NOT re-fire
    // each frame, so on stale frames we keep `latest.0` but empty its
    // `cast_events` list — leaving everything else (effects, projectiles,
    // arcs, missiles, beams) intact for per-frame re-rendering.
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
    } else if let Some(s) = latest.0.as_mut() {
        s.cast_events.clear();
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
    // `BoulderAssets` is unconditionally inserted at Startup by `BoulderPlugin`
    // so it's not wrapped in `Option`. If it ever stops being available a
    // system-validation panic surfaces the issue immediately rather than
    // silently disabling the whole remote-spell render path.
    boulder_assets: Res<crate::game::terrain::boulder::resources::BoulderAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sphere_materials: ResMut<
        Assets<crate::game::units::wizard::spells::visual_assets::FireExplosionSphereMaterial>,
    >,
    mut effect_transforms: Query<&mut Transform>,
    mut plague_clouds: Query<
        &mut crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud,
    >,
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
            if effect.kind == SpellEffectKind::GreaseFire as u8 {
                if let Ok(mut transform) = effect_transforms.get_mut(local_entity) {
                    transform.scale = Vec3::splat(effect.extra[0].max(0.01));
                }
            } else if effect.kind == SpellEffectKind::PlagueWindCloud as u8 {
                // Plague clouds MOVE with the wind — sync the host's live position
                // into the ghost each frame so it drifts instead of sitting static.
                if let Ok(mut transform) = effect_transforms.get_mut(local_entity) {
                    transform.translation.x = effect.x;
                    transform.translation.z = effect.z;
                }
                if let Ok(mut cloud) = plague_clouds.get_mut(local_entity) {
                    cloud.origin.x = effect.x;
                    cloud.origin.z = effect.z;
                }
            }
            continue;
        }

        if let Some(entity) = super::guest_systems::spawn_spell_effect(
            &mut commands,
            effect,
            &assets,
            &mut materials,
            &mut sphere_materials,
            &boulder_assets,
        ) {
            // Tag every ghost spell-effect so SP gameplay systems on the
            // guest can filter them out via `Without<GhostSpellEffect>` —
            // prevents Category-C double-application (BlackHole pull,
            // ArcaneCrystal mini-spell fire, LightningRod strikes,
            // PlagueWind DPS all running independently on both peers).
            //
            // Also attach the host's `NetworkEntityId` so guest-side dispel
            // (and any future host-authoritative spell-effect message) can
            // reference the host's entity by ID.
            commands.entity(entity).insert((
                crate::game::multiplayer::components::GhostSpellEffect,
                NetworkEntityId(effect.net_id),
            ));
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
            ec.try_despawn();
        }
    }

    // ── Tier 1: Ephemeral Spell Projectiles ──────────────────────────────

    for entity in &ghost_projectiles {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }

    for proj in &snapshot.spell_projectiles {
        let pos = Vec3::new(proj.x, proj.y, proj.z);
        // Fireball uses the SP spawn helper so the receiver gets the exact
        // same mesh + material + glow sibling that the caster sees. Ice and
        // meteor still fall back to the slim mesh+material spawn — they'll
        // be migrated to their own shared visual helpers in subsequent
        // iterations of this refactor.
        match proj.kind {
            0 => {
                // `spawn_fireball_visuals` tags BOTH the parent sphere and
                // the glow halo sibling with `OnMultiplayerGameScreen`, so
                // both are cleaned up by `cleanup_mp_game`.
                let entity =
                    crate::game::units::wizard::spells::fireball::casting::spawn_fireball_visuals(
                        &mut commands,
                        &assets,
                        pos,
                        proj.scale.max(0.01),
                        OnMultiplayerGameScreen,
                    );
                commands.entity(entity).insert(GhostSpellProjectile);
            }
            1 => {
                commands.spawn((
                    Mesh3d(assets.cross_plane_sphere.clone()),
                    MeshMaterial3d(assets.ice_projectile.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(proj.scale.max(0.01))),
                    GhostSpellProjectile,
                    OnMultiplayerGameScreen,
                ));
            }
            2 => {
                commands.spawn((
                    Mesh3d(assets.cross_plane_sphere.clone()),
                    MeshMaterial3d(assets.meteor_projectile.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(proj.scale.max(0.01))),
                    GhostSpellProjectile,
                    OnMultiplayerGameScreen,
                ));
            }
            3 => {
                // DispelProjectile ghost — uses the existing dispel_spark
                // material on the same cross-plane sphere mesh. SP-side
                // visuals (the bolt cluster, trailing particles) are
                // local-only; the ghost is a single sphere at the
                // projectile's current XYZ, which is enough for the remote
                // player to see the bolt fly toward its target.
                commands.spawn((
                    Mesh3d(assets.cross_plane_sphere.clone()),
                    MeshMaterial3d(assets.dispel_spark.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(proj.scale.max(0.01))),
                    GhostSpellProjectile,
                    OnMultiplayerGameScreen,
                ));
            }
            _ => continue,
        }
    }

    // ── Tier 1: Ephemeral Spell Arcs ─────────────────────────────────────

    for entity in &ghost_arcs {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
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
            // (Disintegrate ships via the dedicated `BeamSnapshot` path now —
            // the old kind=6 arc is no longer emitted.)
            _ => continue,
        };

        let origin = Vec3::new(arc.ox, arc.oy, arc.oz);
        let target = Vec3::new(arc.tx, arc.ty, arc.tz);
        let diff = target - origin;
        let length = diff.length();
        if length < 0.1 {
            continue;
        }

        // Lightning arcs (chain lightning = 0, descending strike = 1, rod
        // ground arc = 5) need the jagged segmented geometry, not a flat
        // quad. Spawn the segment quads directly here — we do NOT use a
        // `LightningBolt` parent on the receiver because that creates a
        // race with `update_lightning_bolts` (which would queue `ChildOf`
        // commands referencing parents we despawn the same frame, panicking
        // on hierarchy flush). Each snapshot frame fully respawns the
        // ghost segments with fresh random jitter, giving the same
        // crackling appearance.
        if matches!(arc.kind, 0 | 1 | 5) {
            use crate::game::units::wizard::spells::lightning_bolt::generate_jagged_path;
            let (segments, jitter, width) = match arc.kind {
                0 => (16u32, 15.0, 3.0),
                1 => (24u32, 18.0, 8.0),
                5 => (14u32, 10.0, 6.0),
                _ => unreachable!(),
            };
            let mut rng = rand::rng();
            let path = generate_jagged_path(origin, target, segments, jitter, 0.0, &mut rng);
            for window in path.windows(2) {
                let p0 = window[0];
                let p1 = window[1];
                let seg = p1 - p0;
                let seg_len = seg.length();
                if seg_len < 1e-3 {
                    continue;
                }
                let seg_dir = seg / seg_len;
                let midpoint = (p0 + p1) * 0.5;
                let rotation = Quat::from_rotation_arc(Vec3::Y, seg_dir);
                commands.spawn((
                    Mesh3d(assets.unit_rect.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(midpoint)
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(width, seg_len, width)),
                    GhostSpellArc,
                    OnMultiplayerGameScreen,
                ));
            }
            continue;
        }

        let direction = diff / length;
        let midpoint = origin + diff * 0.5;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        let width = match arc.kind {
            3 => 6.0,
            // Finger of Death uses its SP `BEAM_WIDTH` so the ghost core + glow
            // are the same girth the caster sees (crystal beams stay at 20).
            4 => crate::game::units::wizard::spells::finger_of_death::constants::BEAM_WIDTH,
            2 => 20.0,
            _ => 6.0,
        };

        // Beam-type arcs (finger of death=4) use the cross-plane cylinder mesh.
        // (Disintegrate now ships via the dedicated `BeamSnapshot` path, not an arc.)
        let is_beam = arc.kind == 4;
        let mesh = if is_beam {
            assets.cross_plane_cylinder.clone()
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

        // Finger of Death: add the glow halo + origin flare siblings so the
        // ghost matches the SP `spawn_beam` visuals (the SP per-frame glow/
        // flare systems don't run on the ghost — it has no `FingerOfDeathBeam`
        // component). Tagged `GhostSpellArc` so the per-frame arc clear above
        // despawns them too — no accumulation. The glow reuses the core's
        // cylinder mesh so it stays aligned with the beam.
        if arc.kind == 4 {
            use crate::game::units::wizard::spells::finger_of_death::constants as fod;
            let glow_width = width * fod::GLOW_WIDTH_MULTIPLIER;
            commands.spawn((
                Mesh3d(assets.cross_plane_cylinder.clone()),
                MeshMaterial3d(assets.finger_of_death_glow.clone()),
                Transform::from_translation(midpoint)
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(glow_width, length, glow_width)),
                GhostSpellArc,
                OnMultiplayerGameScreen,
            ));
            commands.spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.finger_of_death_flare.clone()),
                Transform::from_translation(origin).with_scale(Vec3::splat(fod::FLARE_RADIUS)),
                GhostSpellArc,
                OnMultiplayerGameScreen,
            ));
        }
    }

    // ── Ephemeral Magic Missiles ─────────────────────────────────────────

    for entity in &ghost_missiles {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
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
            ec.try_despawn();
        }
    }

    for beam in &snapshot.beams {
        let origin = Vec3::new(beam.ox, beam.oy, beam.oz);
        let direction = Vec3::new(beam.dx, beam.dy, beam.dz);
        let length = beam.length;

        let midpoint = origin + direction * (length / 2.0);
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        // Core beam — `disintegrate_cone` mesh to match SP `spawn_beam_core`
        // (placed at the midpoint, same as SP).
        //
        // We deliberately do NOT attach a real `DisintegrateBeam` component to
        // the ghost. Every SP disintegrate system queries `DisintegrateBeam`
        // with no ghost filter: `update_beam_visuals` would recompute the
        // transform from a fresh `time_alive = 0` component (`current_length()`
        // returns 0) and collapse the beam to a tiny stub at the origin, and
        // `cleanup_beams_on_cancel` would despawn it every frame the LOCAL
        // wizard is resting (the guest's normal state). The ghost is purely
        // snapshot-driven instead; tree/bush ignition is host-authoritative.
        let core_width = beam.width * 0.7;
        commands.spawn((
            Mesh3d(assets.disintegrate_cone.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                // Match SP `update_beam_visuals`: core width is `beam_width() * 0.7`.
                .with_scale(Vec3::new(core_width, length, core_width)),
            GhostBeam,
            OnMultiplayerGameScreen,
        ));

        // Glow halo + origin flare siblings (match SP `spawn_beam_visuals`).
        // Tagged `GhostBeam` so the per-frame beam clear above despawns them
        // too. The ground eclipse is skipped — it needs the local wizard's
        // spell-range geometry, which the ghost has no access to.
        use crate::game::units::wizard::spells::disintegrate::constants as disint;
        let glow_width = beam.width * disint::GLOW_WIDTH_MULTIPLIER * 0.7;
        commands.spawn((
            Mesh3d(assets.disintegrate_cone.clone()),
            MeshMaterial3d(assets.disintegrate_glow.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(glow_width, length, glow_width)),
            GhostBeam,
            OnMultiplayerGameScreen,
        ));
        commands.spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(assets.disintegrate_flare.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(disint::FLARE_RADIUS)),
            GhostBeam,
            OnMultiplayerGameScreen,
        ));
    }
}

/// Dispatches the remote peer's one-shot cast VFX events into local spawn
/// calls. Runs after `apply_remote_spell_snapshot` and reads the same
/// `LatestSpellSnapshot` resource.
///
/// Each event maps to one of the existing `vfx::systems::spawn_*` helpers;
/// the spawned entities tag `OnGameplayScreen` so MP cleanup
/// (`cleanup_game` on `OnExit(AppState::MultiplayerGame)`) catches them.
pub fn apply_remote_cast_events(
    mut commands: Commands,
    latest: Res<LatestSpellSnapshot>,
    assets: Option<Res<SpellVisualAssets>>,
    mut sphere_materials: ResMut<
        Assets<crate::game::units::wizard::spells::visual_assets::FireExplosionSphereMaterial>,
    >,
    time: Res<Time>,
    game_config: Res<crate::config::GameConfig>,
    sfx_assets: Option<Res<crate::game::units::wizard::spells::audio::SpellSfxAssets>>,
) {
    let Some(snapshot) = &latest.0 else { return };
    let Some(assets) = assets else { return };
    if snapshot.cast_events.is_empty() {
        return;
    }

    use crate::game::units::wizard::spells::banishment::components::BanishmentVfx;
    use crate::game::units::wizard::spells::vfx::systems as vfx;

    let now = time.elapsed_secs();

    for event in &snapshot.cast_events {
        let pos = Vec3::new(event.x, event.y, event.z);
        let Ok(kind) = CastEventKind::try_from(event.kind) else {
            continue;
        };
        match kind {
            CastEventKind::SchoolFlare => {
                let Ok(school_wire) = SpellSchoolWire::try_from(event.subkind) else {
                    continue;
                };
                let school = match school_wire {
                    SpellSchoolWire::Fire => vfx::SpellSchool::Fire,
                    SpellSchoolWire::Lightning => vfx::SpellSchool::Lightning,
                    SpellSchoolWire::Arcane => vfx::SpellSchool::Arcane,
                    SpellSchoolWire::Nature => vfx::SpellSchool::Nature,
                    SpellSchoolWire::Holy => vfx::SpellSchool::Holy,
                    SpellSchoolWire::Dark => vfx::SpellSchool::Dark,
                    SpellSchoolWire::Force => vfx::SpellSchool::Force,
                    SpellSchoolWire::Transmutation => vfx::SpellSchool::Transmutation,
                };
                vfx::spawn_school_flare(&mut commands, &assets, pos, school, now);
            }
            CastEventKind::AuraBubble | CastEventKind::AuraBubbleContract => {
                let Ok(variant) = AuraBubbleVariant::try_from(event.subkind) else {
                    continue;
                };
                let material = aura_material_handle(&assets, variant);
                let radius = event.extra[0].max(0.01);
                let duration = event.extra[1].max(0.01);
                if kind == CastEventKind::AuraBubbleContract {
                    vfx::spawn_aura_bubble_contracting(
                        &mut commands,
                        &assets,
                        material,
                        pos,
                        radius,
                        duration,
                    );
                } else {
                    vfx::spawn_aura_bubble(&mut commands, &assets, material, pos, radius, duration);
                }
            }
            CastEventKind::SmokePoof => {
                let Ok(variant) = PoofVariant::try_from(event.subkind) else {
                    continue;
                };
                let material = match variant {
                    PoofVariant::Banishment => assets.banishment_poof.clone(),
                    PoofVariant::Polymorph => assets.polymorph_poof.clone(),
                };
                // `extra[0]` carries the caller-supplied count so host /
                // guest spawn the same number of puffs. Clamp defensively
                // against corrupt packets.
                let count = (event.extra[0] as usize).clamp(1, 64);
                vfx::spawn_smoke_poof(&mut commands, &assets, &material, pos, count, now);
            }
            CastEventKind::FloatingMotes => {
                let Ok(material_kind) = MoteMaterial::try_from(event.subkind) else {
                    continue;
                };
                let material = match material_kind {
                    MoteMaterial::Healing => assets.healing_mote.clone(),
                    MoteMaterial::Nature => assets.nature_mote.clone(),
                    MoteMaterial::Sleep => assets.sleep_mote.clone(),
                };
                let radius = event.extra[0].max(0.01);
                let count = (event.extra[1] as usize).clamp(1, 200);
                vfx::spawn_floating_motes(
                    &mut commands,
                    &assets,
                    &material,
                    pos,
                    radius,
                    count,
                    now,
                );
            }
            CastEventKind::Sparks => {
                let Ok(material_kind) = SparkMaterial::try_from(event.subkind) else {
                    continue;
                };
                let material = match material_kind {
                    SparkMaterial::Banishment => assets.banishment_spark.clone(),
                    SparkMaterial::Dispel => assets.dispel_spark.clone(),
                };
                // `extra[0]` carries the caller-supplied count.
                let count = (event.extra[0] as usize).clamp(1, 64);
                vfx::spawn_sparks_with_material(&mut commands, &assets, pos, count, now, material);
            }
            CastEventKind::DustSmoke => {
                // half_width / count default to the SP wall-of-stone collapse
                // (8.0 / 14). Distinguish "not provided" (sentinel < 0) from
                // a legitimately small/zero value the caller passed.
                let half_width = if event.extra[0] >= 0.0 {
                    event.extra[0]
                } else {
                    8.0
                };
                // Clamp count against malformed packets — FloatingMotes
                // uses the same defensive bound.
                let count = if event.extra[1] > 0.0 {
                    (event.extra[1] as usize).clamp(1, 200)
                } else {
                    14
                };
                vfx::spawn_dust_smoke(&mut commands, &assets, pos, half_width, count, now);
            }
            CastEventKind::FinalStandExplosion => {
                // Berserker Rage's Final Stand detonation — spawn the same
                // FinalStandExplosionVfx component the host spawns so the
                // guest's `update_final_stand_vfx` (gated
                // `is_spell_effects_active`) animates it identically. The
                // material is cloned per-instance so the time uniform
                // animates independently from other concurrent explosions.
                use crate::game::units::wizard::spells::berserker_rage::components::FinalStandExplosionVfx;
                use crate::game::units::wizard::spells::spell_materials::clone_sphere_material;
                let max_radius = event.extra[0].max(0.01);
                let lifetime = event.extra[1].max(0.05);
                let mat_handle =
                    clone_sphere_material(&mut sphere_materials, &assets.fireball_explosion_sphere);
                commands.spawn((
                    FinalStandExplosionVfx {
                        time_alive: 0.0,
                        max_radius,
                        lifetime,
                    },
                    Mesh3d(assets.explosion_sphere.clone()),
                    MeshMaterial3d(mat_handle),
                    Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
                    crate::game::components::OnGameplayScreen,
                ));
            }
            CastEventKind::BanishmentLens => {
                // Spawn the BanishmentVfx component so `update_banishment_vfx`
                // (gated `is_spell_effects_active` after Phase 1) animates
                // the lensing-sphere collapse on the guest. Uses the shared
                // `cross_plane_sphere` asset to match the SP path's mesh
                // shape exactly — `banishment_lens` material is designed
                // for the cross-plane fans, not a tessellated sphere.
                let radius = event.extra[0].max(0.01);
                let duration = event.extra[1].max(0.05);
                commands.spawn((
                    BanishmentVfx {
                        time_alive: 0.0,
                        lifetime: duration,
                        start_radius: radius,
                    },
                    Mesh3d(assets.cross_plane_sphere.clone()),
                    MeshMaterial3d(assets.banishment_lens.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(radius)),
                    crate::game::components::OnGameplayScreen,
                ));
            }
            CastEventKind::SfxOneShot => {
                let Some(ref sfx) = sfx_assets else { continue };
                let Ok(sound_id) = SpellSoundId::try_from(event.subkind) else {
                    continue;
                };
                let volume_scale = if event.extra[0] > 0.0 {
                    event.extra[0]
                } else {
                    1.0
                };
                crate::game::units::wizard::spells::audio::play_remote_sfx(
                    &mut commands,
                    sound_id,
                    pos,
                    volume_scale,
                    &game_config,
                    sfx.as_ref(),
                );
            }
        }
    }
}

/// Maps a wire-protocol `AuraBubbleVariant` to its `SpellVisualAssets` handle.
fn aura_material_handle(
    assets: &SpellVisualAssets,
    variant: AuraBubbleVariant,
) -> Handle<crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial> {
    match variant {
        AuraBubbleVariant::Guardian => assets.guardian_aura_sphere.clone(),
        AuraBubbleVariant::BattleHymn => assets.battle_hymn_aura_sphere.clone(),
        AuraBubbleVariant::Haste => assets.haste_aura_sphere.clone(),
        AuraBubbleVariant::Berserker => assets.berserker_aura_sphere.clone(),
        AuraBubbleVariant::Sleep => assets.sleep_aura_sphere.clone(),
        AuraBubbleVariant::RaiseDead => assets.raise_dead_aura_sphere.clone(),
        AuraBubbleVariant::Teleport => assets.teleport_aura_sphere.clone(),
    }
}

use crate::networking::snapshot::SpellSnapshotData;
