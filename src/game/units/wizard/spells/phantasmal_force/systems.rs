use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::PhantasmalForceIndicator;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Hitbox, IllusionDecoy, PermanentCorpse, Team};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Local wizard phantasmal force casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_phantasmal_force_casting(
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

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::PhantasmalForce {
        return;
    }
    let clamped_cursor =
        clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

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
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                if let Some(pos) = clamped_cursor {
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        &visual_assets,
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

    let completed = phantasmal_force_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        clamped_cursor,
    );

    if completed {
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

/// Core phantasmal force casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn phantasmal_force_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    _clamped_cursor: Option<Vec3>,
) -> bool {
    if input.just_released {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    completed = true;
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

    completed
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
        let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);
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

pub(crate) fn spawn_decoys(
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
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = constants::CIRCLE_RADIUS * empowerment;
    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.phantasmal_force_indicator.clone()),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
            PhantasmalForceIndicator::new(position, empowerment),
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
