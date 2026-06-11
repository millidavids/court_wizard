use super::super::components::*;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::components::{
    Health, MarkedForDeathModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, explosion_fade_opacity,
};
use bevy::prelude::*;

/// Updates explosion visuals and timing. Spawns VFX burst on first frame and
/// continuous sparks + smoke throughout the explosion's lifetime.
pub fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Visual-only (scale growth + sparks/smoke). Runs on ghost explosions too so
    // the opposing client sees them grow + fade. Damage lives in
    // `apply_explosion_damage` (gated host-only) and despawn in
    // `cleanup_finished_explosions` (gated host-only; ghosts despawn via snapshot
    // reconciliation), so animating ghosts here is safe.
    mut explosions: Query<(
        &mut FireballExplosion,
        &mut Transform,
        // Ghost explosions on the guest have no host-only `apply_explosion_damage`
        // to reset `time_since_last_tick`, so we reset it ourselves below —
        // otherwise the continuous-surface-VFX block fires every frame on ghosts.
        Has<crate::game::multiplayer::components::GhostSpellEffect>,
    )>,
) {
    use rand::Rng;
    let time_secs = time.elapsed_secs();
    let rng = &mut game_rng.0;

    for (mut explosion, mut transform, is_ghost) in &mut explosions {
        explosion.time_alive += time.delta_secs();
        explosion.time_since_last_tick += time.delta_secs();

        let current_radius = explosion.current_radius();
        transform.scale = Vec3::splat(current_radius);

        // Spawn sparks + smoke + shimmer on first frame (skip for persistent ground effects)
        if !explosion.vfx_spawned && !explosion.skip_growth {
            explosion.vfx_spawned = true;
            let pos = explosion.origin;
            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::SPARK_COUNT,
                time_secs,
            );
            vfx::systems::spawn_explosion_smoke(&mut commands, &visual_assets, pos, time_secs);
            vfx::systems::spawn_heat_shimmer(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::EXPLOSION_SHIMMER_COUNT,
                time_secs,
            );
            vfx::systems::spawn_explosion_dark_smoke(&mut commands, &visual_assets, pos, time_secs);
        }

        // Continuous sparks and smoke from random positions on the explosion surface
        if explosion.vfx_spawned
            && !explosion.skip_growth
            && current_radius > 5.0
            && explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL
        {
            let dir = Vec3::new(
                rng.random_range(-1.0..1.0_f32),
                rng.random_range(0.2..1.0_f32),
                rng.random_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);
            let surface_pos = explosion.origin + dir * current_radius;

            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                surface_pos,
                4,
                time_secs,
            );
            vfx::systems::spawn_explosion_smoke_with_material(
                &mut commands,
                &visual_assets,
                surface_pos,
                time_secs,
                visual_assets.fire_smoke.clone(),
                5,
            );
            // Ghosts have no host-only damage system to reset this timer; do it
            // here so the block fires once per interval (matching SP cadence)
            // instead of every frame on the guest.
            if is_ghost {
                explosion.time_since_last_tick = 0.0;
            }
        }
    }
}

/// Applies damage to units hit by the explosion on a tick interval.
pub fn apply_explosion_damage(
    mut commands: Commands,
    // Host-only — ghost fireball/ScorchedEarth/NapalmTrail explosions
    // on the guest must NOT independently apply DPS. CRDT max-merge would
    // absorb the HP delta but talent progress increments and
    // TerrainDamageMessage events would double-fire.
    mut explosions: Query<
        &mut FireballExplosion,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        &Team,
        Has<SpellShield>,
        Option<&MarkedForDeathModifier>,
    )>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    // The casting peer's team is constant for the whole system run (these
    // explosions only damage real units on the host, whose spells are its own).
    let caster_team =
        crate::game::units::wizard::spells::utils::local_player_team(session.as_deref());
    for mut explosion in &mut explosions {
        if explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL {
            explosion.time_since_last_tick = 0.0;

            // Skip damage iteration for VFX-only explosions (e.g. undead detonation)
            if explosion.damage_per_tick <= 0.0 {
                continue;
            }

            let current_radius = explosion.current_radius();
            let mut hit_count = 0u32;

            terrain_damage.write(TerrainDamageMessage {
                position: explosion.origin,
                radius: current_radius,
                damage: explosion.damage_per_tick,
                damage_type: explosion.damage_type,
            });

            for (
                entity,
                transform,
                mut health,
                mut temp_hp,
                team,
                has_spell_shield,
                existing_mark,
            ) in &mut targets
            {
                let distance = crate::game::units::wizard::spells::utils::xz_distance(
                    explosion.origin,
                    transform.translation,
                );

                if distance <= current_radius {
                    // Team-aware: a friendly King takes its own side's fireball;
                    // the enemy King's shield still blocks it.
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage_per_tick,
                        explosion.damage_type,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    hit_count += 1;

                    // Chain Ignition: apply damage amplification debuff
                    if explosion.chain_ignition && existing_mark.is_none() {
                        commands
                            .entity(entity)
                            .insert(MarkedForDeathModifier::new(0.5, 3.0));
                    }
                }
            }

            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(explosion.source_spell, hit_count);
            }
        }
    }
}

/// Cleans up explosions that have finished animating.
pub fn cleanup_finished_explosions(
    mut commands: Commands,
    // Ghost explosions on the guest are despawned by snapshot reconciliation, not
    // by their own (now-ticking) timer — exclude them so they don't self-despawn
    // early and leave a stale entry in `SpellEffectEntityMap`.
    explosions: Query<
        (Entity, &FireballExplosion),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
) {
    for (entity, explosion) in &explosions {
        if explosion.time_alive >= explosion.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns sub-explosion spheres when the main explosion's radius reaches their position.
///
/// Each pending bubble has a pre-computed trigger distance. When the main explosion
/// grows past that distance, the bubble spawns — giving an amorphous, erupting look.
pub fn spawn_explosion_bubbles(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut spawners: Query<(&FireballExplosion, &mut ExplosionBubbleSpawner, &Transform)>,
) {
    for (explosion, mut spawner, transform) in &mut spawners {
        let current_radius = explosion.current_radius();

        for i in (0..spawner.pending.len()).rev() {
            if current_radius < spawner.pending[i].distance {
                continue;
            }

            let bubble = spawner.pending.swap_remove(i);
            let pos = transform.translation + bubble.direction * bubble.distance;

            // Per-entity material clone for independent fade
            let mat = sphere_materials
                .get(&visual_assets.fireball_explosion_sphere)
                .expect("sphere material template")
                .clone();
            let mat_handle = sphere_materials.add(mat);

            // Duration = remaining time so it ends with the main explosion
            let remaining = (explosion.duration - explosion.time_alive).max(0.1);

            let mut sub = FireballExplosion::new(
                pos,
                bubble.radius,
                0.0, // visual only, no damage
                constants::DAMAGE_TYPE,
                explosion.empowerment,
            );
            sub.duration = remaining;
            sub.vfx_spawned = true; // skip sparks/smoke

            commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
                sub,
                OnGameplayScreen,
            ));
        }
    }
}

/// Fades out explosion spheres that use FireExplosionSphereMaterial over their last portion.
pub fn fade_explosion_spheres(
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    explosions: Query<(
        &FireballExplosion,
        &MeshMaterial3d<FireExplosionSphereMaterial>,
    )>,
) {
    for (explosion, material_handle) in &explosions {
        if explosion.duration <= 0.0 {
            continue;
        }
        let opacity = explosion_fade_opacity(explosion.time_alive / explosion.duration);
        if let Some(mat) = sphere_materials.get_mut(material_handle) {
            mat.opacity = opacity;
        }
    }
}
