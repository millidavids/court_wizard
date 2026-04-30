//! Raise the Dead persistent effects: plague bearer, corpse pull, detonation.

use super::casting::{compute_talent_params, find_nearest_corpse, raise_corpse_as_undead};
use std::collections::HashSet;

use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, PermanentCorpse, PoisonedModifier, Team, TemporaryHitPoints,
};
use crate::game::units::constants::{
    POISON_DURATION, POISON_EFFECTIVENESS_CAP, POISON_EFFECTIVENESS_PER_STACK,
};
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub fn tick_plague_bearer_aura(
    time: Res<Time>,
    mut commands: Commands,
    mut aura_query: Query<
        (&Transform, &mut PlagueBearerAura),
        (With<RaisedUndead>, Without<Corpse>),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Option<&mut PoisonedModifier>,
        ),
        Without<Corpse>,
    >,
    assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    for (aura_transform, mut aura) in &mut aura_query {
        // Spawn plague smoke VFX
        aura.smoke_spawn_timer += delta;
        if aura.smoke_spawn_timer >= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL {
            aura.smoke_spawn_timer -= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL;
            vfx::systems::spawn_plague_smoke_puffs(
                &mut commands,
                &assets,
                aura_transform.translation,
                aura.radius,
                vfx::constants::PLAGUE_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }

        aura.tick_accumulator += delta;
        if aura.tick_accumulator < aura.tick_interval {
            continue;
        }

        let damage = aura.dps * aura.tick_accumulator;
        aura.tick_accumulator = 0.0;

        for (entity, target_transform, team, mut health, temp_hp, poison) in &mut targets {
            // Only damage living enemies (attackers)
            if *team != Team::Attackers {
                continue;
            }

            let dist = aura_transform
                .translation
                .distance(target_transform.translation);
            if dist <= aura.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    damage,
                );

                // Apply poison tint (turns units green via the persistent effect visual system)
                if let Some(mut existing_poison) = poison {
                    existing_poison.stack(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                        POISON_EFFECTIVENESS_CAP,
                    );
                } else {
                    commands.entity(entity).insert(PoisonedModifier::new(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                    ));
                }
                // Don't insert Corpse here — let convert_dead_to_corpses handle
                // the full death pipeline (material swap, lay flat, etc.)
            }
        }
    }
}

/// Tier 2: Corpse Magnet — during channeling, pull corpses toward cursor position.
pub fn pull_corpses_to_cursor(
    time: Res<Time>,
    wizard_query: Query<&CorpseMagnetActive>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut corpse_query: Query<&mut Transform, (With<Corpse>, Without<PermanentCorpse>)>,
) {
    let Ok(magnet) = wizard_query.single() else {
        return;
    };

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some(target) = cursor_pos else {
        return;
    };

    let delta = time.delta_secs();

    for mut transform in &mut corpse_query {
        let dist = target.distance(transform.translation);
        if dist > magnet.pull_radius || dist < 1.0 {
            continue;
        }

        let direction = (target - transform.translation).normalize_or_zero();
        let move_amount = (magnet.pull_speed * delta).min(dist);
        transform.translation += direction * move_amount;
    }
}

/// Tier 3: Undead Detonation — when a raised undead with UndeadDetonation dies, explode.
#[allow(clippy::too_many_arguments)]
pub fn handle_undead_detonation(
    time: Res<Time>,
    mut commands: Commands,
    dead_query: Query<(Entity, &UndeadDetonation, &Transform), (With<RaisedUndead>, Added<Corpse>)>,
    mut targets: Query<
        (
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
    assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let _t = time.elapsed_secs();

    for (dead_entity, detonation, transform) in &dead_query {
        let origin = transform.translation;

        // Despawn the exploding undead — no corpse left behind
        commands.entity(dead_entity).try_despawn();

        // Spawn fireball explosion VFX
        let mut explosion = FireballExplosion::new(
            origin,
            detonation.radius,
            0.0, // No ongoing damage — we apply it manually below
            DamageType::Fire,
            1.0,
        );
        explosion.skip_growth = false;

        let mat_handle =
            clone_sphere_material(&mut sphere_materials, &assets.fireball_explosion_sphere);

        commands.spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(origin).with_scale(Vec3::splat(0.1)),
            explosion,
            crate::game::components::OnGameplayScreen,
        ));

        // Sparks, smoke, and heat shimmer are spawned by update_explosions on first frame

        audio::play_impact_sfx(
            &mut commands,
            &sfx.fireball_impact,
            origin,
            &game_config,
            &sfx,
        );

        // Apply detonation damage
        for (target_transform, team, mut health, temp_hp) in &mut targets {
            if *team != Team::Attackers {
                continue;
            }

            let dist = origin.distance(target_transform.translation);
            if dist <= detonation.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    detonation.damage,
                );
                // Don't insert Corpse here — let convert_dead_to_corpses handle
                // the full death pipeline (material swap, lay flat, etc.)
            }
        }
    }
}

/// Tier 3: Perpetual Unrest — dead enemies near PerpetualUnrest undead are auto-raised.
#[allow(clippy::too_many_arguments)]
pub fn handle_perpetual_unrest(
    mut commands: Commands,
    mut raised_this_frame: Local<HashSet<Entity>>,
    new_corpses: Query<
        (Entity, &Transform, &Team),
        (
            With<Corpse>,
            Without<PermanentCorpse>,
            Without<RaisedUndead>,
        ),
    >,
    unrest_undead: Query<(&Transform, &PerpetualUnrest), (With<RaisedUndead>, Without<Corpse>)>,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    raised_this_frame.clear();
    let talent_params = compute_talent_params(active_talents.as_deref());

    for (corpse_entity, corpse_transform, team) in &new_corpses {
        if *team != Team::Attackers {
            continue;
        }

        let should_raise = unrest_undead.iter().any(|(undead_transform, unrest)| {
            undead_transform
                .translation
                .distance(corpse_transform.translation)
                <= unrest.raise_radius
        });

        if should_raise && raised_this_frame.insert(corpse_entity) {
            let mut hp_mult = 1.0;
            if talent_params.empowered_undead {
                hp_mult *= constants::EMPOWERED_UNDEAD_HP_MULT;
            }

            raise_corpse_as_undead(
                &mut commands,
                corpse_entity,
                corpse_transform.translation,
                UNIT_HEALTH * hp_mult,
                UNIT_MOVEMENT_SPEED * 0.5,
                &talent_params,
                1.0,
                &undead_assets,
                &mut materials,
                talent_progress.as_deref_mut(),
            );
        }
    }
}

/// Tier 3: Revenant Lord — passively resurrects nearby corpses as undead minions.
#[allow(clippy::too_many_arguments)]
pub fn tick_revenant_raise(
    time: Res<Time>,
    mut commands: Commands,
    mut revenant_query: Query<
        (&Transform, &mut RevenantLord),
        (With<RaisedUndead>, Without<Corpse>),
    >,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let talent_params = compute_talent_params(active_talents.as_deref());

    // Strip revenant_lord so passively raised minions don't become Revenants
    let mut minion_params = talent_params.clone();
    minion_params.revenant_lord = false;

    for (rev_transform, mut revenant) in &mut revenant_query {
        revenant.raise_timer += delta;
        if revenant.raise_timer < revenant.raise_interval {
            continue;
        }
        revenant.raise_timer -= revenant.raise_interval;

        if let Some((corpse_entity, position)) = find_nearest_corpse(
            &corpse_query,
            rev_transform.translation,
            revenant.raise_radius,
        ) {
            raise_corpse_as_undead(
                &mut commands,
                corpse_entity,
                position,
                UNIT_HEALTH,
                UNIT_MOVEMENT_SPEED * 0.5,
                &minion_params,
                1.0,
                &undead_assets,
                &mut materials,
                talent_progress.as_deref_mut(),
            );
        }
    }
}
