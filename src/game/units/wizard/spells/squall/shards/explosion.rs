//! Ice explosion updates, frozen ground ticking, and storm-ring reticle.

use bevy::prelude::*;
use rand::Rng;

use super::super::casting::apply_frost_accumulation;
use super::super::components::{FrozenGround, IceExplosion, SquallStorm, SquallStormRing};
use super::super::constants::*;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::components::FrostAccumulation;
use crate::game::units::components::{
    Health, Hitbox, SlowMovementModifier, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::{
    indicator_pulse_scale, local_player_team, sphere_intersects_cylinder, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, explosion_fade_opacity,
};
use crate::networking::session::MultiplayerSession;

/// Updates explosion visuals, applies damage, and tracks Permafrost hits.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn update_ice_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut explosions: Query<
        (
            Entity,
            &mut IceExplosion,
            &mut Transform,
            Option<&MeshMaterial3d<FireExplosionSphereMaterial>>,
        ),
        // Ghost ice explosions mirror the host's; applying frost here double-stacks it.
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut FrostAccumulation>,
            &Hitbox,
            &Team,
        ),
        (
            Without<IceExplosion>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    storms: Query<&SquallStorm, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    // Get the active storm's talent params for permafrost tracking
    let storm_has_permafrost = storms
        .iter()
        .next()
        .map(|s| s.talent_params.permafrost)
        .unwrap_or(false);

    let time_secs = time.elapsed_secs();

    for (explosion_entity, mut explosion, mut transform, material_handle) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Fade out over the last portion of lifetime
        if let Some(handle) = material_handle
            && let Some(mat) = sphere_materials.get_mut(handle)
        {
            mat.opacity = explosion_fade_opacity(explosion.time_alive / EXPLOSION_LIFETIME);
        }

        // Continuous white smoke from explosion surface (throttled to ~20Hz)
        let prev_tick = ((explosion.time_alive - time.delta_secs()) / 0.05) as u32;
        let curr_tick = (explosion.time_alive / 0.05) as u32;
        if current_radius > 5.0
            && curr_tick > prev_tick
            && explosion.time_alive < EXPLOSION_LIFETIME
        {
            let dir = Vec3::new(
                game_rng.0.random_range(-1.0..1.0_f32),
                game_rng.0.random_range(0.2..1.0_f32),
                game_rng.0.random_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);
            let surface_pos = explosion.origin + dir * current_radius;
            vfx::systems::spawn_explosion_smoke_with_material(
                &mut commands,
                &visual_assets,
                surface_pos,
                time_secs,
                visual_assets.ice_smoke.clone(),
                5,
            );
        }

        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;
            let mut units_hit: u32 = 0;

            terrain_damage.write(TerrainDamageMessage {
                position: explosion.origin,
                radius: explosion.max_radius,
                damage: explosion.damage,
                damage_type: explosion.damage_type,
            });

            // Permafrost talent doubles frost accumulation per hit
            let frost_per_hit = if storm_has_permafrost {
                PERMAFROST_FROST_PER_HIT
            } else {
                FROST_PER_HIT
            };

            for (
                unit_entity,
                unit_transform,
                mut health,
                mut temp_hp,
                has_spell_shield,
                frost_accum,
                hitbox,
                team,
            ) in units.iter_mut()
            {
                let hit = sphere_intersects_cylinder(
                    explosion.origin,
                    explosion
                        .current_radius(EXPLOSION_GROWTH_TIME)
                        .max(explosion.max_radius),
                    Vec3::new(
                        unit_transform.translation.x,
                        0.0,
                        unit_transform.translation.z,
                    ),
                    hitbox.radius,
                    hitbox.height,
                );

                if hit {
                    apply_spell_damage_with_team(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        explosion.damage_type,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    units_hit += 1;

                    // Progressive frost accumulation (drives slow + tint + eventual freeze)
                    apply_frost_accumulation(
                        &mut commands,
                        unit_entity,
                        frost_accum,
                        frost_per_hit,
                    );
                }
            }

            // Track talent progress
            if units_hit > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Squall, units_hit);
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).try_despawn();
        }
    }
}

/// Updates frozen ground patches: applies slow to enemies walking over them.
pub(crate) fn update_frozen_ground(
    time: Res<Time>,
    mut commands: Commands,
    mut patches: Query<(Entity, &mut FrozenGround)>,
    mut units: Query<
        (Entity, &Transform, &Team, Option<&mut SlowMovementModifier>),
        (
            With<Health>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    use super::super::casting::apply_or_insert_slow;

    for (patch_entity, mut patch) in patches.iter_mut() {
        patch.time_remaining -= time.delta_secs();

        if patch.time_remaining <= 0.0 {
            commands.entity(patch_entity).try_despawn();
            continue;
        }

        // Apply slow to enemies inside the patch
        for (unit_entity, unit_transform, team, slow_mod) in units.iter_mut() {
            if *team == Team::Defenders {
                continue;
            }

            let distance = xz_distance(unit_transform.translation, patch.position);

            if distance <= patch.radius {
                apply_or_insert_slow(
                    &mut commands,
                    unit_entity,
                    slow_mod,
                    ICE_AGE_SLOW_MODIFIER,
                    ICE_AGE_SLOW_DURATION,
                );
            }
        }
    }
}

/// Updates the storm ring reticle: syncs position with the storm, pulse animation,
/// and despawns the ring when the storm is gone (concentration ended or AZ released).
pub(crate) fn update_storm_ring(
    time: Res<Time>,
    storms: Query<&SquallStorm, Without<crate::game::multiplayer::components::GhostSpellEffect>>,
    mut rings: Query<(Entity, &mut SquallStormRing, &mut Transform)>,
    mut commands: Commands,
) {
    let storm = storms.iter().next();

    for (entity, mut ring, mut transform) in rings.iter_mut() {
        let Some(storm) = storm else {
            // No storm — despawn orphaned ring
            commands.entity(entity).try_despawn();
            continue;
        };

        ring.time_alive += time.delta_secs();
        let pulse = indicator_pulse_scale(ring.time_alive);
        transform.translation.x = storm.position.x;
        transform.translation.z = storm.position.z;
        transform.scale = Vec3::splat(storm.radius * pulse);
    }
}
