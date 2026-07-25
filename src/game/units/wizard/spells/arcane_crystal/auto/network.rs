use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::setup::{
    crystal_target_teams, find_random_enemies_in_range, find_random_targets_in_range, scaled_count,
};
use super::spawn_helpers::spawn_crystal_mini_missile;

use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::meteor_fall::casting::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    fireball_constants, magic_missile_constants, meteor_fall_constants,
};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// When a crystal absorbs a spell, chain the absorption to nearby crystals
/// with CrystalNetwork marker. Each chain re-triggers the crystal's emission
/// at reduced damage.
#[allow(clippy::too_many_arguments)]
pub(crate) fn crystal_network_chain(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal),
        (
            With<CrystalNetwork>,
            Without<crate::game::multiplayer::components::GhostSpellEffect>,
        ),
    >,
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    enemies: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    // Early return if no crystal absorbed this frame
    if !crystals.iter().any(|(_, c)| c.just_absorbed) {
        return;
    }
    let target_teams = crystal_target_teams(session.as_deref());

    // Collect crystal identity, position, and pulse state for chaining;
    // mutable fields (range, empowerment, spell) are re-fetched via get_mut below.
    let crystal_data: Vec<(Entity, Vec3, Option<RememberedSpell>, bool)> = crystals
        .iter()
        .map(|(e, c)| (e, c.position, c.remembered_spell, c.just_absorbed))
        .collect();

    // For each crystal that just pulsed, check if nearby crystals should chain
    for (source_entity, source_pos, source_spell, source_pulsed) in &crystal_data {
        if !source_pulsed {
            continue;
        }
        let Some(remembered) = source_spell else {
            continue;
        };

        for (target_entity, target_pos, _target_spell, target_pulsed) in &crystal_data {
            if source_entity == target_entity || *target_pulsed {
                continue;
            }

            let dist = xz_distance(*source_pos, *target_pos);

            if dist > CRYSTAL_NETWORK_CHAIN_RANGE {
                continue;
            }

            // Chain: trigger the target crystal to emit based on the source's remembered spell
            if let Ok((_, mut target_crystal)) = crystals.get_mut(*target_entity) {
                target_crystal.trigger_pulse();

                // Emit a reduced emission from the chained crystal
                let chain_damage_scale = DAMAGE_SCALE * target_crystal.damage_mult * 0.5;
                match remembered {
                    RememberedSpell::MagicMissile => {
                        let count =
                            scaled_count(MINI_MISSILE_COUNT / 2 + 1, target_crystal.count_mult);
                        let mini_targets = find_random_enemies_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &enemies,
                            target_teams,
                        );
                        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;
                        for (te, tp) in &mini_targets {
                            let direction = (*tp - target_crystal.position).normalize();
                            let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                            let rng = &mut game_rng.0;
                            let wobble = rng.random_range(0.0..std::f32::consts::TAU);
                            spawn_crystal_mini_missile(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                target_crystal.range,
                                direction * speed,
                                wobble,
                                Some(*te),
                                mini_radius,
                                target_crystal.damage_mult,
                                target_teams,
                            );
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Fireball => {
                        let count = scaled_count(MINI_FB_COUNT / 2 + 1, target_crystal.count_mult);
                        let fire_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius =
                            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;
                        for (_, tp) in &fire_targets {
                            let ground = Vec3::new(tp.x, 0.0, tp.z);
                            let direction = (ground - target_crystal.position).normalize();
                            let velocity =
                                direction * fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
                            let entity = fireball_systems::spawn_fireball_entity(
                                &mut commands,
                                &visual_assets,
                                target_crystal.position,
                                velocity,
                                fireball_constants::DAMAGE_PER_TICK * chain_damage_scale,
                                fireball_constants::DAMAGE_TYPE,
                                fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment * chain_damage_scale,
                                mini_radius,
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    RememberedSpell::Meteor => {
                        let count = scaled_count(1, target_crystal.count_mult);
                        let meteor_targets = find_random_targets_in_range(
                            &mut game_rng.0,
                            target_crystal.position,
                            target_crystal.range,
                            count,
                            &targets,
                        );
                        let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;
                        for (_, tp) in &meteor_targets {
                            let spawn_pos = Vec3::new(tp.x, MINI_METEOR_SPAWN_HEIGHT, tp.z);
                            let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
                                &mut commands,
                                &visual_assets,
                                spawn_pos,
                                Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
                                meteor_fall_constants::METEOR_DAMAGE * chain_damage_scale,
                                meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE,
                                target_crystal.empowerment,
                                mini_radius,
                                MeteorProjectileTalentFlags::default(),
                            );
                            commands.entity(entity).insert(CrystalSpawn {
                                origin: target_crystal.position,
                                max_range: target_crystal.range,
                                lifetime: None,
                            });
                        }
                        progress.increment(Spell::ArcaneCrystal, count as u32);
                    }
                    _ => {
                        // Chain lightning, disintegrate, FoD — just pulse without extra emission
                        // (beams are too complex to duplicate in chain)
                    }
                }

                // Spawn visual arc between crystals
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    *source_pos,
                    target_crystal.position,
                    0,
                    target_crystal.empowerment,
                );
            }
        }
    }
}
