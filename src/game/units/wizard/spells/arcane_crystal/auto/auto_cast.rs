use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::setup::{
    crystal_beam_geometry, crystal_target_teams, find_random_targets_in_range,
};
use super::spawn_helpers::spawn_crystal_disintegrate_beam;
use super::spell_variants::{
    auto_cast_chain_lightning, auto_cast_fireballs, auto_cast_fod_beams, auto_cast_magic_missiles,
    auto_cast_meteors,
};

use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::session::MultiplayerSession;

/// Shared parameters for crystal auto-cast helper functions.
pub(super) struct CrystalAutocastParams {
    pub(super) position: Vec3,
    pub(super) range: f32,
    pub(super) empowerment: f32,
    pub(super) damage_mult: f32,
    pub(super) count_mult: f32,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn auto_cast_remembered_spell(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — see hits.rs for rationale.
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut crystal_beams: Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    active_talents: Option<Res<ActiveTalents>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let target_teams = crystal_target_teams(session.as_deref());
    let talent_cfg = disintegrate_systems::compute_talent_config(active_talents.as_deref());
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    // Collect crystal data to avoid borrow conflicts (skip permanent turrets).
    // Store Entity IDs for O(1) lookup via get_mut() instead of O(n) iter_mut().nth().
    let crystal_data: Vec<_> = crystals
        .iter()
        .filter(|(_, c)| !c.permanent)
        .map(|(e, c)| {
            (
                e,
                c.position,
                c.range,
                c.empowerment,
                c.remembered_spell,
                c.auto_cast_timer,
                c.auto_disintegrate_beam.clone(),
                c.damage_mult,
                c.count_mult,
            )
        })
        .collect();

    for (
        entity,
        position,
        range,
        empowerment,
        remembered,
        timer,
        auto_beam,
        damage_mult,
        count_mult,
    ) in crystal_data.into_iter()
    {
        let Some(remembered) = remembered else {
            // No remembered spell — clean up any lingering auto-disintegrate beam
            if let Some((beam_entities, _)) = &auto_beam {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
                if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                    crystal.auto_disintegrate_beam = None;
                }
            }
            continue;
        };

        // === Special case: Disintegrate = constant single beam ===
        if remembered == RememberedSpell::Disintegrate {
            handle_auto_disintegrate(
                &mut game_rng.0,
                entity,
                position,
                range,
                empowerment,
                auto_beam,
                &mut commands,
                &visual_assets,
                &mut crystal_beams,
                &targets,
                &mut crystals,
                &talent_cfg,
            );
            continue;
        }

        // === Timer-based auto-cast for all other spells ===

        // Clean up any lingering auto-disintegrate beam if spell changed
        if let Some((beam_entities, _)) = &auto_beam {
            for beam_entity in beam_entities {
                commands.entity(*beam_entity).try_despawn();
            }
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_disintegrate_beam = None;
            }
        }

        let new_timer = timer + delta;
        let interval = remembered.auto_cast_interval();

        if new_timer >= interval {
            // Reset timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = 0.0;
                crystal.trigger_pulse();
            }

            let autocast = CrystalAutocastParams {
                position,
                range,
                empowerment,
                damage_mult,
                count_mult,
            };

            match remembered {
                RememberedSpell::MagicMissile => {
                    auto_cast_magic_missiles(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &enemies,
                        target_teams,
                    );
                }
                RememberedSpell::Fireball => {
                    auto_cast_fireballs(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::ChainLightning => {
                    auto_cast_chain_lightning(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &mut health_query,
                        caster_team,
                    );
                }
                RememberedSpell::Meteor => {
                    auto_cast_meteors(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                    );
                }
                RememberedSpell::FingerOfDeath => {
                    auto_cast_fod_beams(
                        &mut game_rng.0,
                        &autocast,
                        &mut commands,
                        &visual_assets,
                        &targets,
                        &talent_cfg,
                    );
                }
                RememberedSpell::Disintegrate => unreachable!(),
            }
        } else {
            // Just advance timer
            if let Ok((_, mut crystal)) = crystals.get_mut(entity) {
                crystal.auto_cast_timer = new_timer;
            }
        }
    }
}

/// Manages a persistent auto-disintegrate beam group.
///
/// Instead of despawning/respawning beams every frame (which resets time_alive
/// and breaks the growth animation + damage), we update beam fields in-place.
/// New beams are only spawned when the old target dies/leaves range.
#[allow(clippy::too_many_arguments)]
fn handle_auto_disintegrate(
    rng: &mut impl Rng,
    crystal_entity: Entity,
    position: Vec3,
    range: f32,
    empowerment: f32,
    auto_beam: Option<(Vec<Entity>, Entity)>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_beams: &mut Query<(Entity, &mut DisintegrateBeam), With<CrystalSpawn>>,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    crystals: &mut Query<
        (Entity, &mut ArcaneCrystal),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    if let Some((beam_entities, target_entity)) = auto_beam {
        // Check if any beam entity still exists
        let any_alive = beam_entities.iter().any(|e| crystal_beams.get(*e).is_ok());
        if !any_alive {
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = None;
            }
            // Fall through to spawn a new beam below
        } else if let Ok((_, target_transform)) = targets.get(target_entity) {
            // Target alive — check range
            let dist = xz_distance(position, target_transform.translation);
            if dist <= range {
                // Target still valid — update all beams to track it in-place
                let (base_direction, length) =
                    crystal_beam_geometry(position, target_transform.translation, range);
                for beam_entity in &beam_entities {
                    if let Ok((_, mut beam)) = crystal_beams.get_mut(*beam_entity) {
                        beam.origin = position;
                        beam.direction =
                            Quat::from_axis_angle(Vec3::Y, beam.fan_offset_angle) * base_direction;
                        beam.length = length;
                    }
                }
                return;
            }
            // Target out of range — fall through to replace beam
        } else {
            // Target dead — fall through to replace beam
        }
        // Despawn old beams and find new target
        despawn_beam_group(commands, &beam_entities);
        let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
        if let Some((new_target, new_pos)) = new_targets.first() {
            let new_beams = spawn_crystal_disintegrate_beam(
                commands,
                assets,
                position,
                *new_pos,
                range,
                empowerment,
                Some(talent_cfg),
            );
            if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
                crystal.auto_disintegrate_beam = Some((new_beams, *new_target));
            }
        } else if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = None;
        }
        return;
    }

    // No beam exists — try to spawn one
    let new_targets = find_random_targets_in_range(rng, position, range, 1, targets);
    if let Some((target_entity, target_pos)) = new_targets.first() {
        let beam_entities = spawn_crystal_disintegrate_beam(
            commands,
            assets,
            position,
            *target_pos,
            range,
            empowerment,
            Some(talent_cfg),
        );
        if let Some(mut crystal) = crystals.get_mut(crystal_entity).ok().map(|(_, c)| c) {
            crystal.auto_disintegrate_beam = Some((beam_entities, *target_entity));
        }
    }
}

/// Helper to despawn all beams in a group.
fn despawn_beam_group(commands: &mut Commands, beam_entities: &[Entity]) {
    for beam_entity in beam_entities {
        commands.entity(*beam_entity).try_despawn();
    }
}
