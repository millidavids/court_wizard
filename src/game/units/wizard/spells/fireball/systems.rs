use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use super::styles::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{Health, Team, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;

/// Local wizard fireball casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_fireball_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Fireball {
        return;
    }

    let completed = fireball_casting_logic(
        &input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &visual_assets,
        &sfx,
        &game_config,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core fireball casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn fireball_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
) -> bool {
    let mut completed = false;

    // Check for release event
    if input.just_released {
        if caster_query.get(wizard_entity).is_ok() {
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return false;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST)
                    && let Some(target_pos) = input.cursor_pos
                {
                    let spawn_origin = SPELL_ORIGIN
                        + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);
                    spawn_fireball(commands, assets, spawn_origin, target_pos, primed_spell);
                    audio::play_sfx(commands, &sfx.fireball_cast, spawn_origin, game_config);
                    completed = true;
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                commands.entity(wizard_entity).insert(SpellCaster::new());
                casting_state.start_cast();
            }
        }
    }

    completed
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Spawns a fireball projectile from a PrimedSpell (wizard casting).
pub(crate) fn spawn_fireball(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    primed_spell: &PrimedSpell,
) {
    let direction = (target - origin).normalize();
    let speed = primed_spell.scale(constants::PROJECTILE_SPEED);
    let velocity = direction * speed;

    spawn_fireball_entity(
        commands,
        assets,
        origin,
        velocity,
        primed_spell.scale(constants::DAMAGE_PER_TICK),
        constants::DAMAGE_TYPE,
        primed_spell.scale(constants::EXPLOSION_RADIUS),
        primed_spell.scale(constants::PROJECTILE_COLLISION_RADIUS),
        primed_spell.empowerment,
        primed_spell.scale(FIREBALL_RADIUS),
    );
}

/// Spawns a raw fireball entity with explicit parameters.
///
/// Used by both wizard casting (via `spawn_fireball`) and crystal absorption.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fireball_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    velocity: Vec3,
    damage: f32,
    damage_type: DamageType,
    explosion_radius: f32,
    collision_radius: f32,
    empowerment: f32,
    visual_radius: f32,
) -> Entity {
    let entity = commands
        .spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_projectile.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(visual_radius)),
            Fireball::new(
                velocity,
                damage,
                damage_type,
                explosion_radius,
                collision_radius,
                empowerment,
            ),
            OnGameplayScreen,
        ))
        .id();

    // Spawn glow halo sibling
    vfx::systems::spawn_fire_glow(commands, assets, entity, origin, visual_radius);

    entity
}

/// Updates fireball projectile positions based on velocity.
pub fn move_fireballs(time: Res<Time>, mut fireballs: Query<(&mut Transform, &Fireball)>) {
    for (mut transform, fireball) in &mut fireballs {
        transform.translation += fireball.velocity * time.delta_secs();
    }
}

/// Spawns smoke trail wisps behind flying fireballs.
pub fn spawn_fireball_smoke_trail(
    mut commands: Commands,
    fireballs: Query<&Transform, With<Fireball>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < vfx::constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= vfx::constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for transform in fireballs.iter() {
        vfx::systems::spawn_fire_smoke_wisps(
            &mut commands,
            &visual_assets,
            transform.translation,
            vfx::constants::SMOKE_COUNT_PER_SPAWN,
            t,
            vfx::constants::SMOKE_LIFETIME,
            vfx::constants::SMOKE_SIZE,
            vfx::constants::SMOKE_RISE_SPEED,
            vfx::constants::SMOKE_SPREAD_SPEED,
        );

        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            transform.translation,
            1,
            t,
        );
    }
}

/// Checks for fireball collisions with units or the ground.
///
/// When a fireball hits a unit or the ground, it explodes.
pub fn check_fireball_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    fireballs: Query<(Entity, &Transform, &Fireball)>,
    targets: Query<(&Transform, &Team)>,
    walls: Query<&WallOfStone>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let t = time.elapsed_secs();

    for (fireball_entity, fireball_transform, fireball) in &fireballs {
        let fireball_pos = fireball_transform.translation;

        // Check collision with walls
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(fireball_pos) && fireball_pos.y <= wall.height {
                let explosion_pos = fireball_pos;
                spawn_explosion(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    fireball.explosion_radius,
                    fireball.damage,
                    fireball.empowerment,
                    t,
                    &sfx,
                    &game_config,
                );
                commands.entity(fireball_entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with ground (Y <= 0)
        if fireball_pos.y <= 0.0 {
            // Raise slightly above ground so cross-plane sphere isn't hidden
            let explosion_pos = Vec3::new(fireball_pos.x, 5.0, fireball_pos.z);
            spawn_explosion(
                &mut commands,
                &visual_assets,
                explosion_pos,
                fireball.explosion_radius,
                fireball.damage,
                fireball.empowerment,
                t,
                &sfx,
                &game_config,
            );
            commands.entity(fireball_entity).despawn();
            continue;
        }

        // Check collision with units
        for (target_transform, _team) in &targets {
            let distance = fireball_pos.distance(target_transform.translation);

            if distance < fireball.radius {
                spawn_explosion(
                    &mut commands,
                    &visual_assets,
                    fireball_pos,
                    fireball.explosion_radius,
                    fireball.damage,
                    fireball.empowerment,
                    t,
                    &sfx,
                    &game_config,
                );
                commands.entity(fireball_entity).despawn();
                break;
            }
        }
    }
}

/// Spawns a fireball explosion at the given position with sparks and smoke.
fn spawn_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
    time_secs: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
) {
    commands.spawn((
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.fireball_explosion.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
        FireballExplosion::new(
            position,
            max_radius,
            damage,
            constants::DAMAGE_TYPE,
            empowerment,
        ),
        NetworkedSpellEffect {
            kind: SpellEffectKind::FireballExplosion,
        },
        OnGameplayScreen,
    ));

    // Impact sparks
    vfx::systems::spawn_fire_sparks(
        commands,
        assets,
        position,
        vfx::constants::SPARK_COUNT,
        time_secs,
    );

    // Explosion smoke burst
    vfx::systems::spawn_explosion_smoke(commands, assets, position, time_secs);

    // Heat shimmer burst at impact
    vfx::systems::spawn_heat_shimmer(commands, assets, position, 3, time_secs);

    // Impact sound effect
    audio::play_sfx(commands, &sfx.fireball_impact, position, game_config);
}

/// Updates explosion visuals and timing.
pub fn update_explosions(
    time: Res<Time>,
    mut explosions: Query<(&mut FireballExplosion, &mut Transform)>,
) {
    for (mut explosion, mut transform) in &mut explosions {
        explosion.time_alive += time.delta_secs();
        explosion.time_since_last_tick += time.delta_secs();

        let current_radius = explosion.current_radius(constants::EXPLOSION_DURATION);
        transform.scale = Vec3::splat(current_radius);
    }
}

/// Applies damage to units hit by the explosion on a tick interval.
pub fn apply_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<&mut FireballExplosion>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
) {
    for mut explosion in &mut explosions {
        if explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL {
            explosion.time_since_last_tick = 0.0;

            let current_radius = explosion.current_radius(constants::EXPLOSION_DURATION);

            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
                let distance = explosion.origin.distance(transform.translation);

                if distance <= current_radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage_per_tick,
                        constants::DAMAGE_TYPE,
                        has_spell_shield,
                    );
                }
            }
        }
    }
}

/// Cleans up explosions that have finished animating.
pub fn cleanup_finished_explosions(
    mut commands: Commands,
    explosions: Query<(Entity, &FireballExplosion)>,
) {
    for (entity, explosion) in &explosions {
        if explosion.time_alive >= constants::EXPLOSION_DURATION {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawns fireballs that travel beyond the wizard's spell range.
pub fn despawn_distant_fireballs(
    mut commands: Commands,
    fireballs: Query<(Entity, &Transform), With<Fireball>>,
    wizard_query: Query<&Wizard, (With<LocalWizard>, Without<Fireball>)>,
) {
    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    let spell_range = wizard.spell_range;

    for (entity, transform) in &fireballs {
        let distance_from_wizard = transform.translation.distance(SPELL_ORIGIN);

        if distance_from_wizard > spell_range {
            commands.entity(entity).despawn();
        }
    }
}
