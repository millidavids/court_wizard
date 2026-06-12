use bevy::prelude::*;

use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion;
use crate::game::units::wizard::spells::squall::components::IceExplosion;
use crate::networking::snapshot::SpellEffectSnapshot;

use crate::game::multiplayer::components::OnMultiplayerGameScreen;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};

pub(crate) fn spawn_fireball_explosion(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
) -> Option<Entity> {
    let extra = effect.extra;
    let max_radius = extra[0];
    let empowerment = extra[1];
    let explosion_pos = Vec3::new(pos.x, pos.y.max(5.0), pos.z);
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(explosion_pos).with_scale(Vec3::splat(0.1)),
                FireballExplosion::new(
                    pos,
                    max_radius,
                    0.0,
                    crate::game::units::DamageType::Fire,
                    empowerment,
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_meteor_explosion(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
) -> Option<Entity> {
    let extra = effect.extra;
    let max_radius = extra[0];
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_scale(Vec3::splat(0.1)),
                MeteorExplosion::new(pos, max_radius, 0.0),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_ice_explosion(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
) -> Option<Entity> {
    let extra = effect.extra;
    let max_radius = extra[0];
    let empowerment = extra[1];
    let mat_handle = clone_sphere_material(sphere_materials, &assets.ice_explosion_sphere);
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_scale(Vec3::splat(0.1)),
                IceExplosion::new(pos, max_radius, 0.0, empowerment),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_dispel_impact(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::dispel::components::DispelImpact;
    let extra = effect.extra;
    let duration = extra[0];
    let expand_speed = extra[1];
    // Same mesh + material the host's DispelImpact uses, so the
    // remote peer sees the expanding nullification sphere. SP's
    // `update_dispel_impacts` is gated `is_spell_effects_active`
    // (both peers), so it ticks the growth + despawn on the ghost.
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(assets.guardian_aura_sphere.clone()),
                Transform::from_translation(pos).with_scale(Vec3::ZERO),
                DispelImpact {
                    time_alive: 0.0,
                    duration,
                    expand_speed,
                },
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_squall_storm(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::squall::components::{SquallStorm, SquallTalentParams};
    let extra = effect.extra;
    let flags = effect.flags;
    let radius = extra[0];
    let talent_params = SquallTalentParams {
        permafrost: flags & (1 << 0) != 0,
        hailstones: flags & (1 << 1) != 0,
        sleet_storm: flags & (1 << 2) != 0,
        absolute_zero: flags & (1 << 3) != 0,
        blizzard: flags & (1 << 4) != 0,
        ice_age: flags & (1 << 5) != 0,
        ..SquallTalentParams::default()
    };
    Some(
        commands
            .spawn((
                Transform::from_translation(pos),
                Visibility::Hidden,
                SquallStorm::new(pos, radius, 1.0, talent_params),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_scorched_earth_fire(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::fireball::components::ScorchedEarthFire;
    let extra = effect.extra;
    let max_radius = extra[0];
    let empowerment = extra[1];
    let duration = extra[2];
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
    let mut explosion = FireballExplosion::new(
        pos,
        max_radius,
        0.0,
        crate::game::units::DamageType::Fire,
        empowerment,
    );
    explosion.duration = duration;
    explosion.skip_growth = true;
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos).with_scale(Vec3::splat(max_radius.max(0.01))),
                explosion,
                ScorchedEarthFire,
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_napalm_trail(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::fireball::components::ScorchedEarthFire;
    let extra = effect.extra;
    let max_radius = extra[0];
    let empowerment = extra[1];
    let duration = extra[2];
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);
    let mut trail = FireballExplosion::new(
        pos,
        max_radius,
        0.0,
        crate::game::units::DamageType::Fire,
        empowerment,
    );
    trail.duration = duration;
    Some(
        commands
            .spawn((
                Mesh3d(assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos).with_scale(Vec3::splat(max_radius.max(0.01))),
                trail,
                // Same `ScorchedEarthFire` marker the SP path uses
                // — `spawn_scorched_earth_fire_smoke` is gated on
                // `any_exist::<ScorchedEarthFire>()`, so without
                // this the smoke VFX is missing for host-cast
                // napalm trails on the guest's screen.
                ScorchedEarthFire,
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_flame_ground_fire(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
) -> Option<Entity> {
    // Warglock flamethrower ground fire — an invisible `FireballExplosion`
    // plus the `FlameGroundFire` marker that drives the fire/smoke puffs via
    // `emit_flame_ground_fire_vfx` (which runs on both peers). The host owns
    // the gameplay (`GhostSpellEffect` keeps the guest from re-applying it).
    let ground_pos = Vec3::new(pos.x, 1.5, pos.z);
    let extra = effect.extra;
    let mut explosion = FireballExplosion::new(
        ground_pos,
        extra[0],
        0.0,
        crate::game::units::DamageType::Fire,
        1.0,
    );
    explosion.duration = extra[1];
    explosion.skip_growth = true;
    Some(
        commands
            .spawn((
                Transform::from_translation(ground_pos),
                explosion,
                crate::game::units::wizard::archetypes::gunslinger::FlameGroundFire,
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}
