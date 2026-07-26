use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::setup::{
    find_random_enemies_in_range, find_random_targets_in_range, scaled_count,
};
use super::auto_cast::CrystalAutocastParams;
use super::spawn_helpers::{spawn_crystal_mini_missile, spawn_fod_beams_at};

use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::chain_lightning::constants as cl_constants;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::fireball::systems as fireball_systems;
use crate::game::units::wizard::spells::magic_missile::components::TargetTeams;
use crate::game::units::wizard::spells::meteor_fall::casting::MeteorProjectileTalentFlags;
use crate::game::units::wizard::spells::meteor_fall::systems as meteor_fall_systems;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::{
    disintegrate_constants, fireball_constants, magic_missile_constants, meteor_fall_constants,
};

/// Auto-casts mini magic missiles at random enemies (the caster's hostile
/// team set, passed in by the system that knows the local `PeerId`).
pub(super) fn auto_cast_magic_missiles(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    enemies: &Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    target_teams: TargetTeams,
) {
    let targets = find_random_enemies_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_MISSILE_COUNT, params.count_mult),
        enemies,
        target_teams,
    );
    let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

    for (target_entity, target_pos) in &targets {
        let direction = (*target_pos - params.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;

        let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            commands,
            assets,
            params.position,
            params.range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
            params.damage_mult,
            target_teams,
        );
    }
}

/// Auto-casts mini fireballs at random enemies.
pub(super) fn auto_cast_fireballs(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(MINI_FB_COUNT, params.count_mult),
        targets,
    );
    let mini_radius = fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE * 0.5;

    for (_, target_pos) in &enemies {
        let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
        let direction = (ground_target - params.position).normalize();
        let speed = fireball_constants::PROJECTILE_SPEED * SPEED_SCALE;
        let velocity = direction * speed;

        let entity = fireball_systems::spawn_fireball_entity(
            commands,
            assets,
            params.position,
            velocity,
            fireball_constants::DAMAGE_PER_TICK * DAMAGE_SCALE * params.damage_mult,
            fireball_constants::DAMAGE_TYPE,
            fireball_constants::EXPLOSION_RADIUS * SIZE_SCALE,
            fireball_constants::PROJECTILE_COLLISION_RADIUS * SIZE_SCALE,
            params.empowerment * DAMAGE_SCALE * params.damage_mult,
            mini_radius,
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts chain lightning arcs at random enemies.
pub(super) fn auto_cast_chain_lightning(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    health_query: &mut Query<
        (
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<crate::game::pathfinding::StagingAttacker>,
    >,
    caster_team: Team,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(LIGHTNING_ARC_COUNT, params.count_mult),
        targets,
    );
    let damage =
        cl_constants::INITIAL_DAMAGE * DAMAGE_SCALE * params.empowerment * params.damage_mult;

    for (target_entity, target_pos) in &enemies {
        if let Ok((mut health, mut temp_hp, has_spell_shield, team)) =
            health_query.get_mut(*target_entity)
        {
            apply_spell_damage_with_team(
                commands,
                *target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Electric,
                has_spell_shield,
                caster_team,
                *team,
            );
        }

        chain_lightning_systems::spawn_arc(
            commands,
            assets,
            params.position,
            *target_pos,
            0,
            params.empowerment,
        );
    }
}

/// Auto-casts mini meteors at random enemies.
pub(super) fn auto_cast_meteors(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(2, params.count_mult),
        targets,
    );
    let mini_radius = meteor_fall_constants::METEOR_MESH_RADIUS * SIZE_SCALE;

    for (_, target_pos) in &enemies {
        let spawn_pos = Vec3::new(target_pos.x, MINI_METEOR_SPAWN_HEIGHT, target_pos.z);
        let damage = meteor_fall_constants::METEOR_DAMAGE * DAMAGE_SCALE * params.damage_mult;
        let explosion_radius = meteor_fall_constants::EXPLOSION_RADIUS * SIZE_SCALE;

        let entity = meteor_fall_systems::spawn_meteor_projectile_entity(
            commands,
            assets,
            spawn_pos,
            Vec3::new(0.0, meteor_fall_constants::METEOR_INITIAL_VELOCITY, 0.0),
            damage,
            explosion_radius,
            params.empowerment,
            mini_radius,
            MeteorProjectileTalentFlags::default(),
        );
        commands.entity(entity).insert(CrystalSpawn {
            origin: params.position,
            max_range: params.range,
            lifetime: None,
        });
    }
}

/// Auto-casts Finger of Death beams at random enemies.
pub(super) fn auto_cast_fod_beams(
    rng: &mut impl Rng,
    params: &CrystalAutocastParams,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    targets: &Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    talent_cfg: &disintegrate_systems::TalentConfig,
) {
    let enemies = find_random_targets_in_range(
        rng,
        params.position,
        params.range,
        scaled_count(BEAM_COUNT, params.count_mult),
        targets,
    );
    let fod_damage_per_tick = FOD_ECHO_BASE_DAMAGE * BEAM_DAMAGE_SCALE * params.damage_mult
        / (BEAM_DURATION / disintegrate_constants::DAMAGE_INTERVAL);

    for (_, target_pos) in &enemies {
        spawn_fod_beams_at(
            commands,
            assets,
            params.position,
            *target_pos,
            params.range,
            params.empowerment,
            fod_damage_per_tick,
            talent_cfg,
            BEAM_DAMAGE_SCALE,
        );
    }
}
