use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants;
use super::super::resources::GunState;
use super::super::state::{aim_at_cursor, apply_cone_spread, apply_spread, gun_spawn_pos};
use super::hitscan::spawn_hitscan_ray;
use super::tracer::{spawn_muzzle_flash, spawn_tracer};
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
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
    spawn_muzzle_flash(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        GunType::MachineGun,
    );

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
    spawn_muzzle_flash(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        GunType::Magnum,
    );

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

    spawn_muzzle_flash(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        GunType::RocketLauncher,
    );

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

    spawn_muzzle_flash(
        &mut commands,
        &visual_assets,
        gun_spawn_pos(),
        GunType::Shotgun,
    );

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
