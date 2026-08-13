use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Spawns the crystal entity with visual mesh and range indicator.
pub(crate) fn spawn_crystal(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    talent_params: &ArcaneCrystalTalentParams,
) {
    let range = CRYSTAL_RANGE * empowerment * talent_params.range_mult;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;
    let duration = CRYSTAL_DURATION * empowerment * talent_params.duration_mult;

    let is_turret = talent_params.auto_crystal;
    let mut crystal = ArcaneCrystal::new(
        Vec3::ZERO, // placeholder — set by spawn_crystal_entity
        range,
        duration,
        collision_radius,
        empowerment,
    );
    crystal.damage_mult = talent_params.damage_mult;
    crystal.count_mult = talent_params.count_mult;
    crystal.spell_echo = talent_params.spell_echo;

    if is_turret {
        crystal.permanent = true;
        crystal.duration = f32::MAX;
    }

    let mut entity_commands =
        spawn_crystal_entity(commands, assets, position, empowerment, crystal);

    if is_turret {
        entity_commands.insert(AutoCrystalTimer { timer: 0.0 });
    }
    // Resonance cascade applies to both turrets and normal crystals
    if talent_params.resonance_cascade {
        entity_commands.insert(ResonanceCascade { absorptions: 0 });
    }
    if !is_turret {
        if talent_params.prismatic_explosion {
            entity_commands.insert(PrismaticExplosion);
        }
        if talent_params.crystal_network {
            entity_commands.insert(CrystalNetwork);
        }
    }
}

/// Spawns a permanent crystal turret from saved data (loaded between levels).
pub(crate) fn spawn_permanent_crystal(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &crate::config::save_data::SavedCrystal,
    damage_mult: f32,
    count_mult: f32,
    resonance_cascade: bool,
) {
    let position = Vec3::new(saved.x, 0.0, saved.z);
    let empowerment = saved.empowerment;
    let collision_radius = CRYSTAL_COLLISION_RADIUS * empowerment;

    let mut crystal = ArcaneCrystal::new(
        Vec3::ZERO,
        saved.range,
        f32::MAX,
        collision_radius,
        empowerment,
    );
    crystal.permanent = true;
    crystal.damage_mult = damage_mult;
    crystal.count_mult = count_mult;

    let mut entity_commands =
        spawn_crystal_entity(commands, assets, position, empowerment, crystal);
    entity_commands.insert(AutoCrystalTimer { timer: 0.0 });
    if resonance_cascade {
        entity_commands.insert(ResonanceCascade { absorptions: 0 });
    }
}

/// Shared helper that spawns the crystal mesh entity and range indicator.
/// Returns the entity commands for the crystal so callers can insert additional components.
fn spawn_crystal_entity<'a>(
    commands: &'a mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    mut crystal: ArcaneCrystal,
) -> EntityCommands<'a> {
    let height = CRYSTAL_HEIGHT * empowerment;
    let crystal_pos = Vec3::new(position.x, height / 2.0, position.z);
    let sphere_radius = height / 3.0;
    let range = crystal.range;

    crystal.position = crystal_pos;

    let mut entity_commands = commands.spawn((
        crystal,
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.arcane_crystal.clone()),
        Transform::from_translation(crystal_pos).with_scale(Vec3::new(
            0.7 * sphere_radius,
            1.5 * sphere_radius,
            0.7 * sphere_radius,
        )),
        CrystalTint::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::ArcaneCrystal,
        },
        OnGameplayScreen,
    ));

    let crystal_entity = entity_commands.id();

    // Spawn pink aura sphere as range indicator
    entity_commands.commands().spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(assets.crystal_aura_sphere.clone()),
        Transform::from_translation(Vec3::new(position.x, 0.0, position.z))
            .with_scale(Vec3::splat(range)),
        CrystalRangeIndicator { crystal_entity },
        OnGameplayScreen,
    ));

    entity_commands
}
