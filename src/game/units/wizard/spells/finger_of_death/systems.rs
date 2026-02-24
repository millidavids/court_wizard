use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Spell, LocalWizard, Wizard, WizardInput};
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Health, SpellDamaged, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new data.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        cast_progress: f32,
        delta_secs: f32,
    },
    /// Spawn a new beam (optionally with initial cast progress).
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
        cast_progress: f32,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
    /// Whether to remove the AwaitingFingerOfDeathRelease component.
    remove_awaiting_release: bool,
}

/// Local wizard Finger of Death casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_finger_of_death_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    awaiting_release_query: Query<(), With<AwaitingFingerOfDeathRelease>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut beams: Query<(Entity, &mut FingerOfDeathBeam)>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, mut casting_state, mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::FingerOfDeath { return; }

    let awaiting_release = awaiting_release_query.get(wizard_entity).is_ok();
    let has_existing_beam = beams.iter().next().is_some();

    let result = finger_of_death_casting_logic(
        &input,
        &time,
        wizard_transform,
        &mut casting_state,
        mana,
        primed_spell,
        wizard,
        awaiting_release,
        has_existing_beam,
    );

    // Apply component changes
    if result.remove_awaiting_release {
        commands.entity(wizard_entity).remove::<AwaitingFingerOfDeathRelease>();
    }

    // Apply beam action
    match result.beam_action {
        BeamAction::UpdateBeam { origin, direction, length, cast_progress, delta_secs } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
                beam.cast_progress = cast_progress;
                beam.time_alive += delta_secs;
            }
        }
        BeamAction::SpawnBeam { origin, direction, length, empowerment, cast_progress } => {
            let mut new_beam = FingerOfDeathBeam::new(origin, direction, length, empowerment);
            new_beam.cast_progress = cast_progress;
            spawn_beam(&mut commands, &visual_assets, &mut materials, new_beam);
        }
        BeamAction::DespawnAll => {
            for (beam_entity, _) in beams.iter() {
                commands.entity(beam_entity).despawn();
            }
        }
        BeamAction::None => {}
    }
}

/// Core Finger of Death casting logic -- called by the local system.
#[allow(clippy::too_many_arguments)]
fn finger_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_transform: &Transform,
    casting_state: &mut CastingState,
    mana: &Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    awaiting_release: bool,
    has_existing_beam: bool,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
        remove_awaiting_release: false,
    };

    let wizard_pos = wizard_transform.translation;

    // Check for release event
    if input.just_released {
        result.remove_awaiting_release = true;
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Channeling { .. } => {
            // Finger of Death doesn't channel - just cancel
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Calculate beam target
            if let Some(cursor_pos) = input.cursor_pos {
                let beam_origin =
                    wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                // Clamp target position to spell range
                let to_target = cursor_pos - beam_origin;
                let distance = to_target.length();
                let clamped_target = if distance > wizard.spell_range {
                    beam_origin + to_target.normalize() * wizard.spell_range
                } else {
                    cursor_pos
                };

                let direction = (clamped_target - beam_origin).normalize();
                let beam_length = (clamped_target - beam_origin)
                    .length()
                    .min(constants::BEAM_LENGTH);

                // Calculate cast progress
                let cast_progress = (casting_state.progress(primed_spell.cast_time)).min(1.0);

                // Update existing beam or spawn new one
                if has_existing_beam {
                    result.beam_action = BeamAction::UpdateBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        cast_progress,
                        delta_secs: time.delta_secs(),
                    };
                } else {
                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress,
                    };
                }
            }
        }
        CastingState::Resting => {
            // Not casting - check if we're waiting for mouse release first
            if awaiting_release {
                return result;
            }

            // Check for active input
            if (input.just_pressed || input.pressed)
                && mana.percentage() >= constants::MANA_REQUIREMENT_PERCENT
            {
                casting_state.start_cast();

                // Spawn initial beam
                if let Some(cursor_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    // Clamp target position to spell range
                    let to_target = cursor_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        cursor_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                        cast_progress: 0.0,
                    };
                }
            }
        }
    }

    result
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Create a ray from the camera through the cursor position
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Intersect ray with Y=0 plane (battlefield surface)
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Spawns a Finger of Death beam entity with a cylinder mesh visible from all angles.
pub(crate) fn spawn_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    beam: FingerOfDeathBeam,
) {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    // Clone the base material so each beam can animate alpha independently
    let material = materials
        .get(&assets.finger_of_death_beam)
        .cloned()
        .unwrap_or_default();
    let instance_material = materials.add(material);

    commands.spawn((
        beam,
        Mesh3d(assets.unit_cylinder.clone()),
        MeshMaterial3d(instance_material),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));
}

/// Applies Finger of Death damage when cast completes.
///
/// Checks beams where has_fired == false and cast_progress >= 1.0.
/// Applies 1000 damage instantly to all units along beam (hitscan).
/// Drains 50% of the casting wizard's mana and cancels casting state.
/// Adds AwaitingFingerOfDeathRelease component to prevent immediate recast.
pub fn apply_finger_of_death_damage(
    mut commands: Commands,
    mut mouse_state: ResMut<MouseButtonState>,
    mut beams: Query<&mut FingerOfDeathBeam>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    mut wizard_query: Query<
        (Entity, &mut Mana, &mut CastingState),
        With<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
) {
    let mut any_fired = false;

    for mut beam in beams.iter_mut() {
        // Only apply damage if cast is complete and hasn't fired yet
        if beam.has_fired || beam.cast_progress < 1.0 {
            continue;
        }

        // Mark as fired
        beam.has_fired = true;
        any_fired = true;

        // Find nearest wall intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.length;
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.length * max_t;

        // Apply damage to all units along beam (before wall)
        let beam_width = beam.beam_width();
        let damage = beam.damage();
        for (entity, transform, mut health, mut temp_hp, has_spell_shield) in targets.iter_mut() {
            if has_spell_shield {
                continue;
            }
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - beam.origin).dot(beam.direction);
                if proj <= effective_length {
                    apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                    commands.entity(entity).insert(SpellDamaged);
                }
            }
        }
    }

    // Drain 50% mana and cancel casting state for whichever wizard(s) are actively casting
    if any_fired {
        for (wizard_entity, mut mana, mut casting_state) in wizard_query.iter_mut() {
            if !matches!(*casting_state, CastingState::Resting) {
                mana.current -= mana.max * constants::MANA_REQUIREMENT_PERCENT;
                mana.current = mana.current.max(0.0);
                casting_state.cancel();

                // Add awaiting release marker to prevent immediate recast
                commands.entity(wizard_entity).insert(AwaitingFingerOfDeathRelease);
            }
        }

        // Mark mouse hold as consumed to prevent immediate recast
        mouse_state.left_consumed = true;
    }
}

/// Updates Finger of Death beam visuals based on cast progress and fire state.
pub fn update_finger_of_death_beam_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &mut FingerOfDeathBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        // Update time_since_fired if beam has fired
        if beam.has_fired {
            beam.time_since_fired += time.delta_secs();
        }
        // Beam is always full length, doesn't grow
        let current_len = beam.length;

        // Update position to beam midpoint
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        // Calculate rotation to align the rectangle's Y axis with the beam direction
        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        // Scale the cylinder: X/Z = width, Y = length
        let width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        transform.scale = Vec3::new(width, current_len, width);

        // Update material color and alpha based on fire state and cast progress
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if beam.has_fired {
                // After fire: fade out from 100% to 0% over POST_FIRE_DURATION
                let fade_progress = beam.time_since_fired / constants::POST_FIRE_DURATION;
                let alpha = (1.0 - fade_progress).max(0.0); // 1.0 -> 0.0

                material.base_color = Color::srgba(
                    constants::BEAM_COLOR_FIRED.to_srgba().red,
                    constants::BEAM_COLOR_FIRED.to_srgba().green,
                    constants::BEAM_COLOR_FIRED.to_srgba().blue,
                    alpha,
                );
            } else {
                // During cast: fade in alpha from 0 to ALPHA_CASTING based on cast_progress
                let alpha = constants::ALPHA_CASTING * beam.cast_progress;
                material.base_color = Color::srgba(
                    constants::BEAM_COLOR_CASTING.to_srgba().red,
                    constants::BEAM_COLOR_CASTING.to_srgba().green,
                    constants::BEAM_COLOR_CASTING.to_srgba().blue,
                    alpha,
                );
            }
        }
    }
}

/// Cleans up Finger of Death beams after firing or cancellation.
pub fn cleanup_finger_of_death_beams(
    mut commands: Commands,
    beams: Query<(Entity, &FingerOfDeathBeam)>,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
) {
    let resting = wizard_query
        .single()
        .map(|state| matches!(state, CastingState::Resting))
        .unwrap_or(true);

    for (entity, beam) in beams.iter() {
        let should_despawn = if beam.has_fired {
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            resting
        };

        if should_despawn {
            commands.entity(entity).despawn();
        }
    }
}
