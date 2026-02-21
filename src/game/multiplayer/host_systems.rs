//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send state snapshots to the guest every frame.

use bevy::prelude::*;

use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::components::Arrow;
use crate::game::units::archer::Archer;
use crate::game::units::components::{Corpse, Health, KingsGuard, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::spells::arcane_crystal::components::{
    ArcaneCrystal, CrystalBeam, CrystalLightningArc,
};
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
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    ArrowSnapshot, BeamSnapshot, GameSnapshot, MagicMissileSnapshot, SnapshotTick,
    SpellArcSnapshot, SpellEffectKind, SpellEffectSnapshot, SpellProjectileSnapshot,
    SpellSnapshotData, build_unit_snapshot,
};
use crate::state::MultiplayerGameState;

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// or `NetworkedSpellEffect` but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    new_units: Query<Entity, (With<Health>, With<Team>, Without<NetworkEntityId>)>,
    new_effects: Query<Entity, (With<NetworkedSpellEffect>, Without<NetworkEntityId>)>,
) {
    for entity in new_units.iter().chain(new_effects.iter()) {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}

/// Serializes the full game state and sends it over the unreliable channel.
///
/// Runs every frame (~60Hz). Queries all entities with a `NetworkEntityId` and
/// builds a compact `GameSnapshot` serialized with `bincode`.
#[allow(clippy::type_complexity)]
pub fn send_state_snapshots(
    mut tick: ResMut<SnapshotTick>,
    mut connection: ResMut<NetworkConnection>,
    mut spell_data: ResMut<SpellSnapshotData>,
    units: Query<(
        &NetworkEntityId,
        &Transform,
        &Team,
        &Health,
        Has<Corpse>,
        Has<King>,
        Has<Archer>,
        Has<KingsGuard>,
    )>,
    arrows: Query<&Transform, With<Arrow>>,
    missiles: Query<&Transform, With<MagicMissile>>,
    beams: Query<&DisintegrateBeam>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
        arrows: Vec::with_capacity(arrows.iter().len()),
        magic_missiles: Vec::with_capacity(missiles.iter().len()),
        beams: Vec::with_capacity(beams.iter().len()),
        spell_effects: std::mem::take(&mut spell_data.spell_effects),
        spell_projectiles: std::mem::take(&mut spell_data.spell_projectiles),
        spell_arcs: std::mem::take(&mut spell_data.spell_arcs),
    };

    for (net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard) in &units {
        snapshot.units.push(build_unit_snapshot(
            net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard,
        ));
    }

    for transform in &arrows {
        snapshot.arrows.push(ArrowSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    for transform in &missiles {
        snapshot.magic_missiles.push(MagicMissileSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    for beam in &beams {
        snapshot.beams.push(BeamSnapshot {
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            dx: beam.direction.x,
            dy: beam.direction.y,
            dz: beam.direction.z,
            length: beam.current_length(),
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        connection.outgoing_unreliable.push(data);
    }
}

/// Collects persistent spell effect entities into the `SpellSnapshotData` resource.
///
/// Run before `collect_spell_projectile_snapshots` and `send_state_snapshots` in a chain.
/// Queries all entities with `NetworkedSpellEffect` + `NetworkEntityId` and builds
/// the `spell_effects` vector.
#[allow(clippy::type_complexity)]
pub fn collect_spell_effect_snapshots(
    mut spell_data: ResMut<SpellSnapshotData>,
    effects: Query<(Entity, &NetworkEntityId, &NetworkedSpellEffect, &Transform)>,
    // Kind-specific data queries (nested tuple for param limit)
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
            SpellEffectKind::GreaseFire => {
                // Send the current scale (fire overlay grows over time)
                [transform.scale.x, 0.0, 0.0, 0.0]
            }
            SpellEffectKind::PlagueWindCloud => {
                if let Ok(c) = zone_data.5.get(entity) {
                    [c.radius, c.duration, c.speed, c.direction.x.atan2(c.direction.z)]
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
                    // extra[2] = wall length from the Transform's X scale
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
            net_id: net_id.0,
            kind: effect.kind as u8,
            x: t.x,
            y: t.y,
            z: t.z,
            rotation_y: rot_y,
            extra,
        });
    }
}

/// Collects ephemeral spell projectiles and arcs into the `SpellSnapshotData` resource.
///
/// Run after `collect_spell_effect_snapshots` and before `send_state_snapshots`.
#[allow(clippy::type_complexity)]
pub fn collect_spell_projectile_snapshots(
    mut spell_data: ResMut<SpellSnapshotData>,
    // Ephemeral projectiles
    fireballs: Query<&Transform, With<Fireball>>,
    ice_projectiles: Query<&Transform, With<IceProjectile>>,
    meteor_projectiles: Query<&Transform, With<MeteorProjectile>>,
    // Ephemeral arcs/beams
    chain_arcs: Query<&ChainLightningArc>,
    lightning_strikes: Query<(&LightningStrike, &Transform)>,
    lightning_rod_arcs: Query<(&LightningRodArc, &Transform)>,
    crystal_beams: Query<&CrystalBeam>,
    crystal_arcs: Query<(&CrystalLightningArc, &Transform)>,
    fod_beams: Query<&FingerOfDeathBeam>,
) {
    spell_data.spell_projectiles.clear();
    spell_data.spell_arcs.clear();

    // Fireball projectiles
    for t in &fireballs {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 0,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    // Ice projectiles
    for t in &ice_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 1,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    // Meteor projectiles
    for t in &meteor_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 2,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        });
    }

    // Chain lightning arcs
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

    // Lightning strikes
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

    // Crystal beams
    for beam in &crystal_beams {
        let end = beam.origin + beam.direction * beam.current_length();
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 2,
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }

    // Crystal lightning arcs (need Transform for position data)
    for (_arc, transform) in &crystal_arcs {
        // Crystal arcs are rendered as rectangles positioned at midpoint with rotation.
        // We store the transformed position as-is for the guest to reconstruct.
        let pos = transform.translation;
        let scale = transform.scale;
        let up = transform.rotation * Vec3::Y;
        let half_len = scale.y * 0.5;
        let start = pos - up * half_len;
        let end = pos + up * half_len;
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 3,
            ox: start.x,
            oy: start.y,
            oz: start.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }

    // Finger of death beams
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

    // Lightning rod arcs
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
}

/// Checks for King death and triggers game-over transition.
///
/// Host = Defenders, Guest = Attackers. When a King becomes a corpse:
/// - Defender King dies → guest wins
/// - Attacker King dies → host wins
pub fn check_mp_king_death(
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    dead_kings: Query<&Team, (With<King>, With<Corpse>)>,
) {
    if let Some(team) = dead_kings.iter().next() {
        let result = match team {
            Team::Defenders => GameOverResult::GuestWins,
            Team::Attackers | Team::Undead => GameOverResult::HostWins,
        };

        *game_outcome = match result {
            GameOverResult::HostWins => GameOutcome::Victory,
            GameOverResult::GuestWins => GameOutcome::DefeatKingDied,
        };

        connection
            .outgoing_messages
            .push(NetworkMessage::GameOver(result));
        next_state.set(MultiplayerGameState::ScoreScreen);
    }
}
