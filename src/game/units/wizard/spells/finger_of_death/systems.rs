use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::ScreenDesaturateMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Health, SpellDamaged, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::utils::get_cursor_world_position;

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
        (
            Entity,
            &mut CastingState,
            &Mana,
            &PrimedSpell,
            &Wizard,
        ),
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

    let Ok((wizard_entity, mut casting_state, mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::FingerOfDeath {
        return;
    }

    let awaiting_release = awaiting_release_query.get(wizard_entity).is_ok();
    let has_existing_beam = beams.iter().next().is_some();

    let result = finger_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        mana,
        primed_spell,
        wizard,
        awaiting_release,
        has_existing_beam,
    );

    // Apply component changes
    if result.remove_awaiting_release {
        commands
            .entity(wizard_entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }

    // Apply beam action
    match result.beam_action {
        BeamAction::UpdateBeam {
            origin,
            direction,
            length,
            cast_progress,
            delta_secs,
        } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
                beam.cast_progress = cast_progress;
                beam.time_alive += delta_secs;
            }
        }
        BeamAction::SpawnBeam {
            origin,
            direction,
            length,
            empowerment,
            cast_progress,
        } => {
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

    let wizard_pos = SPELL_ORIGIN;

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

/// Spawns a Finger of Death beam entity with a cylinder mesh visible from all angles,
/// plus a dark glow aura sibling entity.
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

    let beam_entity = commands
        .spawn((
            beam,
            Mesh3d(assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(instance_material),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id();

    // Spawn glow aura sibling
    let glow_material = materials
        .get(&assets.finger_of_death_glow)
        .cloned()
        .unwrap_or_default();
    let glow_instance = materials.add(glow_material);

    commands.spawn((
        FingerOfDeathGlow {
            beam_entity,
        },
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(glow_instance),
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
/// Also spawns necrotic vein particles from hit units, a ground pulse ring, and screen desaturation.
#[allow(clippy::too_many_arguments)]
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
    mut wizard_query: Query<(Entity, &mut Mana, &mut CastingState), With<Wizard>>,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut desaturate: MessageWriter<ScreenDesaturateMessage>,
) {
    let mut any_fired = false;
    // Rotation to make a quad lie flat on the ground (XZ plane).
    const UPWARD_ROTATION: Quat = Quat::from_xyzw(-std::f32::consts::FRAC_1_SQRT_2, 0.0, 0.0, std::f32::consts::FRAC_1_SQRT_2);

    let mut hit_positions: Vec<Vec3> = Vec::new();
    let mut beam_origin = Vec3::ZERO;

    for mut beam in beams.iter_mut() {
        // Only apply damage if cast is complete and hasn't fired yet
        if beam.has_fired || beam.cast_progress < 1.0 {
            continue;
        }

        // Mark as fired
        beam.has_fired = true;
        any_fired = true;
        beam_origin = beam.origin;

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
                    hit_positions.push(transform.translation);
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
                commands
                    .entity(wizard_entity)
                    .insert(AwaitingFingerOfDeathRelease);
            }
        }

        // Mark mouse hold as consumed to prevent immediate recast
        mouse_state.left_consumed = true;

        // Trigger screen desaturation
        desaturate.write(ScreenDesaturateMessage);

        // Spawn necrotic vein particles from each hit unit
        let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
        for hit_pos in &hit_positions {
            let ground_pos = Vec3::new(hit_pos.x, constants::VEIN_Y_POSITION, hit_pos.z);
            for i in 0..constants::VEIN_COUNT {
                let angle = i as f32 * golden_angle;
                let dir = Vec3::new(angle.cos(), 0.0, angle.sin());

                let vein_material = materials
                    .get(&visual_assets.necrotic_vein)
                    .cloned()
                    .unwrap_or_default();
                let instance = materials.add(vein_material);

                commands.spawn((
                    NecroticVein {
                        velocity: dir * constants::VEIN_SPEED,
                        time_alive: 0.0,
                        lifetime: constants::VEIN_LIFETIME,
                        base_size: constants::VEIN_SIZE,
                        wander_phase: i as f32 * 1.7,
                    },
                    Mesh3d(visual_assets.particle_quad.clone()),
                    MeshMaterial3d(instance),
                    Transform::from_translation(ground_pos)
                        .with_rotation(UPWARD_ROTATION)
                        .with_scale(Vec3::splat(constants::VEIN_SIZE)),
                    OnGameplayScreen,
                ));
            }
        }

        // Spawn necrotic pulse ring at beam origin
        let pulse_material = materials
            .get(&visual_assets.necrotic_pulse)
            .cloned()
            .unwrap_or_default();
        let pulse_instance = materials.add(pulse_material);

        commands.spawn((
            NecroticPulse {
                time_alive: 0.0,
                lifetime: constants::PULSE_LIFETIME,
                max_radius: constants::PULSE_MAX_RADIUS,
            },
            Mesh3d(visual_assets.unit_circle.clone()),
            MeshMaterial3d(pulse_instance),
            Transform::from_translation(Vec3::new(
                beam_origin.x,
                constants::PULSE_Y_POSITION,
                beam_origin.z,
            ))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(1.0)),
            OnGameplayScreen,
        ));
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

/// Updates necrotic vein particles: meander, fade color, shrink, despawn when expired.
pub fn update_necrotic_veins(
    mut commands: Commands,
    time: Res<Time>,
    mut veins: Query<(Entity, &mut NecroticVein, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut vein, mut transform, material_handle) in veins.iter_mut() {
        vein.time_alive += dt;

        if vein.time_alive >= vein.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = vein.time_alive / vein.lifetime;

        // Meander: apply lateral sine-wave offset to velocity direction
        let wander_offset = (t * constants::VEIN_WANDER_FREQUENCY + vein.wander_phase).sin()
            * constants::VEIN_WANDER_AMPLITUDE
            * dt;
        let lateral = Vec3::new(-vein.velocity.z, 0.0, vein.velocity.x).normalize_or_zero();
        transform.translation += vein.velocity * dt + lateral * wander_offset;

        // Clamp Y to ground level
        transform.translation.y = constants::VEIN_Y_POSITION;

        // Shrink over lifetime
        let scale = vein.base_size * (1.0 - progress);
        transform.scale = Vec3::splat(scale);

        // Animate material: purple → dark purple → black, alpha fading out
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let r = 0.6 * (1.0 - progress);
            let g = 0.0;
            let b = 0.8 * (1.0 - progress * 0.5);
            let alpha = 0.8 * (1.0 - progress);
            material.base_color = Color::srgba(r, g, b, alpha);
            let em_scale = 1.0 - progress;
            material.emissive = LinearRgba::new(1.5 * em_scale, 0.0, 2.0 * em_scale, 1.0);
        }
    }
}

/// Updates the glow aura to follow its beam, pulsing and fading with the beam lifecycle.
pub fn update_finger_of_death_glow(
    mut glow_query: Query<(
        &FingerOfDeathGlow,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    beam_query: Query<(&FingerOfDeathBeam, &Transform), Without<FingerOfDeathGlow>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut glow_transform, material_handle) in glow_query.iter_mut() {
        let Ok((beam, beam_transform)) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        // Copy position and rotation from beam
        glow_transform.translation = beam_transform.translation;
        glow_transform.rotation = beam_transform.rotation;

        // Width: beam width * multiplier with subtle sine-wave pulsing
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let glow_width = base_width * constants::GLOW_WIDTH_MULTIPLIER * pulse;
        let beam_length = beam.length;
        glow_transform.scale = Vec3::new(glow_width, beam_length, glow_width);

        // Alpha: follow beam lifecycle
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = if beam.has_fired {
                // Fade out with beam
                let fade = (1.0 - beam.time_since_fired / constants::POST_FIRE_DURATION).max(0.0);
                0.25 * fade
            } else {
                // Fade in during cast
                0.15 * beam.cast_progress
            };
            material.base_color = Color::srgba(0.2, 0.0, 0.3, alpha);
        }
    }
}

/// Despawns glow entities when their beam no longer exists.
pub fn cleanup_finger_of_death_glow(
    mut commands: Commands,
    glow_query: Query<(Entity, &FingerOfDeathGlow)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if beam_query.get(glow.beam_entity).is_err() {
            commands.entity(glow_entity).despawn();
        }
    }
}

/// Updates necrotic pulse ring: expand scale, fade alpha, despawn when expired.
pub fn update_necrotic_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(
        Entity,
        &mut NecroticPulse,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut pulse, mut transform, material_handle) in pulses.iter_mut() {
        pulse.time_alive += dt;

        if pulse.time_alive >= pulse.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = pulse.time_alive / pulse.lifetime;

        // Expand from small to max_radius
        let radius = pulse.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        // Fade alpha from 0.5 to 0
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 0.5 * (1.0 - progress);
            material.base_color = Color::srgba(0.4, 0.0, 0.6, alpha);
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
