use super::super::components::{
    Bloodlust, ContagiousRage, FinalStand, FinalStandExplosionVfx, Frenzy, FrenzyActive,
    UndyingFury, UndyingFuryActive,
};
use super::super::constants;
use super::super::messages::ContagiousRageKillMessage;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    BerserkerRageModifier, Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material, explosion_fade_opacity,
};
use bevy::prelude::*;

/// Undying Fury: prevent death for enraged units.
/// Runs after combat but before corpse conversion.
/// If a unit with UndyingFury has <= 0 HP, restores them to 1 HP
/// and starts the active protection timer.
pub fn undying_fury_trigger(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health), (With<UndyingFury>, Without<Corpse>)>,
) {
    for (entity, mut health) in &mut query {
        if health.is_dead() {
            health.current = 1.0;
            commands.entity(entity).remove::<UndyingFury>();
            commands.entity(entity).insert(UndyingFuryActive {
                time_remaining: constants::UNDYING_FURY_DURATION,
            });
        }
    }
}

/// Tick Undying Fury Active timer and enforce minimum 1 HP while active.
/// When timer expires, the unit can die normally.
pub fn tick_undying_fury_active(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut UndyingFuryActive, &mut Health), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for (entity, mut active, mut health) in &mut query {
        active.time_remaining -= delta;
        if active.time_remaining <= 0.0 {
            commands.entity(entity).remove::<UndyingFuryActive>();
        } else {
            // Enforce minimum 1 HP while active
            if health.current < 1.0 {
                health.current = 1.0;
            }
        }
    }
}

/// Frenzy: toggle FrenzyActive based on HP threshold.
/// Runs each frame for units with the Frenzy component.
pub fn frenzy_check_system(
    mut commands: Commands,
    query: Query<
        (Entity, &Health, &Frenzy, Option<&FrenzyActive>),
        (With<BerserkerRageModifier>, Without<Corpse>),
    >,
) {
    for (entity, health, frenzy, active) in &query {
        let below_threshold =
            health.max > 0.0 && health.current / health.max <= frenzy.hp_threshold;
        if below_threshold && active.is_none() {
            commands.entity(entity).insert(FrenzyActive);
        } else if !below_threshold && active.is_some() {
            commands.entity(entity).remove::<FrenzyActive>();
        }
    }
}

/// Contagious Rage: when an enraged unit kills an enemy, spread rage to the nearest calm ally.
pub fn contagious_rage_spread(
    mut commands: Commands,
    mut kill_events: MessageReader<ContagiousRageKillMessage>,
    killer_query: Query<(&Transform, &Team, &ContagiousRage), Without<Corpse>>,
    candidates: Query<
        (Entity, &Transform, &Team),
        (
            Without<BerserkerRageModifier>,
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    for event in kill_events.read() {
        let Ok((killer_pos, killer_team, rage_params)) = killer_query.get(event.killer) else {
            continue;
        };

        // Find nearest same-team ally without berserker rage
        let nearest = candidates
            .iter()
            .filter(|(_, _, team)| **team == *killer_team)
            .min_by(|(_, a_pos, _), (_, b_pos, _)| {
                let da = a_pos.translation.distance_squared(killer_pos.translation);
                let db = b_pos.translation.distance_squared(killer_pos.translation);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((target_entity, _, _)) = nearest {
            // Apply rage with reduced effectiveness
            let effectiveness = 1.0 - constants::CONTAGIOUS_RAGE_EFFECTIVENESS_LOSS;
            commands
                .entity(target_entity)
                .insert(BerserkerRageModifier::new(
                    rage_params.damage_bonus * effectiveness,
                    rage_params.vulnerability * effectiveness,
                    rage_params.duration * effectiveness,
                ));
            // Spread the ContagiousRage component so kills by the new unit also spread
            commands.entity(target_entity).insert(ContagiousRage {
                damage_bonus: rage_params.damage_bonus * effectiveness,
                vulnerability: rage_params.vulnerability * effectiveness,
                duration: rage_params.duration * effectiveness,
            });
        }
    }
}

/// Final Stand: when an enraged unit dies, explode for AoE damage.
/// Queries corpses with FinalStand and applies damage to nearby enemies.
/// Spawns a fireball explosion visual at the death location.
pub fn final_stand_explosion(
    mut commands: Commands,
    dead_query: Query<(Entity, &FinalStand, &Transform, &Team, &Health), With<Corpse>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    time: Res<Time>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    for (corpse_entity, final_stand, transform, team, health) in &dead_query {
        // Damage = fraction of the dead unit's max HP
        let explosion_damage = health.max * final_stand.damage_fraction;
        let position = transform.translation;

        for (target_entity, target_pos, target_team, mut target_health, temp_hp) in
            targets.iter_mut()
        {
            if *target_team == *team {
                continue;
            }
            if crate::game::units::wizard::spells::utils::xz_distance(
                target_pos.translation,
                position,
            ) <= final_stand.radius
            {
                apply_spell_damage(
                    &mut commands,
                    target_entity,
                    &mut target_health,
                    temp_hp.map(|t| t.into_inner()),
                    explosion_damage,
                    DamageType::Force,
                    false,
                );
            }
        }

        // Spawn fireball explosion visual
        let mat_handle = clone_sphere_material(
            &mut sphere_materials,
            &visual_assets.fireball_explosion_sphere,
        );

        commands.spawn((
            FinalStandExplosionVfx {
                time_alive: 0.0,
                max_radius: final_stand.radius,
                lifetime: constants::FINAL_STAND_VFX_LIFETIME,
            },
            Mesh3d(visual_assets.explosion_sphere.clone()),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
            OnGameplayScreen,
        ));

        // MP: ship the explosion to the guest so the same sphere appears
        // there. The guest's `update_final_stand_vfx` (gated
        // `is_spell_effects_active`) animates it once the component exists.
        vfx::systems::emit_cast_event(
            &mut pending_cast_events,
            crate::networking::snapshot::CastEventKind::FinalStandExplosion,
            0,
            position,
            [
                final_stand.radius,
                constants::FINAL_STAND_VFX_LIFETIME,
                0.0,
                0.0,
            ],
        );

        // Fire sparks + smoke burst
        let time_secs = time.elapsed_secs();
        vfx::systems::spawn_fire_sparks(
            &mut commands,
            &visual_assets,
            position,
            constants::FINAL_STAND_SPARK_COUNT,
            time_secs,
        );
        vfx::systems::spawn_explosion_smoke(&mut commands, &visual_assets, position, time_secs);
        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            position,
            vfx::constants::EXPLOSION_SHIMMER_COUNT,
            time_secs,
        );
        vfx::systems::spawn_explosion_dark_smoke(
            &mut commands,
            &visual_assets,
            position,
            time_secs,
        );

        // One-shot: remove marker so explosion doesn't fire again
        commands.entity(corpse_entity).remove::<FinalStand>();
    }
}

/// Updates Final Stand explosion visuals: expand, fade, then despawn.
pub fn update_final_stand_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut query: Query<(
        Entity,
        &mut FinalStandExplosionVfx,
        &mut Transform,
        &MeshMaterial3d<FireExplosionSphereMaterial>,
    )>,
) {
    for (entity, mut vfx, mut transform, material_handle) in &mut query {
        vfx.time_alive += time.delta_secs();
        if vfx.time_alive >= vfx.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }
        let progress = (vfx.time_alive / vfx.lifetime).min(1.0);
        let current_radius = vfx.max_radius * progress;
        transform.scale = Vec3::splat(current_radius.max(0.1));

        let opacity = explosion_fade_opacity(progress);
        if let Some(mut mat) = sphere_materials.get_mut(material_handle) {
            mat.opacity = opacity;
        }
    }
}

/// Clean up berserker rage talent components when the base modifier is removed.
/// This handles the case where the buff expires naturally.
pub fn cleanup_berserker_rage_talents(
    mut commands: Commands,
    query: Query<Entity, super::casting::BerserkerCleanupFilter>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<Bloodlust>()
            .remove::<Frenzy>()
            .remove::<FrenzyActive>()
            .remove::<UndyingFury>()
            .remove::<UndyingFuryActive>()
            .remove::<ContagiousRage>()
            .remove::<FinalStand>();
    }
}
