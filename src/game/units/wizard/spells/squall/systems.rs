//! Squall spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::components::{IceExplosion, IceProjectile, SquallCircleIndicator, SquallStorm};
use super::constants::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;

/// Gets cursor position projected onto Y=0 plane.
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

    // Intersect ray with Y=0 plane
    if ray.direction.y.abs() < 0.0001 {
        return None; // Ray is parallel to plane
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // Intersection is behind camera
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a position to be within the wizard's spell range.
fn clamp_to_spell_range(
    target: Vec3,
    wizard_pos: Vec3,
    spell_range: f32,
    storm_radius: f32,
) -> Vec3 {
    let wizard_height = wizard_pos.y;

    // Calculate max ground radius using Pythagorean theorem
    let max_ground_radius = if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    // Account for storm radius so entire circle stays within range
    let max_center_distance = (max_ground_radius - storm_radius).max(0.0);

    // Calculate XZ plane distance from wizard to cursor
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        // Clamp to ensure entire circle stays within spell range
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

/// Spawns the visual circle indicator during casting.
fn spawn_circle_indicator(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = STORM_RADIUS * empowerment;

    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.squall_indicator.clone()),
            Transform::from_translation(Vec3::new(position.x, CIRCLE_Y_POSITION, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(radius)),
            SquallCircleIndicator::new(position, empowerment),
            OnGameplayScreen,
        ))
        .id()
}

/// Local wizard squall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_squall_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SquallCircleIndicator>,
    existing_storms: Query<Entity, With<SquallStorm>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Squall {
        return;
    }

    let completed = squall_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard_transform,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &existing_storms,
        &mut commands,
        &visual_assets,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core squall casting logic.
#[allow(clippy::too_many_arguments)]
fn squall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SquallCircleIndicator>,
    existing_storms: &Query<Entity, With<SquallStorm>>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
) -> bool {
    let mut completed = false;

    // Check for release event - cancel cast
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return false;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = wizard_transform.translation;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale;

    cursor_world_pos = clamp_to_spell_range(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    commands,
                    assets,
                    cursor_world_pos,
                    primed_spell.empowerment,
                );
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    // Despawn any existing storms (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).despawn();
                    }

                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            commands.spawn((
                                SquallStorm::new(
                                    indicator.position,
                                    storm_radius,
                                    primed_spell.empowerment,
                                ),
                                ConcentrationSpell {
                                    spell_name: "Squall",
                                },
                                OnGameplayScreen,
                            ));
                        }
                        commands.entity(indicator_entity).despawn();
                    }

                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }

    completed
}

/// Updates circle indicator visuals during casting.
///
/// Applies pulse animation and updates position.
pub(super) fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut SquallCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        // Update time alive for pulse animation
        indicator.time_alive += time.delta_secs();

        // Apply pulse scale
        let radius = STORM_RADIUS * indicator.empowerment;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);

        // Update position
        transform.translation.x = indicator.position.x;
        transform.translation.y = CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Spawns ice projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_ice_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<&mut SquallStorm>,
) {
    let mut rng = rand::thread_rng();

    for mut storm in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if it's time to spawn another projectile
        if storm.time_since_spawn >= ICE_SPAWN_INTERVAL {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = rng.gen_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                ICE_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Spawn projectile
            let damage = FROST_DAMAGE * storm.empowerment;
            let explosion_radius = EXPLOSION_RADIUS * storm.empowerment;

            commands.spawn((
                IceProjectile::new(
                    Vec3::new(0.0, ICE_INITIAL_VELOCITY, 0.0),
                    damage,
                    explosion_radius,
                    ICE_PROJECTILE_RADIUS,
                    storm.empowerment,
                ),
                Mesh3d(visual_assets.unit_sphere.clone()),
                MeshMaterial3d(visual_assets.ice_projectile.clone()),
                Transform::from_translation(spawn_pos)
                    .with_scale(Vec3::splat(ICE_PROJECTILE_MESH_RADIUS)),
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates ice projectile physics - applies gravity and moves projectiles.
pub(super) fn update_ice_projectiles(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &mut IceProjectile)>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut projectile) in projectiles.iter_mut() {
        // Apply gravity
        projectile.velocity.y += ICE_GRAVITY * delta;

        // Move projectile
        transform.translation += projectile.velocity * delta;
    }
}

/// Checks for ice projectile collisions with ground or walls, spawns explosions.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_ice_projectile_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    projectiles: Query<(Entity, &Transform, &IceProjectile)>,
    walls: Query<&WallOfStone>,
) {
    for (entity, transform, projectile) in projectiles.iter() {
        let projectile_pos = transform.translation;

        // Check wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(projectile_pos) && projectile_pos.y <= wall.height {
                // Hit wall - spawn explosion at wall surface
                let explosion_pos = Vec3::new(projectile_pos.x, wall.height, projectile_pos.z);
                spawn_ice_explosion(
                    &mut commands,
                    &visual_assets,
                    explosion_pos,
                    projectile.explosion_radius,
                    projectile.damage,
                    projectile.empowerment,
                );
                commands.entity(entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check ground collision (Y <= 0)
        if projectile_pos.y <= 0.0 {
            // Hit ground - spawn explosion at ground level
            let explosion_pos = Vec3::new(projectile_pos.x, 0.0, projectile_pos.z);
            spawn_ice_explosion(
                &mut commands,
                &visual_assets,
                explosion_pos,
                projectile.explosion_radius,
                projectile.damage,
                projectile.empowerment,
            );
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns an ice explosion at the given position.
fn spawn_ice_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    max_radius: f32,
    damage: f32,
    empowerment: f32,
) {
    // Position slightly above battlefield (y=1) to avoid z-fighting
    let explosion_pos = Vec3::new(position.x, 1.0, position.z);

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(assets.ice_explosion.clone()),
        Transform::from_translation(explosion_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.1)),
        IceExplosion::new(position, max_radius, damage, empowerment),
        NetworkedSpellEffect {
            kind: SpellEffectKind::IceExplosion,
        },
        OnGameplayScreen,
    ));
}

/// Updates explosion visuals and applies damage.
pub(super) fn update_ice_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut explosions: Query<(Entity, &mut IceExplosion, &mut Transform)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<IceExplosion>,
    >,
) {
    for (explosion_entity, mut explosion, mut transform) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;

            for (unit_entity, unit_transform, mut health, mut temp_hp, has_spell_shield) in
                units.iter_mut()
            {
                let distance = unit_transform.translation.distance(explosion.origin);

                if distance <= explosion.max_radius {
                    apply_spell_damage(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        DamageType::Frost,
                        has_spell_shield,
                    );
                }
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).despawn();
        }
    }
}
