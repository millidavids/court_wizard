use super::super::components::*;
use super::super::constants;
use super::suppress::{
    collect_dispellable_effects, is_offensive_effect, remove_mind_control_in_radius,
    strip_spell_shields_in_radius, suppress_spell_effects_in_radius,
};
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::components::{
    BattleHymnModifier, BerserkerRageModifier, Corpse, FogEvasionModifier, HasteModifier, Health,
    MindControlled, Petrified, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{LocalWizard, Mana, Spell};
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::vfx::constants as vfx_constants;
use crate::game::units::wizard::spells::vfx::systems::spawn_explosion_smoke;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use bevy::prelude::*;

// ===== Talent Params =====

/// Computes talent parameters from active talent selections.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_dispel_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    visual_assets: Res<SpellVisualAssets>,
    // Host-only — without this filter, a host-cast Dispel's ghost impact on
    // the guest would also run suppress_spell_effects_in_radius, locally
    // despawning other ghost spell effects whose stale-id then prevents
    // them from re-spawning from snapshot (permanent invisibility).
    mut impacts: Query<
        (
            Entity,
            &mut DispelImpact,
            &mut Transform,
            Has<BroadSpectrum>,
            Has<ManaDrain>,
            Has<ExplosiveNullification>,
            Has<SpellReflection>,
            Has<NullZoneOnImpact>,
            Has<WizardCastDispel>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect), Without<DispelImpact>>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut wizard_mana: Query<&mut Mana, With<LocalWizard>>,
    progress_and_toggles: (ResMut<BattleTalentProgress>, Option<Res<ActiveToggles>>),
    // Combined query for buff removal, damage, enemy finding, and mind control removal
    mut unit_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Has<HasteModifier>,
            Has<BerserkerRageModifier>,
            Has<BattleHymnModifier>,
            Has<FogEvasionModifier>,
            Has<MindControlled>,
            Has<Petrified>,
        ),
        (Without<Corpse>, Without<DispelImpact>),
    >,
) {
    let (mut progress, active_toggles) = progress_and_toggles;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let time_secs = time.elapsed_secs();
    let mut damage_targets: Vec<(Entity, f32, bool)> = Vec::new();

    for (
        entity,
        mut impact,
        mut transform,
        has_broad_spectrum,
        has_mana_drain,
        has_explosive,
        has_reflection,
        has_null_zone,
        has_wizard_cast,
    ) in &mut impacts
    {
        impact.time_alive += time.delta_secs();

        if impact.time_alive >= impact.duration {
            // Null Zone: spawn persistent anti-magic zone at impact point before despawning
            if has_null_zone {
                spawn_null_zone(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    transform.translation,
                    scorched_mult,
                );
            }
            commands.entity(entity).try_despawn();
            continue;
        }

        // Expand at constant speed (Counterspell talent makes this faster via expand_speed)
        let radius = impact.expand_speed * impact.time_alive;
        transform.scale = Vec3::splat(radius);

        let impact_center = transform.translation;

        // Collect dispellable spell effects once for this frame
        let all_dispellable: Vec<_> = collect_dispellable_effects(
            spell_effects
                .iter()
                .map(|(e, tf, nse)| (e, tf.translation, nse.kind)),
        );

        // Suppress all dispellable spell effects within radius
        let dispelled = suppress_spell_effects_in_radius(
            &mut commands,
            impact_center,
            radius,
            &all_dispellable,
            &wall_of_fire_query,
            &wall_of_stone_query,
            &spike_growth_query,
            &grease_query,
            &meteor_fire_query,
            &mut obstacle_events,
        );
        let mut dispelled_count = dispelled.len() as u32;

        // Talent effects on each dispelled spell effect
        for &(_spell_entity, effect_pos, effect_kind) in &dispelled {
            // Mana Drain: refund mana
            if has_mana_drain {
                let refund = constants::spell_effect_mana_cost(effect_kind)
                    * constants::MANA_DRAIN_REFUND_FRACTION;
                if refund > 0.0
                    && let Ok(mut mana) = wizard_mana.single_mut()
                {
                    mana.regenerate(refund);
                }
            }

            // Spell Reflection: find nearest enemy target for reflected damage
            let reflection_target = if has_reflection && is_offensive_effect(effect_kind) {
                let mut best: Option<(f32, Vec3)> = None;
                for (_, tf, team, _, _, _, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    let d = xz_distance(tf.translation, effect_pos);
                    if best.is_none_or(|(bd, _)| d < bd) {
                        best = Some((d, tf.translation));
                    }
                }
                best.map(|(_, p)| p)
            } else {
                None
            };

            damage_targets.clear();

            // Explosive Nullification: damage enemies near the dispelled effect + VFX
            if has_explosive {
                spawn_dispel_explosion(&mut commands, &visual_assets, effect_pos, time_secs);
                for (entity, tf, team, _, _, has_shield, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    if xz_distance(tf.translation, effect_pos)
                        <= constants::EXPLOSIVE_NULLIFICATION_RADIUS
                    {
                        damage_targets.push((
                            entity,
                            constants::EXPLOSIVE_NULLIFICATION_DAMAGE,
                            has_shield,
                        ));
                    }
                }
            }

            // Spell Reflection: damage enemies near the reflected target
            if let Some(target_pos) = reflection_target {
                for (entity, tf, team, _, _, has_shield, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    if xz_distance(tf.translation, target_pos) <= constants::SPELL_REFLECTION_RADIUS
                    {
                        damage_targets.push((
                            entity,
                            constants::SPELL_REFLECTION_DAMAGE,
                            has_shield,
                        ));
                    }
                }
            }

            // Apply collected damage
            for &(target_entity, damage, has_shield) in &damage_targets {
                if let Ok((_, _, _, mut health, mut temp_hp, _, _, _, _, _, _, _)) =
                    unit_query.get_mut(target_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Force,
                        has_shield,
                    );
                }
            }
        }

        // Remove mind control from units in range
        let mc_freed = remove_mind_control_in_radius(
            &mut commands,
            impact_center,
            radius,
            unit_query
                .iter()
                .filter_map(|(entity, tf, _, _, _, _, _, _, _, _, has_mc, _)| {
                    has_mc.then_some((entity, tf.translation))
                }),
        );
        dispelled_count += mc_freed;

        // Strip spell shields from enemy units in range (shielder-applied shields)
        let shields_stripped = strip_spell_shields_in_radius(
            &mut commands,
            impact_center,
            radius,
            unit_query.iter().filter_map(
                |(entity, tf, team, _, _, has_shield, _, _, _, _, _, _)| {
                    (has_shield && Team::Defenders.is_enemy(team))
                        .then_some((entity, tf.translation))
                },
            ),
        );
        dispelled_count += shields_stripped;

        // Broad Spectrum: strip buffs from enemies in range
        if has_broad_spectrum {
            for (
                unit_entity,
                unit_tf,
                team,
                _health,
                _temp_hp,
                _has_shield,
                has_haste,
                has_rage,
                has_hymn,
                has_fog,
                _has_mind_control,
                has_petrified,
            ) in &unit_query
            {
                if xz_distance(unit_tf.translation, impact_center) > radius {
                    continue;
                }

                // Dispel cures petrified allies
                if *team == Team::Defenders && has_petrified {
                    commands.entity(unit_entity).remove::<Petrified>();
                }

                if Team::Defenders.is_enemy(team) {
                    let mut stripped = false;
                    commands.entity(unit_entity).remove::<TemporaryHitPoints>();
                    if has_haste {
                        commands.entity(unit_entity).remove::<HasteModifier>();
                        stripped = true;
                    }
                    if has_rage {
                        commands
                            .entity(unit_entity)
                            .remove::<BerserkerRageModifier>();
                        stripped = true;
                    }
                    if has_hymn {
                        commands.entity(unit_entity).remove::<BattleHymnModifier>();
                        stripped = true;
                    }
                    if has_fog {
                        commands.entity(unit_entity).remove::<FogEvasionModifier>();
                        stripped = true;
                    }
                    if stripped {
                        dispelled_count += 1;
                    }
                }
            }
        }

        // Track talent progress — only wizard-cast dispels count. Enemy
        // Dispellers spawn impacts without `WizardCastDispel`.
        if dispelled_count > 0 && has_wizard_cast {
            progress.increment(Spell::Dispel, dispelled_count);
        }
    }
}

/// Spawns a persistent Null Zone at the given position.
pub(crate) fn spawn_null_zone(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    scorched_mult: f32,
) {
    let origin = Vec3::new(position.x, 0.0, position.z);
    let radius = constants::NULL_ZONE_RADIUS;

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius, constants::NULL_ZONE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::NULL_ZONE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_translation(origin + Vec3::Y * (constants::NULL_ZONE_HEIGHT / 2.0)),
        NullZone {
            time_remaining: constants::NULL_ZONE_DURATION * scorched_mult,
            radius,
            origin,
        },
        OnGameplayScreen,
    ));
}

/// Spawns white sparks and smoke at a dispelled effect's position (Explosive Nullification VFX).
fn spawn_dispel_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
) {
    use crate::game::units::wizard::spells::vfx::systems::spawn_sparks_with_material;
    spawn_sparks_with_material(
        commands,
        assets,
        position,
        vfx_constants::SPARK_COUNT,
        time_secs,
        assets.dispel_spark.clone(),
    );
    spawn_explosion_smoke(commands, assets, position, time_secs);
}
