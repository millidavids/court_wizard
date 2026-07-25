use bevy::prelude::*;
use rand::Rng;

use super::super::components::{
    FireDoT, FrostAccumulation, PendingDamageEffect, PoisonedModifier, RootedModifier, Shocked,
    SmellyModifier, Team,
};
use super::super::constants::{
    FROST_ACCUMULATION_PER_HIT, FROST_GENERIC_DECAY_DELAY, POISON_DURATION,
    POISON_EFFECTIVENESS_CAP, POISON_EFFECTIVENESS_PER_STACK, SMELLY_DURATION,
};
use super::super::damage::DamageType;
use super::super::king::components::SpellShield;
use crate::config::GameConfig;
use crate::config::WizardType;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::archetypes::meteorologist::components::{
    BurningPatch, ColdModifier, DryModifier,
};
use crate::game::units::wizard::archetypes::meteorologist::constants::{
    BURNING_PATCH_COLOR, BURNING_PATCH_DPS, BURNING_PATCH_LIFETIME, BURNING_PATCH_RADIUS,
    BURNING_PATCH_TICK_INTERVAL, COLD_FREEZE_DURATION, COLD_FROST_SLOW_MULTIPLIER,
    DRY_BURNING_PATCH_COUNT, DRY_BURNING_PATCH_SCATTER,
};

/// Processes `PendingDamageEffect` markers and creates/stacks persistent effects.
///
/// Each frame, reads all PendingDamageEffect components, determines the damage type,
/// and either creates a new persistent effect component or stacks onto an existing one.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn process_pending_damage_effects(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    config: Res<GameConfig>,
    // `Without<GhostEntity>` keeps multiplayer ghost units out of the
    // local SP pipeline — those get their PendingDamageEffect forwarded
    // to the host via `forward_spell_hits_to_host` and processed on the
    // host's authoritative copy instead.
    pending_query: Query<
        (
            Entity,
            &PendingDamageEffect,
            &Transform,
            // Optional: most damageable entities are units with a Team, but some
            // (e.g. the multiplayer wizard) have Health and no Team. Keeping this
            // optional avoids silently dropping their PendingDamageEffect.
            Option<&Team>,
            Has<SpellShield>,
            Has<ColdModifier>,
            Has<DryModifier>,
        ),
        (
            Without<crate::game::multiplayer::components::GhostEntity>,
            // Defense-in-depth chokepoint: even if a future spell forgets its
            // own staging filter and banks a PendingDamageEffect on a staging
            // unit, it must never materialize into a DoT.
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut fire_query: Query<&mut FireDoT>,
    mut frost_query: Query<&mut FrostAccumulation>,
    mut electric_query: Query<&mut Shocked>,
    mut poison_query: Query<&mut PoisonedModifier>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Shared mesh/material for burning patches (Drought fire synergy), created once if needed
    let mut burning_patch_mesh: Option<Handle<Mesh>> = None;
    let mut burning_patch_material: Option<Handle<StandardMaterial>> = None;

    for (entity, pending, transform, team, has_shield, has_cold, has_dry) in pending_query.iter() {
        // A King's SpellShield blocks the DoT only when the spell came from the
        // ENEMY team. Friendly fire (same team) still applies. `source_team ==
        // None` keeps the old block-all behavior. Team-less entities never carry a
        // shield, so they're always processed.
        let shield_blocks = has_shield
            && match team {
                Some(t) => pending.source_team != Some(*t),
                None => true,
            };
        if shield_blocks {
            commands.entity(entity).remove::<PendingDamageEffect>();
            continue;
        }
        // Excremage converts all damage types to Poop
        let effective_type = if config.wizard_type == WizardType::Excremage {
            DamageType::Poop
        } else {
            pending.damage_type
        };
        match effective_type {
            DamageType::Fire => {
                if let Ok(mut fire_dot) = fire_query.get_mut(entity) {
                    fire_dot.stack(pending.damage);
                } else {
                    commands.entity(entity).insert(FireDoT::new(pending.damage));
                }
                // Drought synergy: fire spells create burning ground patches
                if has_dry {
                    let impact_pos = transform.translation;
                    let patch_mesh = burning_patch_mesh
                        .get_or_insert_with(|| meshes.add(Circle::new(BURNING_PATCH_RADIUS)));
                    let patch_material = burning_patch_material.get_or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: BURNING_PATCH_COLOR,
                            unlit: true,
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        })
                    });
                    for _ in 0..DRY_BURNING_PATCH_COUNT {
                        let offset_x = game_rng
                            .0
                            .random_range(-DRY_BURNING_PATCH_SCATTER..DRY_BURNING_PATCH_SCATTER);
                        let offset_z = game_rng
                            .0
                            .random_range(-DRY_BURNING_PATCH_SCATTER..DRY_BURNING_PATCH_SCATTER);
                        let patch_pos = Vec3::new(
                            impact_pos.x + offset_x,
                            0.5, // Just above ground
                            impact_pos.z + offset_z,
                        );
                        commands.spawn((
                            BurningPatch {
                                lifetime: BURNING_PATCH_LIFETIME,
                                radius: BURNING_PATCH_RADIUS,
                                damage_per_tick: BURNING_PATCH_DPS * BURNING_PATCH_TICK_INTERVAL,
                                tick_timer: BURNING_PATCH_TICK_INTERVAL,
                            },
                            Mesh3d(patch_mesh.clone()),
                            MeshMaterial3d(patch_material.clone()),
                            Transform::from_translation(patch_pos)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                            OnGameplayScreen,
                        ));
                    }
                }
            }
            DamageType::Frost => {
                // Frost damage drives accumulation (progressive slow + freeze)
                let frost_amount = if has_cold {
                    FROST_ACCUMULATION_PER_HIT * COLD_FROST_SLOW_MULTIPLIER
                } else {
                    FROST_ACCUMULATION_PER_HIT
                };

                if let Ok(mut frost) = frost_query.get_mut(entity) {
                    frost.add_frost(frost_amount, FROST_GENERIC_DECAY_DELAY);
                } else {
                    commands.entity(entity).insert(FrostAccumulation::new(
                        frost_amount,
                        FROST_GENERIC_DECAY_DELAY,
                    ));
                }
                // Blizzard synergy: frost + cold = freeze (brief root)
                if has_cold {
                    commands
                        .entity(entity)
                        .insert(RootedModifier::new(COLD_FREEZE_DURATION));
                }
            }
            DamageType::Electric => {
                if let Ok(mut charge) = electric_query.get_mut(entity) {
                    charge.stack(pending.damage);
                } else {
                    commands.entity(entity).insert(Shocked::new(pending.damage));
                }
            }
            DamageType::Poison => {
                if let Ok(mut poison) = poison_query.get_mut(entity) {
                    poison.stack(
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
            }
            DamageType::Poop => {
                commands
                    .entity(entity)
                    .insert(SmellyModifier::new(SMELLY_DURATION));
            }
            // Force, Necrotic, Nature — no persistent effect
            _ => {}
        }

        commands.entity(entity).remove::<PendingDamageEffect>();
    }
}
