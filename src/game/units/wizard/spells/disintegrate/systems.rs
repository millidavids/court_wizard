use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Spell, LocalWizard, Wizard, WizardInput};
use super::components::DisintegrateBeam;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new origin, direction, length.
    UpdateBeam { origin: Vec3, direction: Vec3, length: f32 },
    /// Spawn a new beam.
    SpawnBeam { origin: Vec3, direction: Vec3, length: f32, empowerment: f32 },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
}

/// Local wizard disintegrate casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (&Transform, &mut CastingState, &mut Mana, &PrimedSpell, &Wizard),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut beams: Query<(Entity, &mut DisintegrateBeam)>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let released = left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_transform, mut casting_state, mut mana, primed_spell, wizard)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::Disintegrate { return; }

    let has_existing_beam = beams.iter().next().is_some();

    let result = disintegrate_casting_logic(
        &input,
        &time,
        wizard_transform,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        has_existing_beam,
    );

    match result.beam_action {
        BeamAction::UpdateBeam { origin, direction, length } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
            }
        }
        BeamAction::SpawnBeam { origin, direction, length, empowerment } => {
            spawn_beam(&mut commands, &visual_assets, origin, direction, length, empowerment);
        }
        BeamAction::DespawnAll => {
            for (entity, _) in beams.iter() {
                commands.entity(entity).despawn();
            }
        }
        BeamAction::None => {}
    }
}

/// Core disintegrate casting logic.
///
/// Takes extracted data from queries and returns actions for the wrapper to apply.
#[allow(clippy::too_many_arguments)]
fn disintegrate_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_transform: &Transform,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    has_existing_beam: bool,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
    };

    let wizard_pos = wizard_transform.translation;

    // Check for release
    if input.just_released {
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            let mana_cost = constants::MANA_COST_PER_SECOND * time.delta_secs();

            if mana.consume(mana_cost) {
                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    if has_existing_beam {
                        result.beam_action = BeamAction::UpdateBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                        };
                    } else {
                        result.beam_action = BeamAction::SpawnBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                            empowerment: primed_spell.empowerment,
                        };
                    }
                }
            } else {
                casting_state.cancel();
                result.beam_action = BeamAction::DespawnAll;
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                casting_state.start_channeling();

                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
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
                    };
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && mana.can_afford(constants::MANA_COST_PER_SECOND * 0.1)
            {
                casting_state.start_cast();
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
    // Ray equation: origin + direction * t
    // Plane equation: y = 0
    // Solve for t: origin.y + direction.y * t = 0
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// System that applies damage to all units hit by disintegrate beams.
///
/// This is a high-risk spell that damages both attackers and defenders,
/// but not the wizard.
pub fn apply_disintegrate_damage(
    mut commands: Commands,
    mut beam_query: Query<&mut DisintegrateBeam>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    time: Res<Time>,
) {
    for mut beam in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        // Find the nearest wall intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.current_length();
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.current_length() * max_t;

        if beam.should_damage() {
            for (entity, transform, mut health, mut temp_hp, has_spell_shield) in target_query.iter_mut() {
                let position = transform.translation;
                // Check if point is in beam AND before the wall
                if beam.contains_point(position) {
                    let proj = (position - beam.origin).dot(beam.direction);
                    if proj <= effective_length {
                        apply_spell_damage(
                            &mut commands,
                            entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            beam.damage_per_tick(),
                            constants::DAMAGE_TYPE,
                            has_spell_shield,
                        );
                    }
                }
            }

            beam.reset_damage_timer();
        }
    }
}

/// System that despawns beams when wizard is not actively casting/channeling disintegrate.
///
/// Checks CastingState directly to avoid deferred command timing issues.
/// Excludes crystal-spawned beams (those with CrystalSpawn) — they're managed by the crystal.
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
    beam_query: Query<Entity, (With<DisintegrateBeam>, Without<CrystalSpawn>)>,
) {
    if let Ok(casting_state) = wizard_query.single() {
        if matches!(casting_state, CastingState::Resting) {
            for entity in beam_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawns a beam entity with a cylinder mesh visible from all angles.
pub(crate) fn spawn_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
) -> Entity {
    let midpoint = origin + direction * (length / 2.0);

    commands.spawn((
        DisintegrateBeam::new(origin, direction, length, empowerment),
        Mesh3d(assets.unit_cylinder.clone()),
        MeshMaterial3d(assets.disintegrate_beam.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    )).id()
}

/// Spawns a beam entity with custom damage per tick (for crystal use).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_damage(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    damage_per_tick: f32,
) -> Entity {
    let midpoint = origin + direction * (length / 2.0);

    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.damage_per_tick_override = Some(damage_per_tick);

    commands.spawn((
        beam,
        Mesh3d(assets.unit_cylinder.clone()),
        MeshMaterial3d(assets.disintegrate_beam.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    )).id()
}

/// System that updates beam cylinder transform to match beam data.
pub fn update_beam_visuals(mut beam_query: Query<(&DisintegrateBeam, &mut Transform)>) {
    for (beam, mut transform) in beam_query.iter_mut() {
        // Get current animated length
        let current_len = beam.current_length();

        // Update position to beam midpoint
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        // Align cylinder Y axis with beam direction
        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        // Scale: X/Z = beam width (cylinder radius), Y = beam length
        let beam_width = constants::BEAM_WIDTH * beam.empowerment;
        transform.scale = Vec3::new(beam_width, current_len, beam_width);
    }
}
