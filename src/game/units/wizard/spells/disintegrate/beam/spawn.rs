use super::super::casting::TalentConfig;
use super::super::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam, DisintegrateParticle,
    SearingFinaleDetonation,
};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::utils::UniqueHitTracker;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_damage(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    damage_per_tick: f32,
    talent_cfg: Option<&TalentConfig>,
    mini_spell_scale: f32,
    fan_offset_angle: f32,
) -> Entity {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.damage_per_tick_override = Some(damage_per_tick);
    if let Some(cfg) = talent_cfg {
        beam.width_multiplier = cfg.width_multiplier;
        beam.damage_multiplier = cfg.damage_multiplier;
        beam.escalating = cfg.escalating;
        beam.resonance = cfg.resonance;
    }
    beam.mini_spell_scale = mini_spell_scale;
    beam.fan_offset_angle = fan_offset_angle;
    beam.ground_collision = true;
    // Crystal beams only get the core beam mesh — no glow, flare, or eclipse.
    spawn_beam_core(commands, assets, beam)
}

/// Spawns a beam with talent configuration applied.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    cfg: &TalentConfig,
    fan_offset_angle: f32,
    annihilation_cast_pos: Vec3,
    annihilation_sweep_dir: Vec3,
) {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.width_multiplier = cfg.width_multiplier;
    beam.damage_multiplier = cfg.damage_multiplier;
    beam.fan_offset_angle = fan_offset_angle;
    beam.escalating = cfg.escalating;
    beam.sweeping = cfg.sweeping;
    beam.searing_finale = cfg.searing_finale;
    beam.resonance = cfg.resonance;
    beam.annihilation = cfg.annihilation;
    beam.annihilation_cast_pos = annihilation_cast_pos;
    if cfg.sweeping {
        if cfg.annihilation {
            // For sky beams, sweep_center_direction stores the XZ forward reference
            beam.sweep_center_direction = annihilation_sweep_dir;
        } else {
            beam.sweep_center_direction = direction;
        }
        beam.sweep_direction = 1.0;
    }
    spawn_beam_visuals(commands, assets, beam);
}

/// Spawns only the core beam entity (mesh + component). Used by crystal beams
/// which don't need glow, flare, or eclipse siblings.
pub(super) fn spawn_beam_core(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    commands
        .spawn((
            beam,
            UniqueHitTracker::default(),
            Mesh3d(assets.disintegrate_cone.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns the core beam entity plus glow, flare, and eclipse siblings.
/// Used by wizard-cast disintegrate beams.
fn spawn_beam_visuals(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);
    let beam_entity = spawn_beam_core(commands, assets, beam);

    // Glow cone sibling (wider, semi-transparent)
    commands.spawn((
        BeamGlow { beam_entity },
        Mesh3d(assets.disintegrate_cone.clone()),
        MeshMaterial3d(assets.disintegrate_glow.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Origin flare sphere
    commands.spawn((
        BeamOriginFlare { beam_entity },
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(assets.disintegrate_flare.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Ground eclipse at beam impact point
    commands.spawn((
        BeamEclipse { beam_entity },
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.disintegrate_eclipse.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.05, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        OnGameplayScreen,
    ));

    beam_entity
}

/// Helper to despawn all beam-related visual entities.
pub(crate) fn despawn_all_beam_visuals(
    commands: &mut Commands,
    beams: &Query<(Entity, &mut DisintegrateBeam), Without<CrystalSpawn>>,
    glow_query: &Query<Entity, With<BeamGlow>>,
    flare_query: &Query<Entity, With<BeamOriginFlare>>,
    particle_query: &Query<Entity, With<DisintegrateParticle>>,
    eclipse_query: &Query<Entity, With<BeamEclipse>>,
) {
    for (entity, _) in beams.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in glow_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in flare_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in particle_query.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in eclipse_query.iter() {
        commands.entity(entity).try_despawn();
    }
}

/// Spawns a searing finale detonation entity along a beam's path.
pub(crate) fn spawn_searing_finale(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: &DisintegrateBeam,
) {
    let current_len = beam.current_length();
    if current_len < 1.0 {
        return;
    }

    let midpoint = beam.origin + beam.direction * (current_len / 2.0);
    let half_width = beam.beam_width() * constants::SEARING_FINALE_WIDTH_MULT;
    let damage =
        beam.damage_per_tick() / constants::DAMAGE_INTERVAL * constants::SEARING_FINALE_DAMAGE_MULT;

    let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

    commands.spawn((
        SearingFinaleDetonation {
            origin: beam.origin,
            direction: beam.direction,
            length: current_len,
            half_width,
            damage,
            time_alive: 0.0,
            damage_applied: false,
        },
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.searing_finale.clone()),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(half_width * 2.0, current_len, half_width * 2.0)),
        OnGameplayScreen,
    ));
}
