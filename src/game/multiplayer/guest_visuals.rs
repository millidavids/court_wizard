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
pub(super) fn spawn_spell_effect(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
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
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.fog_cloud_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        FogCloudZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
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
            let radius = extra[0];
            let duration = extra[1];
            let speed = extra[2];
            let direction_angle = extra[3];
            let direction = Vec3::new(direction_angle.sin(), 0.0, direction_angle.cos());
            let material = materials.add(materials.get(&assets.plague_wind_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        PlagueWindCloud::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                            speed,
                            direction,
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::MeteorGroundFire => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.meteor_ground_fire)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
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
            let max_radius = extra[0];
            let empowerment = extra[1];
            // Icosphere scaled by max_radius * growth_factor in update_black_hole_visuals
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.black_hole_sphere.clone()),
                        MeshMaterial3d(assets.black_hole.clone()),
                        Transform::from_translation(pos).with_scale(Vec3::ZERO), // Grows from 0 via update_black_hole_visuals
                        BlackHole::new(pos, max_radius, empowerment, Default::default()),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::ArcaneCrystal => {
            let range = extra[0];
            let duration = extra[1];
            let empowerment = extra[2];
            let height = 35.0 * empowerment; // CRYSTAL_HEIGHT * empowerment
            let sphere_radius = height / 3.0;
            // Cross-plane sphere scaled to crystal shape
            Some(commands.spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.arcane_crystal.clone()),
                Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z))
                    .with_scale(Vec3::new(0.7 * sphere_radius, 1.5 * sphere_radius, 0.7 * sphere_radius)),
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
            let tower_radius = 8.0; // TOWER_RADIUS
            Some(commands.spawn((
                Mesh3d(assets.unit_cuboid.clone()),
                MeshMaterial3d(assets.lightning_rod.clone()),
                Transform::from_translation(Vec3::new(pos.x, tower_height / 2.0, pos.z))
                    .with_scale(Vec3::new(tower_radius * 2.0, tower_height, tower_radius * 2.0)),
                crate::game::units::wizard::spells::lightning_rod::components::LightningRod::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    duration, empowerment,
                    crate::game::units::wizard::spells::lightning_rod::components::LightningRodTalentParams::default(),
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
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.wall_of_stone.clone()),
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
                            permanent: false,
                        },
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::WallOfFire => {
            let half_width = extra[0];
            let duration = extra[1];
            let wall_length = extra[2];
            let material = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            let wall_height = 10.0;
            Some(
                commands
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
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
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
