use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput};
use super::components::PhantasmalForceIndicator;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::{Health, Hitbox, IllusionDecoy, PermanentCorpse, Team};
use crate::game::units::infantry::resources::InfantryAssets;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned).
    completed: bool,
}

/// Local wizard phantasmal force casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_phantasmal_force_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    mut indicator_query: Query<&mut PhantasmalForceIndicator>,
    infantry_assets: Res<InfantryAssets>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::PhantasmalForce { return; }

    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                if let Some(pos) = clamped_cursor {
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        pos,
                        primed_spell.empowerment,
                    );
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                }
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor {
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                    && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
                {
                    indicator.position = pos;
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
        }
    }

    let cast_result = phantasmal_force_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
    );

    if cast_result.completed {
        // Spawn decoys using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                spawn_decoys(
                    &mut commands,
                    indicator.position,
                    primed_spell.empowerment,
                    &infantry_assets,
                );
            }
            commands.entity(indicator_entity).despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Guest wizard phantasmal force casting -- reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_phantasmal_force_casting_guest(
    time: Res<Time>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<GuestWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    infantry_assets: Res<InfantryAssets>,
    guest_cursor: Res<GuestCursorPosition>,
    guest_input: Res<GuestInputState>,
) {
    let input = WizardInput {
        just_pressed: guest_input.just_pressed,
        pressed: guest_input.pressed,
        just_released: guest_input.just_released,
        cursor_pos: guest_cursor.position,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::PhantasmalForce { return; }

    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    // Handle release
    if input.just_released {
        if caster_query.get(wizard_entity).is_ok() {
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage SpellCaster for guest (no indicator)
    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                commands.entity(wizard_entity).insert(SpellCaster::new());
            }
        }
        CastingState::Channeling { .. } => {
            if caster_query.get(wizard_entity).is_ok() {
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
        _ => {}
    }

    let cast_result = phantasmal_force_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
    );

    if cast_result.completed {
        // Spawn decoys at cursor position for guest
        if let Some(pos) = clamped_cursor {
            spawn_decoys(
                &mut commands,
                pos,
                primed_spell.empowerment,
                &infantry_assets,
            );
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
    }
}

/// Core phantasmal force casting logic -- called by both local and guest systems.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn phantasmal_force_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> CastResult {
    let mut result = CastResult { completed: false };

    if input.just_released {
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    result.completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                casting_state.start_cast();
            }
        }
    }

    result
}

/// Clamps cursor position to spell range accounting for circle radius.
fn clamp_cursor_to_range(
    cursor_pos: Option<Vec3>,
    wizard_transform: &Transform,
    wizard: &Wizard,
    primed_spell: &PrimedSpell,
) -> Option<Vec3> {
    let mut pos = cursor_pos?;

    let wizard_pos = wizard_transform.translation;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        pos = wizard_pos + normalized_direction * max_center_distance;
    }

    Some(pos)
}

pub fn update_phantasmal_force_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut PhantasmalForceIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Ticks illusion decoy timers and despawns expired ones.
pub fn tick_illusion_decoys(
    mut commands: Commands,
    time: Res<Time>,
    mut decoys: Query<(Entity, &mut IllusionDecoy)>,
) {
    let delta = time.delta_secs();
    for (entity, mut decoy) in &mut decoys {
        if decoy.update(delta) {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_decoys(
    commands: &mut Commands,
    position: Vec3,
    empowerment: f32,
    infantry_assets: &InfantryAssets,
) {
    let duration = constants::DECOY_DURATION * empowerment;
    let spread = constants::DECOY_SPREAD;

    // Spawn decoys in a triangle pattern
    let offsets = [
        Vec3::new(0.0, 0.0, -spread),
        Vec3::new(-spread * 0.866, 0.0, spread * 0.5),
        Vec3::new(spread * 0.866, 0.0, spread * 0.5),
    ];

    for i in 0..constants::DECOY_COUNT as usize {
        let offset = if i < offsets.len() {
            offsets[i]
        } else {
            Vec3::ZERO
        };
        let spawn_pos = position + offset;

        commands.spawn((
            Mesh3d(infantry_assets.mesh.clone()),
            MeshMaterial3d(infantry_assets.defender_material.clone()),
            Transform::from_translation(Vec3::new(spawn_pos.x, 0.0, spawn_pos.z)),
            Team::Defenders,
            Health::new(constants::DECOY_HP),
            Hitbox::new(5.0, 10.0),
            IllusionDecoy::new(duration),
            PermanentCorpse, // Prevents resurrection
            OnGameplayScreen,
        ));
    }
}

fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = constants::CIRCLE_RADIUS * empowerment;
    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: constants::CIRCLE_COLOR,
        unlit: true,
        ..default()
    });
    commands
        .spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(circle_material),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            PhantasmalForceIndicator::new(position),
            OnGameplayScreen,
        ))
        .id()
}

fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    let Ok(window) = window_query.single() else {
        return None;
    };
    let cursor_position = window.cursor_position()?;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return None;
    };
    if ray.direction.y.abs() < 0.0001 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.direction * t)
}
