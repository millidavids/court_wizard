use bevy::prelude::*;

use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectSnapshot;

use crate::game::multiplayer::components::OnMultiplayerGameScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub(crate) fn spawn_wall_of_stone(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
) -> Option<Entity> {
    let extra = effect.extra;
    let half_length = extra[0];
    let half_width = extra[1];
    let height = extra[2];
    let duration = extra[3];
    let rotation = Quat::from_rotation_y(effect.rotation_y);
    // Reconstruct forward/right from rotation
    let forward = rotation * Vec3::X;
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    // Inserting `WallRising` makes the shared `animate_rising_walls`
    // system play the rise-from-the-ground animation here on the
    // remote peer too, instead of the wall just popping into
    // existence at full height. Spawn underground (`-height / 2.0`)
    // so the very first frame of the animator pulls the wall up
    // instead of yanking it from full height down to underground.
    Some(
        commands
            .spawn((
                Mesh3d(assets.unit_cuboid.clone()),
                MeshMaterial3d(assets.wall_of_stone.clone()),
                Transform::from_translation(Vec3::new(pos.x, -height / 2.0, pos.z))
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(half_length * 2.0, height, half_width * 2.0)),
                WallOfStone {
                    center: Vec3::new(pos.x, 0.0, pos.z),
                    half_length,
                    half_width,
                    forward,
                    right,
                    height,
                    time_alive: 0.0,
                    duration,
                    sinking: false,
                    empowerment: 1.0,
                    permanent: false,
                },
                crate::game::units::wizard::spells::wall_of_stone::components::WallRising::new(
                    crate::game::units::wizard::spells::wall_of_stone::constants::WALL_RISE_DURATION,
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_wall_of_fire(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireTalentParams;
    let extra = effect.extra;
    let flags = effect.flags;
    let half_width = extra[0];
    let duration = extra[1];
    let wall_length = extra[2];
    let talent_params = WallOfFireTalentParams {
        searing_heat: flags & (1 << 0) != 0,
        scorched_earth: flags & (1 << 1) != 0,
        spreading_flames: flags & (1 << 2) != 0,
        firestorm: flags & (1 << 3) != 0,
        twin_walls: flags & (1 << 4) != 0,
        consuming_inferno: flags & (1 << 5) != 0,
        ..WallOfFireTalentParams::default()
    };
    let material = materials.add(StandardMaterial {
        base_color: Color::NONE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let rotation = Quat::from_rotation_y(effect.rotation_y);
    let wall_height = 10.0;
    let wall_entity = commands
        .spawn((
            Mesh3d(assets.unit_cuboid.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(pos.x, wall_height / 2.0, pos.z))
                .with_rotation(rotation)
                .with_scale(Vec3::new(wall_length, wall_height, 60.0)),
            WallOfFireEffect::new(
                Vec3::ZERO,
                Vec3::ZERO,
                half_width,
                0.0,
                crate::game::units::DamageType::Fire,
                1.0,
                duration,
                talent_params,
            ),
            OnMultiplayerGameScreen,
        ))
        .id();

    // Ignition spark burst along the wall so the opposing client sees
    // fire kick up (matches the SP `spawn_wall_vfx` spark portion). The
    // mesh's length runs along local X, so the wall axis is `rot * X`.
    // (SP's looping crackle SFX is host-local and not replicated here.)
    let wall_axis = rotation * Vec3::X;
    let start = Vec3::new(pos.x, 3.0, pos.z) - wall_axis * (wall_length * 0.5);
    let spark_points = 4;
    let t_secs = start.x * 0.01;
    for j in 0..spark_points {
        let frac = (j as f32 + 0.5) / spark_points as f32;
        let spark_pos = start + wall_axis * (wall_length * frac);
        crate::game::units::wizard::spells::vfx::systems::spawn_fire_sparks(
            commands,
            assets,
            spark_pos,
            crate::game::units::wizard::spells::vfx::constants::SPARK_COUNT / 2,
            t_secs + j as f32,
        );
    }
    Some(wall_entity)
}
