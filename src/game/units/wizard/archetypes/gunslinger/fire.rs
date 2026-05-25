//! Gun firing systems for each weapon type.

use super::state::{aim_at_cursor, apply_cone_spread, apply_spread, gun_spawn_pos};
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants;
use super::resources::{FlamethrowerSfx, GunState};
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fireball::systems::spawn_fireball_entity;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Initialize gun state when entering gameplay.
#[allow(clippy::too_many_arguments)]
pub fn fire_machine_gun(
    mouse: Res<ButtonInput<MouseButton>>,
    left_held: Res<crate::game::input::components::MouseLeftHeldThisFrame>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut gun_state: ResMut<GunState>,
    visual_assets: Res<SpellVisualAssets>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    // Fire on either a real left-mouse hold or a gamepad RT hold (the
    // gamepad layer writes `MouseLeftHeld` messages aggregated into
    // `MouseLeftHeldThisFrame`).
    let firing = mouse.pressed(MouseButton::Left) || left_held.held;
    if gun_state.selected_gun != GunType::MachineGun || !firing {
        return;
    }

    let ammo = gun_state.current_ammo_mut();
    if ammo.reloading || ammo.current == 0 || ammo.fire_cooldown > 0.0 {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some((dir, range)) = aim_at_cursor(cursor_pos) else {
        return;
    };

    let rng = &mut game_rng.0;
    let spread = rng.random_range(-constants::MACHINE_GUN_SPREAD..constants::MACHINE_GUN_SPREAD);
    let shot_dir = apply_spread(dir, spread);

    // Spawn hitscan ray for instant damage (from origin to cursor ground position)
    spawn_hitscan_ray(
        &mut commands,
        gun_spawn_pos(),
        shot_dir,
        range,
        constants::MACHINE_GUN_DAMAGE,
    );

    // Spawn visual tracer bullet
    spawn_tracer(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        shot_dir * constants::MACHINE_GUN_BULLET_SPEED,
        range,
    );
    spawn_muzzle_flash(&mut commands, &visual_assets, gun_spawn_pos());

    ammo.current -= 1;
    ammo.fire_cooldown = constants::MACHINE_GUN_FIRE_INTERVAL;

    audio::play_sfx(
        &mut commands,
        &sfx.machine_gun_shot,
        gun_spawn_pos(),
        &config,
        &sfx,
    );
}

/// Fire magnum — click to fire single high-damage hitscan bullet.
#[allow(clippy::too_many_arguments)]
pub fn fire_magnum(
    mouse: Res<ButtonInput<MouseButton>>,
    mut gamepad_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    mut gun_state: ResMut<GunState>,
    visual_assets: Res<SpellVisualAssets>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    let just_pressed =
        mouse.just_pressed(MouseButton::Left) || gamepad_pressed.read().next().is_some();
    if gun_state.selected_gun != GunType::Magnum || !just_pressed {
        return;
    }

    let ammo = gun_state.current_ammo_mut();
    if ammo.reloading || ammo.current == 0 || ammo.fire_cooldown > 0.0 {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some((dir, range)) = aim_at_cursor(cursor_pos) else {
        return;
    };

    spawn_hitscan_ray(
        &mut commands,
        gun_spawn_pos(),
        dir,
        range,
        constants::MAGNUM_DAMAGE,
    );

    spawn_tracer(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        dir * constants::MAGNUM_BULLET_SPEED,
        range,
    );
    spawn_muzzle_flash(&mut commands, &visual_assets, gun_spawn_pos());

    ammo.current -= 1;
    ammo.fire_cooldown = constants::MAGNUM_FIRE_INTERVAL;

    audio::play_sfx(
        &mut commands,
        &sfx.magnum_shot,
        gun_spawn_pos(),
        &config,
        &sfx,
    );
}

/// Fire rocket launcher — click to fire a fireball projectile that explodes on impact.
#[allow(clippy::too_many_arguments)]
pub fn fire_rocket(
    mouse: Res<ButtonInput<MouseButton>>,
    mut gamepad_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    mut gun_state: ResMut<GunState>,
    visual_assets: Res<SpellVisualAssets>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    let just_pressed =
        mouse.just_pressed(MouseButton::Left) || gamepad_pressed.read().next().is_some();
    if gun_state.selected_gun != GunType::RocketLauncher || !just_pressed {
        return;
    }

    let ammo = gun_state.current_ammo_mut();
    if ammo.reloading || ammo.current == 0 || ammo.fire_cooldown > 0.0 {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some(target) = cursor_pos else { return };

    let direction = (target - gun_spawn_pos()).normalize();
    let velocity = direction * constants::ROCKET_SPEED;

    // Use Fireball's own explosion radius so the rocket's detonation looks
    // identical to a Fireball cast (same bubble count, same blast size).
    spawn_fireball_entity(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        velocity,
        constants::ROCKET_DAMAGE,
        DamageType::Fire,
        crate::game::units::wizard::spells::fireball::constants::EXPLOSION_RADIUS,
        constants::ROCKET_RADIUS,
        1.0,
        constants::ROCKET_RADIUS * 2.0,
    );

    spawn_muzzle_flash(&mut commands, &visual_assets, gun_spawn_pos());

    ammo.current -= 1;
    ammo.fire_cooldown = constants::ROCKET_FIRE_INTERVAL;

    audio::play_sfx(
        &mut commands,
        &sfx.rocket_launcher_shot,
        gun_spawn_pos(),
        &config,
        &sfx,
    );
}

/// Fire shotgun — click to fire 30 hitscan pellets in a cone.
#[allow(clippy::too_many_arguments)]
pub fn fire_shotgun(
    mouse: Res<ButtonInput<MouseButton>>,
    mut gamepad_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut gun_state: ResMut<GunState>,
    visual_assets: Res<SpellVisualAssets>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    let just_pressed =
        mouse.just_pressed(MouseButton::Left) || gamepad_pressed.read().next().is_some();
    if gun_state.selected_gun != GunType::Shotgun || !just_pressed {
        return;
    }

    let ammo = gun_state.current_ammo_mut();
    if ammo.reloading || ammo.current == 0 || ammo.fire_cooldown > 0.0 {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some((dir, range)) = aim_at_cursor(cursor_pos) else {
        return;
    };

    let rng = &mut game_rng.0;

    for _ in 0..constants::SHOTGUN_PELLET_COUNT {
        let pellet_dir = apply_cone_spread(dir, constants::SHOTGUN_SPREAD, rng);

        spawn_hitscan_ray(
            &mut commands,
            gun_spawn_pos(),
            pellet_dir,
            range,
            constants::SHOTGUN_PELLET_DAMAGE,
        );

        spawn_tracer(
            &mut commands,
            &visual_assets,
            gun_spawn_pos(),
            pellet_dir * constants::SHOTGUN_BULLET_SPEED,
            range,
        );
    }

    spawn_muzzle_flash(&mut commands, &visual_assets, gun_spawn_pos());

    ammo.current -= 1;
    ammo.fire_cooldown = constants::SHOTGUN_FIRE_INTERVAL;

    audio::play_sfx(
        &mut commands,
        &sfx.shotgun_shot,
        gun_spawn_pos(),
        &config,
        &sfx,
    );
}

/// Fire flamethrower — hold mouse to spray flame projectiles toward cursor with gravity arc.
#[allow(clippy::too_many_arguments)]
pub fn fire_flamethrower(
    mouse: Res<ButtonInput<MouseButton>>,
    left_held: Res<crate::game::input::components::MouseLeftHeldThisFrame>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut gun_state: ResMut<GunState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    mut flamethrower_sfx: ResMut<FlamethrowerSfx>,
) {
    let is_firing = gun_state.selected_gun == GunType::Flamethrower
        && (mouse.pressed(MouseButton::Left) || left_held.held)
        && !gun_state.current_ammo().reloading
        && gun_state.current_ammo().current > 0;

    // Manage looping flamethrower sound (spawn directly to avoid ChannelingSfx cleanup by disintegrate)
    if is_firing && flamethrower_sfx.entity.is_none() {
        let volume = config.effective_sfx_volume();
        let entity = commands
            .spawn((
                AudioPlayer::new(sfx.disintegrate_channel.clone()),
                PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume)),
                OnGameplayScreen,
            ))
            .id();
        flamethrower_sfx.entity = Some(entity);
    } else if !is_firing && let Some(entity) = flamethrower_sfx.entity.take() {
        commands.entity(entity).try_despawn();
    }

    if !is_firing {
        return;
    }

    let ammo = gun_state.current_ammo_mut();
    if ammo.fire_cooldown > 0.0 {
        return;
    }

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some((dir, _range)) = aim_at_cursor(cursor_pos) else {
        return;
    };

    // Aim toward cursor with cone spread, gravity will arc it down
    let rng = &mut game_rng.0;
    let spread_dir = apply_cone_spread(dir, constants::FLAMETHROWER_SPREAD, rng);
    let velocity = spread_dir * constants::FLAMETHROWER_SPEED;

    // Visuals: no mesh — `emit_flame_particle_vfx` spawns fire+smoke puffs
    // along the flame's path each frame so the projectile reads as a
    // billowing flame rather than a single sphere sprite.
    commands.spawn((
        Transform::from_translation(gun_spawn_pos()),
        FlameParticle {
            velocity,
            damage: constants::FLAMETHROWER_DAMAGE,
            lifetime: constants::FLAMETHROWER_PARTICLE_LIFETIME,
            time_alive: 0.0,
            radius: constants::FLAME_PARTICLE_SIZE,
        },
        OnGameplayScreen,
    ));

    ammo.current -= 1;
    ammo.fire_cooldown = constants::FLAMETHROWER_FIRE_INTERVAL;
}

// ===== Hitscan collision system =====

/// Process all hitscan rays — find closest enemy along each ray within cylinder radius,
/// apply damage, then despawn the ray.
pub fn check_hitscan_collisions(
    mut commands: Commands,
    rays: Query<(Entity, &HitscanRay)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
            Has<SpellShield>,
        ),
        Without<Corpse>,
    >,
) {
    for (ray_entity, ray) in &rays {
        let mut closest_hit: Option<(Entity, f32)> = None;

        for (enemy_entity, enemy_transform, _health, _temp_hp, _team, has_spell_shield) in &enemies
        {
            if has_spell_shield {
                continue;
            }

            // Point-to-line-segment distance test (cylinder collision)
            let to_enemy = enemy_transform.translation - ray.origin;
            let t = to_enemy.dot(ray.direction).clamp(0.0, ray.max_range);
            let closest_point = ray.origin + ray.direction * t;
            let distance = closest_point.distance(enemy_transform.translation);

            if distance < ray.cylinder_radius {
                // Track closest hit along the ray
                if closest_hit.is_none_or(|(_, prev_t)| t < prev_t) {
                    closest_hit = Some((enemy_entity, t));
                }
            }
        }

        // Apply damage to the closest hit
        if let Some((hit_entity, _)) = closest_hit
            && let Ok((_entity, _transform, mut health, mut temp_hp, _team, _shield)) =
                enemies.get_mut(hit_entity)
        {
            apply_spell_damage(
                &mut commands,
                hit_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                ray.damage,
                DamageType::Force,
                false,
            );
            commands
                .entity(hit_entity)
                .insert(crate::game::units::hit_flash::HitFlash {
                    timer: constants::BULLET_HIT_FLASH_DURATION,
                });
        }

        // Always despawn the ray after processing
        commands.entity(ray_entity).try_despawn();
    }
}

// ===== Tracer movement and cleanup =====

/// Move bullet tracers in a straight line.
pub fn move_tracers(time: Res<Time>, mut tracers: Query<(&mut Transform, &BulletTracer)>) {
    for (mut transform, tracer) in &mut tracers {
        transform.translation += tracer.velocity * time.delta_secs();
    }
}

/// Despawn tracers that have traveled beyond their range.
pub fn despawn_distant_tracers(
    mut commands: Commands,
    tracers: Query<(Entity, &Transform, &BulletTracer)>,
) {
    for (entity, transform, tracer) in &tracers {
        if transform.translation.distance(tracer.origin) > tracer.max_range {
            commands.entity(entity).try_despawn();
        }
    }
}

// ===== Flame particle systems =====

/// Update flame particles — move with gravity arc and age.
pub fn update_flame_particles(
    time: Res<Time>,
    mut flames: Query<(&mut Transform, &mut FlameParticle)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut flame) in &mut flames {
        flame.time_alive += dt;
        // Apply gravity to pull flames downward in an arc
        flame.velocity.y -= constants::FLAMETHROWER_GRAVITY * dt;
        transform.translation += flame.velocity * dt;
        // Grow from start size to max size over lifetime
        let t = (flame.time_alive / flame.lifetime).min(1.0);
        let scale = constants::FLAME_PARTICLE_START_SIZE
            + t * (constants::FLAME_PARTICLE_SIZE - constants::FLAME_PARTICLE_START_SIZE);
        transform.scale = Vec3::splat(scale);
    }
}

/// Emits fire+smoke puffs at each flame projectile's current position so the
/// flamethrower reads as a billowing stream of flame instead of a single
/// sprite. Throttled by a small interval to avoid swamping the particle
/// system; one puff per particle per emit tick.
pub fn emit_flame_particle_vfx(
    mut commands: Commands,
    time: Res<Time>,
    flames: Query<(&Transform, &FlameParticle)>,
    visual_assets: Res<SpellVisualAssets>,
    mut emit_timer: Local<f32>,
) {
    *emit_timer += time.delta_secs();
    if *emit_timer < 0.04 {
        return;
    }
    *emit_timer = 0.0;

    let t = time.elapsed_secs();
    for (transform, flame) in &flames {
        // Tighter half-width than the gameplay radius so puffs cluster around
        // the flame core rather than spraying outward.
        let half_width = flame.radius * 0.4;
        crate::game::units::wizard::spells::vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            transform.translation,
            half_width,
            1,
            t + transform.translation.x * 0.013,
        );
    }
}

/// Marker on the burning-ground AOE left behind by flamethrower projectiles.
/// Drives `emit_flame_ground_fire_vfx` so the patch keeps billowing fire+
/// smoke for the duration of its damage tick.
#[derive(Component)]
pub struct FlameGroundFire;

/// Periodically spawns fire+smoke at each burning-ground patch. Mirrors the
/// emission used by Wall of Fire / Grease so the visual is consistent.
pub fn emit_flame_ground_fire_vfx(
    mut commands: Commands,
    time: Res<Time>,
    patches: Query<(&Transform, &FireballExplosion), With<FlameGroundFire>>,
    visual_assets: Res<SpellVisualAssets>,
    mut emit_timer: Local<f32>,
) {
    *emit_timer += time.delta_secs();
    if *emit_timer < 0.12 {
        return;
    }
    *emit_timer = 0.0;

    let t = time.elapsed_secs();
    for (transform, explosion) in &patches {
        // Stop emitting if the patch is about to despawn so we don't leave
        // dangling puffs.
        if explosion.duration - explosion.time_alive < 0.25 {
            continue;
        }
        let pos = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
        crate::game::units::wizard::spells::vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            pos,
            explosion.max_radius * 0.6,
            2,
            t + pos.x * 0.017,
        );
    }
}

/// Check flame particle collisions with enemies — apply tick damage.
pub fn check_flame_collisions(
    mut commands: Commands,
    flames: Query<(&Transform, &FlameParticle)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
            Has<SpellShield>,
        ),
        (Without<FlameParticle>, Without<Corpse>),
    >,
    time: Res<Time>,
    mut damage_timer: Local<f32>,
) {
    // Only apply damage every 0.1s to avoid frame-rate dependent DPS
    *damage_timer += time.delta_secs();
    if *damage_timer < 0.1 {
        return;
    }
    *damage_timer -= 0.1;

    for (flame_transform, flame) in &flames {
        for (enemy_entity, enemy_transform, mut health, mut temp_hp, _team, has_spell_shield) in
            &mut enemies
        {
            let distance = flame_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < flame.radius + 20.0 {
                if has_spell_shield {
                    continue;
                }
                apply_spell_damage(
                    &mut commands,
                    enemy_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    flame.damage,
                    DamageType::Fire,
                    false,
                );
            }
        }
    }
}

/// Despawn flame particles on ground contact or expiry, spawn burning ground AOE on impact.
pub fn despawn_expired_flames(
    mut commands: Commands,
    flames: Query<(Entity, &Transform, &FlameParticle)>,
) {
    for (entity, transform, flame) in &flames {
        let hit_ground = transform.translation.y <= 1.0;
        let expired = flame.time_alive >= flame.lifetime;

        if hit_ground || expired {
            // Spawn burning ground AOE on ground contact
            if hit_ground {
                let pos = Vec3::new(transform.translation.x, 1.5, transform.translation.z);
                let mut ground_fire = FireballExplosion::new(
                    pos,
                    constants::BURNING_GROUND_RADIUS,
                    constants::BURNING_GROUND_DAMAGE,
                    DamageType::Fire,
                    1.0,
                );
                ground_fire.duration = constants::BURNING_GROUND_DURATION;
                ground_fire.skip_growth = true;

                // Visuals come from `emit_flame_ground_fire_vfx` (fire+smoke
                // puffs) — the FireballExplosion entity itself stays
                // invisible and only carries the gameplay logic + marker.
                commands.spawn((
                    Transform::from_translation(pos),
                    ground_fire,
                    FlameGroundFire,
                    OnGameplayScreen,
                ));
            }

            commands.entity(entity).try_despawn();
        }
    }
}

// ===== VFX =====

/// Update and despawn muzzle flashes.
pub fn update_muzzle_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut MuzzleFlash, &mut Transform)>,
) {
    for (entity, mut flash, mut transform) in &mut flashes {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let scale =
                (flash.timer / constants::MUZZLE_FLASH_DURATION) * constants::MUZZLE_FLASH_SIZE;
            transform.scale = Vec3::splat(scale);
        }
    }
}

// ===== Helper functions =====

fn spawn_hitscan_ray(
    commands: &mut Commands,
    origin: Vec3,
    direction: Vec3,
    max_range: f32,
    damage: f32,
) {
    commands.spawn((
        HitscanRay {
            origin,
            direction,
            max_range,
            cylinder_radius: constants::HITSCAN_CYLINDER_RADIUS,
            damage,
        },
        OnGameplayScreen,
    ));
}

fn spawn_tracer(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    velocity: Vec3,
    max_range: f32,
) {
    // Orient the cylinder along the velocity direction for a bullet-line look
    let dir = velocity.normalize_or_zero();
    let rotation = Quat::from_rotation_arc(Vec3::Y, dir);
    commands.spawn((
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.bullet_tracer.clone()),
        Transform::from_translation(position)
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                constants::BULLET_RADIUS,
                constants::BULLET_LENGTH,
                constants::BULLET_RADIUS,
            )),
        BulletTracer {
            velocity,
            max_range,
            origin: position,
        },
        OnGameplayScreen,
    ));
}

fn spawn_muzzle_flash(commands: &mut Commands, assets: &SpellVisualAssets, position: Vec3) {
    commands.spawn((
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.fireball_projectile.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(constants::MUZZLE_FLASH_SIZE)),
        MuzzleFlash {
            timer: constants::MUZZLE_FLASH_DURATION,
        },
        OnGameplayScreen,
    ));
}
