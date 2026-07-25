use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use super::super::resources::{FlamethrowerSfx, GunState};
use super::super::state::{aim_at_cursor, apply_cone_spread, gun_spawn_pos};
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    Corpse, Health, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::utils::{get_cursor_world_position, local_player_team};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Marker on the burning-ground AOE left behind by flamethrower projectiles.
/// Drives `emit_flame_ground_fire_vfx` so the patch keeps billowing fire+
/// smoke for the duration of its damage tick.
#[derive(Component)]
pub struct FlameGroundFire;

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

/// Check flame particle collisions with enemies — apply tick damage. Staging
/// attackers (not yet activated at their rally point) are excluded.
#[allow(clippy::type_complexity)]
pub fn check_flame_collisions(
    mut commands: Commands,
    // Ghost flames (opponent's flamethrower, replicated for visuals) deal no
    // damage — the real damage already crosses via CRDT.
    flames: Query<(&Transform, &FlameParticle), Without<GhostFlameParticle>>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &crate::game::units::components::Team,
            Has<SpellShield>,
        ),
        (
            Without<FlameParticle>,
            Without<Corpse>,
            Without<StagingAttacker>,
        ),
    >,
    time: Res<Time>,
    mut damage_timer: Local<f32>,
    session: Option<Res<MultiplayerSession>>,
) {
    // Only apply damage every 0.1s to avoid frame-rate dependent DPS
    *damage_timer += time.delta_secs();
    if *damage_timer < 0.1 {
        return;
    }
    *damage_timer -= 0.1;

    let caster_team = local_player_team(session.as_deref());
    for (flame_transform, flame) in &flames {
        for (enemy_entity, enemy_transform, mut health, mut temp_hp, team, has_spell_shield) in
            &mut enemies
        {
            let distance = flame_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < flame.radius + 20.0 {
                // Enemy shielded King is immune; your own King takes the flames (friendly fire).
                if has_spell_shield && caster_team != *team {
                    continue;
                }
                apply_spell_damage_with_team(
                    &mut commands,
                    enemy_entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    flame.damage,
                    DamageType::Fire,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}

/// Despawn flame particles on ground contact or expiry, spawn burning ground AOE on impact.
pub fn despawn_expired_flames(
    mut commands: Commands,
    flames: Query<(Entity, &Transform, &FlameParticle, Has<GhostFlameParticle>)>,
) {
    for (entity, transform, flame, is_ghost) in &flames {
        let hit_ground = transform.translation.y <= 1.0;
        let expired = flame.time_alive >= flame.lifetime;

        if hit_ground || expired {
            // Spawn burning ground AOE on ground contact — but NOT for ghost
            // flames: the opponent receives the burning patch as a networked
            // spell effect, so spawning it here too would double it.
            if hit_ground && !is_ghost {
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
