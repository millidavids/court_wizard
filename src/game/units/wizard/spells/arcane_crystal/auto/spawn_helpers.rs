use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::disintegrate::systems as disintegrate_systems;
use crate::game::units::wizard::spells::disintegrate_constants;
use crate::game::units::wizard::spells::magic_missile::components::TargetTeams;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

use super::super::components::*;
use super::super::constants::*;
use super::super::setup::crystal_beam_geometry;

/// Spawns a crystal mini magic missile with pre-advanced homing.
///
/// Shared helper for both absorption and auto-cast. `target_teams` is the
/// caster's hostile team set — supplied by the calling system after reading
/// `PeerId` (guest targets Defenders, host targets Attackers).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_crystal_mini_missile(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    crystal_position: Vec3,
    crystal_range: f32,
    initial_velocity: Vec3,
    wobble_offset: f32,
    target: Option<Entity>,
    visual_radius: f32,
    damage_mult: f32,
    target_teams: TargetTeams,
) {
    let mut mini_missile =
        crate::game::units::wizard::spells::magic_missile::components::MagicMissile::new(
            initial_velocity,
            wobble_offset,
            target,
            DAMAGE_SCALE * damage_mult,
            target_teams,
            crystal_range,
            crystal_position,
        );
    mini_missile.time_alive = MINI_MISSILE_HOMING_ADVANCE;

    commands.spawn((
        mini_missile,
        CrystalSpawn {
            origin: crystal_position,
            max_range: crystal_range,
            lifetime: None,
        },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.crystal_mini_missile.clone()),
        Transform::from_translation(crystal_position).with_scale(Vec3::splat(visual_radius)),
        OnGameplayScreen,
    ));
}

/// Spawns crystal disintegrate beam(s) with damage scaling and CrystalSpawn marker.
/// When forked talent is active, spawns 3 beams in a fan pattern.
/// Returns all spawned beam entities.
pub(crate) fn spawn_crystal_disintegrate_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    max_range: f32,
    empowerment: f32,
    talent_cfg: Option<&disintegrate_systems::TalentConfig>,
) -> Vec<Entity> {
    let (base_direction, length) = crystal_beam_geometry(origin, target, max_range);
    let forked = talent_cfg.is_some_and(|cfg| cfg.forked);

    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };

    let mut entities = Vec::with_capacity(offsets.len());
    for &offset in offsets {
        let direction = if offset.abs() > 0.001 {
            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
        } else {
            base_direction
        };
        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
            commands,
            assets,
            origin,
            direction,
            length,
            empowerment,
            disintegrate_constants::DAMAGE_PER_TICK * BEAM_DAMAGE_SCALE,
            talent_cfg,
            BEAM_DAMAGE_SCALE,
            offset,
        );
        commands.entity(beam_entity).insert(CrystalSpawn {
            origin,
            max_range,
            lifetime: None,
        });
        entities.push(beam_entity);
    }
    entities
}

/// Spawns Finger of Death beams at a target position.
/// Returns all spawned beam entities.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fod_beams_at(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target_pos: Vec3,
    range: f32,
    empowerment: f32,
    fod_damage_per_tick: f32,
    talent_cfg: &disintegrate_systems::TalentConfig,
    beam_damage_scale: f32,
) {
    let (base_direction, length) = crystal_beam_geometry(origin, target_pos, range);
    let forked = talent_cfg.forked;
    let offsets: &[f32] = if forked {
        &[-FORKED_FAN_HALF_ANGLE, 0.0, FORKED_FAN_HALF_ANGLE]
    } else {
        &[0.0]
    };
    for &offset in offsets {
        let direction = if offset.abs() > 0.001 {
            Quat::from_axis_angle(Vec3::Y, offset) * base_direction
        } else {
            base_direction
        };
        let beam_entity = disintegrate_systems::spawn_beam_with_damage(
            commands,
            assets,
            origin,
            direction,
            length,
            empowerment,
            fod_damage_per_tick,
            Some(talent_cfg),
            beam_damage_scale,
            offset,
        );
        commands.entity(beam_entity).insert(CrystalSpawn {
            origin,
            max_range: range,
            lifetime: Some(BEAM_DURATION),
        });
    }
}
