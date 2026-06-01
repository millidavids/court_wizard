//! Guest visual: pick_material, spawn_spell_effect, handle_game_over.

use bevy::prelude::*;

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
use crate::game::units::wizard::spells::spike_growth::components::{
    SpikeGrowthTalentParams, SpikeGrowthZone,
};
use crate::game::units::wizard::spells::squall::components::IceExplosion;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{SpellEffectKind, SpellEffectSnapshot};
use crate::state::MultiplayerGameState;

use super::components::OnMultiplayerGameScreen;
// Note: `GhostSpellEffect` is inserted by the caller in `spell_sync.rs`
// after `spawn_spell_effect` returns the entity, so this file doesn't
// import it.
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};

/// Receives the latest unit state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
///
/// Filters incoming unreliable data by type prefix byte, processing only game
/// snapshots (unit data). Spell visual snapshots are handled by `spell_sync.rs`.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(super) fn pick_material(
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    materials: &mut Assets<StandardMaterial>,
    team: crate::game::units::components::Team,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
) -> Handle<StandardMaterial> {
    use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
    use crate::game::units::systems::{corpse_material_for_team, create_default_sprite_material};

    if is_corpse {
        let idx = rand::random_range(0..CORPSE_MATERIAL_VARIANTS);
        if is_king {
            king_assets.corpse_materials[idx].clone()
        } else if is_archer {
            corpse_material_for_team(
                &archer_assets.defender_corpse_materials,
                &archer_assets.attacker_corpse_materials,
                &archer_assets.undead_corpse_materials,
                team,
                idx,
            )
        } else {
            corpse_material_for_team(
                &infantry_assets.defender_corpse_materials,
                &infantry_assets.attacker_corpse_materials,
                &infantry_assets.undead_corpse_materials,
                team,
                idx,
            )
        }
    } else if is_king {
        use crate::game::units::king::constants::KING_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            king_assets.sprite_texture.clone(),
            KING_SPRITE_TINT,
        )
    } else if is_archer {
        let tint = crate::game::units::systems::archer_sprite_tint_for_team(team);
        create_default_sprite_material(materials, archer_assets.sprite_texture.clone(), tint)
    } else if is_guard {
        use crate::game::units::infantry::constants::KINGS_GUARD_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            infantry_assets.sprite_texture.clone(),
            KINGS_GUARD_SPRITE_TINT,
        )
    } else {
        let tint = crate::game::units::systems::sprite_tint_for_team(team);
        create_default_sprite_material(materials, infantry_assets.sprite_texture.clone(), tint)
    }
}

/// Spawns a persistent spell effect entity with real components.
///
/// Returns the spawned entity, or `None` if the kind is unknown.
/// Zone fade systems modify materials directly, so zones get unique material clones.
/// Some effects (black hole, lightning rod) allocate meshes at spawn since they need
/// specific sizes; these are rare entities so the cost is negligible.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_spell_effect(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    boulder_assets: &crate::game::terrain::boulder::resources::BoulderAssets,
) -> Option<Entity> {
    let kind = SpellEffectKind::try_from(effect.kind).ok()?;
    let pos = Vec3::new(effect.x, effect.y, effect.z);
    let extra = effect.extra;
    let flags = effect.flags;

    // Rotation for flat circles (face up on the ground plane)
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    match kind {
        // ── Zones (flat circles, real components, unique materials for fading) ──
        SpellEffectKind::SpikeGrowthZone => {
            let radius = extra[0];
            let duration = extra[1];
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                        SpikeGrowthZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            0.0,
                            0.0,
                            duration,
                            SpikeGrowthTalentParams::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::HealingPlumeZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.healing_plume_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        HealingPlumeZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::EntangleGround => {
            let duration = extra[1];
            // Spawn the visible vine rings (same as single-player) so the entangle
            // zone isn't invisible on the opposing client. RNG here is cosmetic.
            crate::game::units::wizard::spells::entangle::vines::spawn_vine_toruses(
                &mut rand::rng(),
                commands,
                assets,
                materials,
                Vec3::new(pos.x, 0.0, pos.z),
                120.0,
                duration,
                OnMultiplayerGameScreen,
            );
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                        Visibility::Hidden,
                        EntangleGroundEffect::new(
                            duration,
                            Vec3::new(pos.x, 1.0, pos.z),
                            120.0,
                            crate::game::units::wizard::spells::entangle::components::EntangleTalentParams::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::FogCloudZone => {
            use crate::game::units::wizard::spells::fog_cloud::components::{
                BlindingMistZone, ChokingFogZone, ConcealingVeilZone, DisorientingVaporsZone,
                PhantomFogZone, RollingFogZone,
            };
            let radius = extra[0];
            let duration = extra[1];
            // Bug fix: ghost zone previously got `evasion_chance=0.0` and
            // `evasion_refresh_duration=0.0`, making the fog do nothing on
            // the remote peer regardless of caster. The collector now packs
            // these in `extra[2]` and `extra[3]` so the ghost matches the
            // caster's values.
            let evasion_chance = extra[2];
            let evasion_refresh_duration = extra[3];
            let material = materials.add(materials.get(&assets.fog_cloud_zone)?.clone());
            let mut ec = commands.spawn((
                Mesh3d(assets.unit_circle.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                FogCloudZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius,
                    evasion_chance,
                    evasion_refresh_duration,
                    1.0,
                    duration,
                ),
                OnMultiplayerGameScreen,
            ));
            // Insert FogCloud talent marker components based on the host's
            // packed flags. Gameplay-authoritative behavior (Choking DPS,
            // Rolling drift, Phantom spawns) is host-only, so these markers
            // on the ghost are mostly for visual consistency / system
            // existence-checks. Default field values are fine — the host
            // ticks the real ones.
            if flags & (1 << 0) != 0 {
                ec.insert(BlindingMistZone);
            }
            if flags & (1 << 1) != 0 {
                ec.insert(ConcealingVeilZone);
            }
            if flags & (1 << 2) != 0 {
                ec.insert(DisorientingVaporsZone);
            }
            if flags & (1 << 3) != 0 {
                ec.insert(PhantomFogZone { spawn_timer: 0.0 });
            }
            if flags & (1 << 4) != 0 {
                ec.insert(ChokingFogZone::new(0.0, 1.0));
            }
            if flags & (1 << 5) != 0 {
                ec.insert(RollingFogZone { speed: 0.0 });
            }
            Some(ec.id())
        }

        SpellEffectKind::GreaseZone => {
            let radius = extra[0];
            let duration = extra[1];
            let mut base_mat = materials.get(&assets.grease_zone)?.clone();
            base_mat.alpha_mode = bevy::render::alpha::AlphaMode::Mask(0.01);
            let material = materials.add(base_mat);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 2.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        GreaseZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            0.0,
                            1.0,
                            duration,
                            0.0,
                            0.0,
                            0.0,
                            1.0,
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::GreaseFire => {
            // Fire overlay is a second circle on top of the grease zone.
            // Scale is updated every frame from the snapshot (fire spread animation).
            let scale = extra[0].max(0.01);
            let material = materials.add(materials.get(&assets.grease_fire)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.1, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(scale)),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::PlagueWindCloud => {
            use crate::game::units::wizard::spells::plague_wind::components::PlagueWindTalentParams;
            let radius = extra[0];
            let duration = extra[1];
            let speed = extra[2];
            let direction_angle = extra[3];
            let direction = Vec3::new(direction_angle.sin(), 0.0, direction_angle.cos());
            // Unpack the talent boolean flags sent by the host (see the
            // matching collector arm in `spell_sync.rs::collect_spell_effect_snapshots`).
            // Numeric talent multipliers are kept at default — they are
            // already baked into the host's authoritative damage values
            // that flow back via the CRDT pipeline, so reproducing them on
            // the ghost would double-count.
            let talent_params = PlagueWindTalentParams {
                plague_carrier: flags & (1 << 0) != 0,
                toxic_weakness: flags & (1 << 1) != 0,
                choking_gas: flags & (1 << 2) != 0,
                pandemic: flags & (1 << 3) != 0,
                twin_plumes: flags & (1 << 4) != 0,
                necrotic_rot: flags & (1 << 5) != 0,
                ..PlagueWindTalentParams::default()
            };
            // No mesh — the cloud's visual is the shared green `plague_smoke`
            // particle system (`emit_plague_cloud_particles` runs on every
            // PlagueWindCloud, including this ghost), matching single-player. The
            // old flat green disc was a placeholder.
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
                        PlagueWindCloud::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                            speed,
                            direction,
                            talent_params,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::MeteorGroundFire => {
            let radius = extra[0];
            let duration = extra[1];
            // No mesh — single-player spawns the burning patch mesh-less and
            // renders it through the shared fire-particle system. The flat
            // orange disc was a ghost-only placeholder that sat on top of the
            // particles (the user's "old orange circle").
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                            .with_scale(Vec3::splat(radius)),
                        Visibility::default(),
                        MeteorGroundFire::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        // ── Objects (3D meshes, shared materials) ──
        SpellEffectKind::BlackHole => {
            use crate::game::units::wizard::spells::black_hole::components::BlackHoleTalentParams;
            let max_radius = extra[0];
            let empowerment = extra[1];
            let talent_params = BlackHoleTalentParams {
                event_horizon: flags & (1 << 0) != 0,
                crushing_pressure: flags & (1 << 1) != 0,
                void_siphon: flags & (1 << 2) != 0,
                singularity: flags & (1 << 3) != 0,
                dimensional_rift: flags & (1 << 4) != 0,
                ..BlackHoleTalentParams::default()
            };
            // Icosphere scaled by max_radius * growth_factor in update_black_hole_visuals.
            // The ghost still won't run the damage/pull/etc. systems (those
            // are gated `Without<GhostSpellEffect>` for host-authoritative)
            // — but visual systems and talent-driven extra visuals can now
            // see the correct flags.
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.black_hole_sphere.clone()),
                        MeshMaterial3d(assets.black_hole.clone()),
                        Transform::from_translation(pos).with_scale(Vec3::ZERO),
                        BlackHole::new(pos, max_radius, empowerment, talent_params),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::ArcaneCrystal => {
            use crate::game::units::wizard::spells::arcane_crystal::components::{
                ArcaneCrystal, AutoCrystalTimer, CrystalNetwork, CrystalRangeIndicator,
                PrismaticExplosion, ResonanceCascade,
            };
            let range = extra[0];
            let duration = extra[1];
            let empowerment = extra[2];
            let height = 35.0 * empowerment; // CRYSTAL_HEIGHT * empowerment
            let sphere_radius = height / 3.0;
            // Cross-plane sphere scaled to crystal shape
            let mut ec = commands.spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.arcane_crystal.clone()),
                Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z)).with_scale(
                    Vec3::new(
                        0.7 * sphere_radius,
                        1.5 * sphere_radius,
                        0.7 * sphere_radius,
                    ),
                ),
                ArcaneCrystal::new(
                    Vec3::new(pos.x, height / 2.0, pos.z),
                    range,
                    duration,
                    range * 0.15,
                    empowerment,
                ),
                OnMultiplayerGameScreen,
            ));
            // Re-insert any talent marker components present on the original
            // caster's crystal so the receiving peer's talent visuals / state
            // systems attach correctly.
            if flags & (1 << 0) != 0 {
                ec.insert(ResonanceCascade { absorptions: 0 });
            }
            if flags & (1 << 1) != 0 {
                ec.insert(PrismaticExplosion);
            }
            if flags & (1 << 2) != 0 {
                ec.insert(AutoCrystalTimer { timer: 0.0 });
            }
            if flags & (1 << 3) != 0 {
                ec.insert(CrystalNetwork);
            }
            let crystal_entity = ec.id();
            // Mirror the pink aura range sphere that the local-cast path
            // spawns in `arcane_crystal/setup.rs`. Without this, the
            // receiving peer sees the crystal but not its sphere of
            // influence — the bubble visual is missing on the ghost side.
            ec.commands().spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(assets.crystal_aura_sphere.clone()),
                Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z))
                    .with_scale(Vec3::splat(range)),
                CrystalRangeIndicator { crystal_entity },
                OnMultiplayerGameScreen,
            ));
            Some(crystal_entity)
        }

        SpellEffectKind::LightningRod => {
            use crate::game::units::wizard::spells::lightning_rod::components::{
                LightningRod, LightningRodTalentParams,
            };
            let duration = extra[0];
            let empowerment = extra[1];
            // Unpack the talent booleans sent by the host. The numeric
            // duration/strike-interval/arc-radius/damage multipliers are
            // already baked into the host's authoritative strike damage
            // (which flows back via CRDT), so they stay at default here.
            let talent_params = LightningRodTalentParams {
                chain_reaction: flags & (1 << 0) != 0,
                magnetic_field: flags & (1 << 1) != 0,
                overcharge: flags & (1 << 2) != 0,
                storm_spire: flags & (1 << 3) != 0,
                tesla_coil: flags & (1 << 4) != 0,
                lightning_nexus: flags & (1 << 5) != 0,
                ..LightningRodTalentParams::default()
            };
            // Lightning rod uses a cylinder mesh; create one at spawn.
            // This is a small allocation but rods are rare (1-2 at most).
            let tower_height = 60.0; // TOWER_HEIGHT
            let tower_radius = 8.0; // TOWER_RADIUS
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.lightning_rod.clone()),
                        Transform::from_translation(Vec3::new(pos.x, tower_height / 2.0, pos.z))
                            .with_scale(Vec3::new(
                                tower_radius * 2.0,
                                tower_height,
                                tower_radius * 2.0,
                            )),
                        LightningRod::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            duration,
                            empowerment,
                            talent_params,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
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
            // Inserting `WallRising` makes the shared `animate_rising_walls`
            // system play the rise-from-the-ground animation here on the
            // remote peer too, instead of the wall just popping into
            // existence at full height. Spawn underground (`-height / 2.0`)
            // so the very first frame of the animator pulls the wall up
            // instead of yanking it from full height down to underground.
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.wall_of_stone.clone()),
                        Transform::from_translation(Vec3::new(pos.x, -height / 2.0, pos.z))
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
                            permanent: false,
                        },
                        crate::game::units::wizard::spells::wall_of_stone::components::WallRising::new(
                            crate::game::units::wizard::spells::wall_of_stone::constants::WALL_RISE_DURATION,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::WallOfFire => {
            use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireTalentParams;
            let half_width = extra[0];
            let duration = extra[1];
            let wall_length = extra[2];
            let talent_params = WallOfFireTalentParams {
                searing_heat: flags & (1 << 0) != 0,
                scorched_earth: flags & (1 << 1) != 0,
                spreading_flames: flags & (1 << 2) != 0,
                firestorm: flags & (1 << 3) != 0,
                twin_walls: flags & (1 << 4) != 0,
                consuming_inferno: flags & (1 << 5) != 0,
                ..WallOfFireTalentParams::default()
            };
            let material = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            let wall_height = 10.0;
            let wall_entity = commands
                .spawn((
                    Mesh3d(assets.unit_cuboid.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(pos.x, wall_height / 2.0, pos.z))
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(wall_length, wall_height, 60.0)),
                    WallOfFireEffect::new(
                        Vec3::ZERO,
                        Vec3::ZERO,
                        half_width,
                        0.0,
                        crate::game::units::DamageType::Fire,
                        1.0,
                        duration,
                        talent_params,
                    ),
                    OnMultiplayerGameScreen,
                ))
                .id();

            // Ignition spark burst along the wall so the opposing client sees
            // fire kick up (matches the SP `spawn_wall_vfx` spark portion). The
            // mesh's length runs along local X, so the wall axis is `rot * X`.
            // (SP's looping crackle SFX is host-local and not replicated here.)
            let wall_axis = rotation * Vec3::X;
            let start = Vec3::new(pos.x, 3.0, pos.z) - wall_axis * (wall_length * 0.5);
            let spark_points = 4;
            let t_secs = start.x * 0.01;
            for j in 0..spark_points {
                let frac = (j as f32 + 0.5) / spark_points as f32;
                let spark_pos = start + wall_axis * (wall_length * frac);
                crate::game::units::wizard::spells::vfx::systems::spawn_fire_sparks(
                    commands,
                    assets,
                    spark_pos,
                    crate::game::units::wizard::spells::vfx::constants::SPARK_COUNT / 2,
                    t_secs + j as f32,
                );
            }
            Some(wall_entity)
        }

        // ── Explosions (sphere meshes, scale-driven animation) ──
        SpellEffectKind::FireballExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            let explosion_pos = Vec3::new(pos.x, pos.y.max(5.0), pos.z);
            let mat_handle =
                clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(mat_handle),
                        Transform::from_translation(explosion_pos).with_scale(Vec3::splat(0.1)),
                        FireballExplosion::new(
                            pos,
                            max_radius,
                            0.0,
                            crate::game::units::DamageType::Fire,
                            empowerment,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::MeteorExplosion => {
            let max_radius = extra[0];
            let mat_handle =
                clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(mat_handle),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_scale(Vec3::splat(0.1)),
                        MeteorExplosion::new(pos, max_radius, 0.0),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::IceExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            let mat_handle = clone_sphere_material(sphere_materials, &assets.ice_explosion_sphere);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(mat_handle),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_scale(Vec3::splat(0.1)),
                        IceExplosion::new(pos, max_radius, 0.0, empowerment),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::DispelImpact => {
            use crate::game::units::wizard::spells::dispel::components::DispelImpact;
            let duration = extra[0];
            let expand_speed = extra[1];
            // Same mesh + material the host's DispelImpact uses, so the
            // remote peer sees the expanding nullification sphere. SP's
            // `update_dispel_impacts` is gated `is_spell_effects_active`
            // (both peers), so it ticks the growth + despawn on the ghost.
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(assets.guardian_aura_sphere.clone()),
                        Transform::from_translation(pos).with_scale(Vec3::ZERO),
                        DispelImpact {
                            time_alive: 0.0,
                            duration,
                            expand_speed,
                        },
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::SquallStorm => {
            use crate::game::units::wizard::spells::squall::components::{
                SquallStorm, SquallTalentParams,
            };
            let radius = extra[0];
            let talent_params = SquallTalentParams {
                permafrost: flags & (1 << 0) != 0,
                hailstones: flags & (1 << 1) != 0,
                sleet_storm: flags & (1 << 2) != 0,
                absolute_zero: flags & (1 << 3) != 0,
                blizzard: flags & (1 << 4) != 0,
                ice_age: flags & (1 << 5) != 0,
                ..SquallTalentParams::default()
            };
            Some(
                commands
                    .spawn((
                        Transform::from_translation(pos),
                        Visibility::Hidden,
                        SquallStorm::new(pos, radius, 1.0, talent_params),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::ScorchedEarthFire => {
            use crate::game::units::wizard::spells::fireball::components::{
                FireballExplosion, ScorchedEarthFire,
            };
            let max_radius = extra[0];
            let empowerment = extra[1];
            let duration = extra[2];
            let mat_handle =
                clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
            let mut explosion = FireballExplosion::new(
                pos,
                max_radius,
                0.0,
                crate::game::units::DamageType::Fire,
                empowerment,
            );
            explosion.duration = duration;
            explosion.skip_growth = true;
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(mat_handle),
                        Transform::from_translation(pos)
                            .with_scale(Vec3::splat(max_radius.max(0.01))),
                        explosion,
                        ScorchedEarthFire,
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::NapalmTrail => {
            use crate::game::units::wizard::spells::fireball::components::{
                FireballExplosion, ScorchedEarthFire,
            };
            let max_radius = extra[0];
            let empowerment = extra[1];
            let duration = extra[2];
            let mat_handle =
                clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
            let mut trail = FireballExplosion::new(
                pos,
                max_radius,
                0.0,
                crate::game::units::DamageType::Fire,
                empowerment,
            );
            trail.duration = duration;
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.explosion_sphere.clone()),
                        MeshMaterial3d(mat_handle),
                        Transform::from_translation(pos)
                            .with_scale(Vec3::splat(max_radius.max(0.01))),
                        trail,
                        // Same `ScorchedEarthFire` marker the SP path uses
                        // — `spawn_scorched_earth_fire_smoke` is gated on
                        // `any_exist::<ScorchedEarthFire>()`, so without
                        // this the smoke VFX is missing for host-cast
                        // napalm trails on the guest's screen.
                        ScorchedEarthFire,
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::BoulderProjectileEffect => {
            // Spawn a ghost mid-air boulder at the host's transform; the
            // host's per-frame `NetworkedSpellEffect` snapshot keeps it on
            // the same arc. `extra[0]` carries the sprite index so the
            // guest picks the matching boulder material.
            let sprite_index = extra[0] as usize;
            let idx = sprite_index.min(boulder_assets.materials.len().saturating_sub(1));
            Some(
                commands
                    .spawn((
                        Mesh3d(boulder_assets.mesh.clone()),
                        MeshMaterial3d(boulder_assets.materials[idx].clone()),
                        Transform::from_translation(pos),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::BoulderObstacle => {
            // Spawn a ghost grounded boulder at the host's land position.
            // `extra[0]` = sprite_index, `extra[1]` = radius, `extra[2]` =
            // height. Billboard so the sprite faces the camera identically
            // to the SP path. No `Boulder` / `ObstacleHealth` component on
            // the ghost — those are host-authoritative and would re-trigger
            // gameplay systems if present (they're gated `is_gameplay_running`
            // = host-only, so they wouldn't actually fire on the guest, but
            // leaving them off is cleaner).
            let sprite_index = extra[0] as usize;
            let idx = sprite_index.min(boulder_assets.materials.len().saturating_sub(1));
            Some(
                commands
                    .spawn((
                        Mesh3d(boulder_assets.mesh.clone()),
                        MeshMaterial3d(boulder_assets.materials[idx].clone()),
                        Transform::from_translation(pos),
                        crate::game::components::Billboard,
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
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
