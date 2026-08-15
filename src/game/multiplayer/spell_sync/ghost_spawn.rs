use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::multiplayer::components::{
    GhostBeam, GhostMagicMissile, GhostSpellArc, GhostSpellProjectile, OnMultiplayerGameScreen,
    SpellEffectEntityMap,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::snapshot::{AuraBubbleVariant, SpellEffectKind};

use super::snapshot_send::LatestSpellSnapshot;

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
    mut ghost_crystals: Query<
        &mut crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal,
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
            } else if effect.kind == SpellEffectKind::ArcaneCrystal as u8 {
                // A crystal is always placed before it absorbs anything, so its
                // infusion is never known at ghost-spawn time. Track it per frame
                // or the guest's crystal stays visually uninfused for its whole
                // life. Visual only — gameplay systems skip ghost crystals.
                if let Ok(mut crystal) = ghost_crystals.get_mut(local_entity) {
                    crystal.infusion =
                        crate::game::units::wizard::spells::arcane_crystal::infusions::
                            CrystalInfusion::from_sync_id(effect.extra[3]);
                }
            }
            continue;
        }

        if let Some(entity) = super::super::guest_systems::spawn_spell_effect(
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

        // Finger of Death (kind=4) is the growing CONE: the cross-plane triangle
        // mesh, tip anchored at the beam ORIGIN (the caster's staff) and widening
        // out along the beam — matching the SP `spawn_beam` visual. Other beam-type
        // arcs are flat quads centred on the midpoint.
        let is_beam = arc.kind == 4;
        let (mesh, translation, scale) = if is_beam {
            (
                assets.cross_plane_triangle.clone(),
                origin,
                Vec3::new(width, length, width),
            )
        } else {
            (
                assets.unit_rect.clone(),
                midpoint,
                Vec3::new(width, length, 1.0),
            )
        };

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(translation)
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
                Mesh3d(assets.cross_plane_triangle.clone()),
                MeshMaterial3d(assets.finger_of_death_glow.clone()),
                Transform::from_translation(origin)
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
        //
        // Crystal-emitted beams get the core ONLY, matching SP `spawn_beam_core`
        // (`disintegrate/beam/spawn.rs`). They used to get the wizard treatment too,
        // which parked a `FLARE_RADIUS`-scaled opaque sphere on the crystal's exact
        // position — swallowing it whole on the opposing player's screen, and
        // permanently so, since Disintegrate keeps a beam alive for the crystal's
        // entire life.
        if beam.flags & crate::networking::snapshot::BEAM_FLAG_FROM_CRYSTAL != 0 {
            continue;
        }
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

/// Maps a wire-protocol `AuraBubbleVariant` to its `SpellVisualAssets` handle.
pub(super) fn aura_material_handle(
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
