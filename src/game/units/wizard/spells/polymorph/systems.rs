use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{AttackTiming, Corpse, Health, PolymorphedModifier};

/// Local wizard polymorph casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_polymorph_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((_wizard_entity, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Polymorph {
        return;
    }

    let completed = polymorph_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &mut materials,
        &targets_query,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core polymorph casting logic.
#[allow(clippy::too_many_arguments)]
fn polymorph_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets_query: &Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Some(cursor_pos) = input.cursor_pos
                        && let Some((target_entity, _, target_health, target_material)) =
                            targets_query
                                .iter()
                                .filter_map(|(entity, transform, health, material)| {
                                    let dist = transform.translation.distance(cursor_pos);
                                    if dist <= constants::TARGET_SEARCH_RADIUS {
                                        Some((entity, dist, health, material))
                                    } else {
                                        None
                                    }
                                })
                                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    {
                        let duration = constants::POLYMORPH_DURATION * primed_spell.empowerment;
                        let original_material = target_material.0.clone();

                        // Create sheep material
                        let sheep_material = materials.add(StandardMaterial {
                            base_color: constants::SHEEP_COLOR,
                            ..default()
                        });

                        commands.entity(target_entity).insert((
                            PolymorphedModifier::new(
                                duration,
                                target_health.current,
                                target_health.max,
                                original_material,
                            ),
                            MeshMaterial3d(sheep_material),
                            Health::new(constants::SHEEP_HP),
                        ));
                        commands.entity(target_entity).remove::<AttackTiming>();
                    }
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    completed
}

/// Ticks polymorphed unit timers and restores them when expired.
pub fn tick_polymorphed_units(
    mut commands: Commands,
    time: Res<Time>,
    mut polymorphed: Query<(Entity, &mut PolymorphedModifier, &mut Health)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier, mut health) in &mut polymorphed {
        if modifier.update(delta) {
            // Polymorph expired - restore unit
            health.current = modifier.original_health_current;
            health.max = modifier.original_health_max;
            commands.entity(entity).insert((
                MeshMaterial3d(modifier.original_material.clone()),
                AttackTiming::new(),
            ));
            commands.entity(entity).remove::<PolymorphedModifier>();
        }
    }
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
