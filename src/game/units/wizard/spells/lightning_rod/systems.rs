//! Lightning Rod spell systems.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{
    LightningRod, LightningRodArc, LightningRodCircleIndicator, LightningStrike,
};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{Corpse, Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Local wizard Lightning Rod casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_lightning_rod_casting(
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
    mut indicator_query: Query<&mut LightningRodCircleIndicator>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::LightningRod {
        return;
    }

    let wizard_pos = wizard_transform.translation;
    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, wizard_pos, wizard.spell_range));

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(MANA_COST)
        && let Some(pos) = clamped_pos
    {
        let circle_entity =
            spawn_circle_indicator(&mut commands, &visual_assets, pos, primed_spell.empowerment);
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update indicator position during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(pos) = clamped_pos
        && let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
    {
        indicator.position = pos;
    }

    // Get the final spawn position from indicator if available
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    // Override cursor_pos with indicator position for shared logic
    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let completed = lightning_rod_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        wizard_transform,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &visual_assets,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core Lightning Rod casting logic.
#[allow(clippy::too_many_arguments)]
fn lightning_rod_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    _wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
) -> bool {
    let wizard_pos = wizard_transform.translation;

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

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
                && input.cursor_pos.is_some()
            {
                // SpellCaster insertion handled by the wrapper
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    let spawn_pos = input.cursor_pos.unwrap_or(wizard_pos);

                    spawn_lightning_rod(commands, assets, spawn_pos, primed_spell.empowerment);
                    completed = true;
                }

                // Clean up indicator and caster
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
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

    if ray.direction.y.abs() < 0.0001 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a position to be within the wizard's spell range.
fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let wizard_height = wizard_pos.y;

    let max_ground_radius = if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_ground_radius && distance > 0.001 {
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_ground_radius
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
    let radius = ARC_RADIUS * empowerment;

    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.lightning_rod_indicator.clone()),
            Transform::from_translation(Vec3::new(position.x, CIRCLE_Y_POSITION, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(radius)),
            LightningRodCircleIndicator::new(position, empowerment),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns the lightning rod tower entity.
pub(crate) fn spawn_lightning_rod(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
) {
    let tower_height = TOWER_HEIGHT;
    let tower_radius = TOWER_RADIUS;

    // Cylinder sits centered, so position at half height
    let spawn_pos = Vec3::new(position.x, tower_height / 2.0, position.z);

    // unit_cylinder is Cylinder::new(0.5, 1.0), scale to tower dimensions
    // radius scale = tower_radius / 0.5, height scale = tower_height / 1.0
    let radius_scale = tower_radius / 0.5;

    commands.spawn((
        LightningRod::new(
            Vec3::new(position.x, 0.0, position.z),
            TOWER_DURATION * empowerment,
            empowerment,
        ),
        Mesh3d(assets.unit_cylinder.clone()),
        MeshMaterial3d(assets.lightning_rod.clone()),
        Transform::from_translation(spawn_pos).with_scale(Vec3::new(
            radius_scale,
            tower_height,
            radius_scale,
        )),
        NetworkedSpellEffect {
            kind: SpellEffectKind::LightningRod,
        },
        OnGameplayScreen,
    ));
}

/// Updates circle indicator visuals during casting.
pub(super) fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut LightningRodCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();

        let radius = ARC_RADIUS * indicator.empowerment;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);

        transform.translation.x = indicator.position.x;
        transform.translation.y = CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Updates lightning rod timers, spawns lightning strikes, and despawns expired rods.
pub(super) fn update_lightning_rod(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut rods: Query<(Entity, &mut LightningRod)>,
) {
    let delta = time.delta_secs();

    for (entity, mut rod) in rods.iter_mut() {
        rod.time_alive += delta;
        rod.time_since_strike += delta;

        // Despawn if expired
        if rod.is_expired() {
            commands.entity(entity).despawn();
            continue;
        }

        // Spawn lightning strike on interval
        if rod.time_since_strike >= STRIKE_INTERVAL {
            rod.time_since_strike = 0.0;

            let strike_start = Vec3::new(rod.position.x, STRIKE_SPAWN_HEIGHT, rod.position.z);

            let target_pos = Vec3::new(rod.position.x, TOWER_HEIGHT, rod.position.z);

            let bolt_length = STRIKE_SPAWN_HEIGHT - TOWER_HEIGHT;
            let bolt_width = STRIKE_BOLT_WIDTH * rod.empowerment;

            let midpoint = (strike_start + target_pos) / 2.0;

            commands.spawn((
                LightningStrike {
                    target_pos,
                    speed: STRIKE_SPEED,
                    arc_damage: ARC_DAMAGE * rod.empowerment,
                    arc_radius: ARC_RADIUS * rod.empowerment,
                    empowerment: rod.empowerment,
                },
                Mesh3d(visual_assets.unit_rect.clone()),
                MeshMaterial3d(visual_assets.lightning_strike.clone()),
                Transform::from_translation(midpoint).with_scale(Vec3::new(
                    bolt_width,
                    bolt_length,
                    bolt_width,
                )),
                OnGameplayScreen,
            ));
        }
    }
}

/// Moves lightning strikes downward and triggers arcs when they hit the rod.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_lightning_strikes(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut strikes: Query<(Entity, &mut Transform, &LightningStrike)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, strike) in strikes.iter_mut() {
        // Move downward
        transform.translation.y -= strike.speed * delta;

        // Check if bolt has reached the rod
        if transform.translation.y <= strike.target_pos.y {
            // Spawn arcs to nearby units
            spawn_arcs_to_nearby_units(
                &mut commands,
                &visual_assets,
                strike.target_pos,
                strike.arc_damage,
                strike.arc_radius,
                strike.empowerment,
                &mut units,
            );

            // Despawn the strike bolt
            commands.entity(entity).despawn();
        }
    }
}

/// Finds nearby units and spawns lightning arcs from the rod to each target.
#[allow(clippy::too_many_arguments)]
fn spawn_arcs_to_nearby_units(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    rod_top: Vec3,
    damage: f32,
    radius: f32,
    empowerment: f32,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
) {
    // Collect targets sorted by distance (closest first)
    let mut targets: Vec<(Entity, Vec3, f32)> = units
        .iter()
        .map(|(entity, transform, _, _, _)| {
            let pos = transform.translation;
            let dist = Vec3::new(rod_top.x, 0.0, rod_top.z).distance(Vec3::new(pos.x, 0.0, pos.z));
            (entity, pos, dist)
        })
        .filter(|(_, _, dist)| *dist <= radius)
        .collect();

    targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    targets.truncate(ARC_MAX_TARGETS);

    // Apply damage and spawn arc visuals
    for (target_entity, target_pos, _) in &targets {
        if let Ok((_, _, mut health, mut temp_hp, has_spell_shield)) = units.get_mut(*target_entity)
        {
            apply_spell_damage(
                commands,
                *target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Electric,
                has_spell_shield,
            );
        }

        // Spawn arc visual
        spawn_arc(commands, assets, rod_top, *target_pos, empowerment);
    }
}

/// Spawns a lightning arc visual between two points.
fn spawn_arc(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    start: Vec3,
    end: Vec3,
    empowerment: f32,
) {
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    let length = start.distance(end);

    let arc_width = ARC_WIDTH * empowerment;

    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    commands.spawn((
        LightningRodArc::new(ARC_LIFETIME),
        Mesh3d(assets.unit_rect.clone()),
        MeshMaterial3d(assets.lightning_rod_arc.clone()),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(arc_width, length, arc_width)),
        OnGameplayScreen,
    ));
}

/// Updates lightning arc visuals with pulsing animation and despawns expired arcs.
pub(super) fn update_lightning_rod_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut arcs: Query<(
        Entity,
        &mut LightningRodArc,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut arc, material_handle) in &mut arcs {
        arc.time_alive += time.delta_secs();
        arc.lifetime -= time.delta_secs();

        // Despawn expired arcs
        if arc.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Pulsing intensity animation
        let intensity = 0.7 + 0.3 * (arc.time_alive * 20.0).sin();

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let base = ARC_COLOR;
            material.base_color = Color::srgba(
                base.to_srgba().red * intensity,
                base.to_srgba().green * intensity,
                base.to_srgba().blue * intensity,
                base.to_srgba().alpha,
            );
        }
    }
}
