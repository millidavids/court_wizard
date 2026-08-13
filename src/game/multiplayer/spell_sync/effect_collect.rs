use bevy::prelude::*;

use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::arcane_crystal::infusions::CrystalInfusion;
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::entangle::components::EntangleGroundEffect;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeZone;
use crate::game::units::wizard::spells::lightning_rod::components::LightningRod;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire,
};
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::squall::components::IceExplosion;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::snapshot::{SpellEffectKind, SpellEffectSnapshot, SpellSnapshotData};

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
                    // extra[3] carries what the crystal is infused with, so the
                    // guest's ghost can tint to match. Updated every frame by the
                    // ArcaneCrystal arm in `ghost_spawn`, since a crystal is always
                    // placed before it absorbs anything and would otherwise be
                    // stuck showing the uninfused colour it had at spawn.
                    (
                        [
                            ac.range,
                            ac.duration,
                            ac.empowerment,
                            CrystalInfusion::as_sync_id(ac.infusion),
                        ],
                        flags,
                    )
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
            // Warglock flamethrower ground fire — a `FireballExplosion` carrying
            // the burning patch. Ship its radius + duration so the opponent
            // renders the matching fire/smoke puffs.
            SpellEffectKind::FlameGroundFire => {
                if let Ok(exp) = explosion_data.0.get(entity) {
                    ([exp.max_radius, exp.duration, exp.damage_per_tick, 0.0], 0)
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
